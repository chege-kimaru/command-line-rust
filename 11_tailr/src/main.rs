use std::{
    fs::File,
    io::{BufRead, BufReader, Read, Seek, SeekFrom},
};

use crate::TakeValue::*;
use anyhow::{anyhow, bail, Result};
use clap::{Arg, ArgAction, Command, Parser};
use once_cell::sync::OnceCell;
use regex::Regex;

static NUM_RE: OnceCell<Regex> = OnceCell::new();

#[derive(Debug, Parser)]
#[command(about, author, version)]
/// Rust version of `tail`
struct Args {
    /// Input file(s)
    #[arg(value_name = "FILE", required = true)]
    files: Vec<String>,

    /// Number of lines
    #[arg(short = 'n', long, value_name = "LINES", default_value = "10")]
    lines: String,

    /// Number of bytes
    #[arg(short = 'c', long, value_name = "BYTES", conflicts_with = "lines")]
    bytes: Option<String>,

    /// Supress headers
    #[arg(short, long)]
    quiet: bool,
}

// 0 means nothing should be selected (-0) and +0 means everything should be selected
#[derive(Debug, PartialEq)] // PartialEq for comparing
enum TakeValue {
    PlusZero,     // represents +0
    TakeNum(i64), // Represents valid integer value
}

fn main() {
    if let Err(e) = run(Args::parse()) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<()> {
    let lines = parse_num(args.lines).map_err(|e| anyhow!("illegal line count -- {e}"))?;

    let bytes = args
        .bytes
        .map(parse_num)
        .transpose()
        .map_err(|e| anyhow!("illegal byte count -- {e}"))?;

    let num_files = args.files.len();
    for (file_num, filename) in args.files.iter().enumerate() {
        match File::open(&filename) {
            Err(err) => eprintln!("{filename}: {err}"),
            Ok(file) => {
                if !args.quiet && num_files > 1 {
                    println!(
                        "{}==> {filename} <==",
                        if file_num > 0 { "\n" } else { "" },
                    );
                }

                let (total_lines, total_bytes) = count_lines_bytes(&filename)?;
                let file = BufReader::new(file);
                if let Some(num_bytes) = &bytes {
                    print_bytes(file, &num_bytes, total_bytes)?;
                } else {
                    print_lines(file, &lines, total_lines)?;
                }
            }
        }
    }

    Ok(())
}

// My solution - Fails some tests
// fn print_lines(mut file: impl BufRead, num_lines: &TakeValue, total_lines: i64) -> Result<()> {
//     if let Some(start_index) = get_start_index(num_lines, total_lines) {
//         let mut i = 0;
//         for line in file.lines() {
//             if i >= start_index {
//                 let line = line?;
//                 println!("{line}");
//             }
//             i = i + 1;
//         }
//     }

//     Ok(())
// }

fn print_lines(mut file: impl BufRead, num_lines: &TakeValue, total_lines: i64) -> Result<()> {
    if let Some(start) = get_start_index(num_lines, total_lines) {
        let mut line_num = 0;
        let mut buf = Vec::new();
        loop {
            let bytes_read = file.read_until(b'\n', &mut buf)?;
            if bytes_read == 0 {
                break;
            }
            if line_num >= start {
                print!("{}", String::from_utf8_lossy(&buf));
            }
            line_num += 1;
            buf.clear();
        }
    }

    Ok(())
}

// My solution - Fails some tests
// fn print_bytes<T: Read + Seek>(
//     mut file: T,
//     num_bytes: &TakeValue,
//     total_bytes: i64
// ) -> Result<()>
// // where 
// //     T: Read + Seek
// {
//     if let Some(start_index) = get_start_index(num_bytes, total_bytes) {
//         let mut i = 0;
//         for byte in file.bytes() {
//             if i >= start_index {
//                 let byte = byte?;
//                 print!("{}", String::from_utf8_lossy(&[byte]));
//             }
//             i = i + 1;
//         }
//     }

//     Ok(())
// }

fn print_bytes<T: Read + Seek>(
    mut file: T,
    num_bytes: &TakeValue,
    total_bytes: i64
) -> Result<()>
// where 
//     T: Read + Seek
{
    if let Some(start) = get_start_index(num_bytes, total_bytes) {
        file.seek(SeekFrom::Start(start))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        if !buffer.is_empty() {
            print!("{}", String::from_utf8_lossy(&buffer));
        }
    }

    Ok(())
}

// My solution
// fn get_start_index(take_val: &TakeValue, total: i64) -> Option<u64> {
//     if total == 0 {
//         return None
//     }
//     match take_val {
//         PlusZero => Some(0),
//         TakeNum(num) => {
//             if num > &0 { // Positive numbers
//                 if num <= &total {
//                     Some((num - 1) as u64)
//                 } else {
//                     None // Return nothing
//                 }
//             } else if num < &0 { // Negatve numbers
//                 if num.wrapping_abs() <= total {
//                     Some((total + num) as u64) // Note num is a negative number eg 10 + -2
//                 } else {
//                     Some(0) // Return whole file
//                 }
//             } else { // Zero
//                 None
//             }
//         }
//     }
// }

fn get_start_index(take_val: &TakeValue, total: i64) -> Option<u64> {
    match take_val {
        PlusZero => {
            if total > 0 { 
                Some(0)
            } else {
                None
            }
        }
        TakeNum(num) => {
            if num == &0 || total == 0 || num > &total { 
                None
            } else {
                let start = if num < &0 { total + num } else { num - 1 }; 
                Some(if start < 0 { 0 } else { start as u64 }) 
            }
        }
    }
}

fn count_lines_bytes(filename: &str) -> Result<(i64, i64)> {
    let mut file = BufReader::new(File::open(filename)?);
    let mut num_lines = 0;
    let mut num_bytes = 0;
    let mut buf = Vec::new();
    loop {
        let bytes_read = file.read_until(b'\n', &mut buf)?;
        if bytes_read == 0 {
            break;
        }
        num_lines = num_lines + 1;
        num_bytes = num_bytes + bytes_read as i64;
        buf.clear();
    }
    Ok((num_lines, num_bytes))
}

fn parse_num(val: String) -> Result<TakeValue> {
    let num_re = NUM_RE.get_or_init(|| Regex::new(r"^([+-])?(\d+)$").unwrap()); // Note "-"" is used in cases like 0-9, put it at the end in this case, to show it is used to match a hyphen

    match num_re.captures(&val) {
        Some(caps) => {
            let sign = caps.get(1).map_or("-", |m| m.as_str());
            let signed_num = format!("{sign}{}", caps.get(2).unwrap().as_str());

            if let Ok(num) = signed_num.parse() {
                if sign == "+" && num == 0 {
                    Ok(PlusZero)
                } else {
                    Ok(TakeNum(num))
                }
            } else {
                bail!(val)
            }
        }
        _ => bail!(val),
    }
}

// fn parse_num(val: String) -> Result<TakeValue> {
//     let signs: &[char] = &['+', '-']; // The type annotation is required because Rust infers the type &[char; 2], which is a reference to an array, but I want to coerce the value to a slice.
//     let res = val
//         .starts_with(signs)
//         .then(|| val.parse())
//         .unwrap_or_else(|| val.parse().map(i64::wrapping_neg)); // a positive value will be returned as negative, while a negative value will remain negative

//     match res {
//         Ok(num) => {
//             if num == 0 && val.starts_with('+') {
//                 Ok(PlusZero)
//             } else {
//                 Ok(TakeNum(num))
//             }
//         },
//         _ => bail!(val),
//     }
// }

// fn parse_num(val: String) -> Result<TakeValue> {
//     match val.parse::<i64>() {
//         Err(_e) => bail!(val),
//         Ok(num) => {
//             if val.starts_with("+") {
//                 if num == 0 {
//                     Ok(PlusZero)
//                 } else {
//                     Ok(TakeNum(num))
//                 }
//             } else {
//                 Ok(TakeNum(if num < 0 { num } else { -num }))
//             }
//         }
//     }
// }

fn _get_args() -> Args {
    let matches = Command::new("tailr")
        .version("0.1.0")
        .author("Kevin Chege <chege.kimaru@gmail.com>")
        .about("Rust version of `tail`")
        .arg(
            Arg::new("files")
                .value_name("FILE")
                .help("Input file(s)")
                .required(true)
                .num_args(1..),
        )
        .arg(
            Arg::new("lines")
                .short('n')
                .long("lines")
                .value_name("LINES")
                .help("Number of lines")
                .default_value("10"),
        )
        .arg(
            Arg::new("bytes")
                .short('c')
                .long("bytes")
                .value_name("BYTES")
                .conflicts_with("lines")
                .help("Number of bytes"),
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .action(ArgAction::SetTrue)
                .help("Suppress headers"),
        )
        .get_matches();

    Args {
        files: matches.get_many("files").unwrap().cloned().collect(),
        lines: matches.get_one("lines").cloned().unwrap(),
        bytes: matches.get_one("bytes").cloned(),
        quiet: matches.get_flag("quiet"),
    }
}

#[cfg(test)]
mod tests {
    use super::{count_lines_bytes, get_start_index, parse_num, TakeValue::*};

    #[test]
    fn test_parse_num() {
        // All integers should be interpreted as negative numbers
        let res = parse_num("3".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(-3));

        // A leading "+" should result in a positive number
        let res = parse_num("+3".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(3));

        // An explicit "-" value should result in a negative number
        let res = parse_num("-3".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(-3));

        // Zero is zero
        let res = parse_num("0".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(0));

        // Plus zero is special
        let res = parse_num("+0".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), PlusZero);

        // Test boundaries
        let res = parse_num(i64::MAX.to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN + 1));

        let res = parse_num((i64::MIN + 1).to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN + 1));

        let res = parse_num(format!("+{}", i64::MAX));
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MAX));

        let res = parse_num(i64::MIN.to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN));

        // A floating-point value is invalid
        let res = parse_num("3.14".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "3.14");

        // Any non-integer string is invalid
        let res = parse_num("foo".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "foo");
    }

    #[test]
    fn test_count_lines_bytes() {
        let res = count_lines_bytes("tests/inputs/one.txt");
        assert!(res.is_ok());
        let (lines, bytes) = res.unwrap();
        assert_eq!(lines, 1);
        assert_eq!(bytes, 24);

        let res = count_lines_bytes("tests/inputs/twelve.txt");
        assert!(res.is_ok());
        let (lines, bytes) = res.unwrap();
        assert_eq!(lines, 12);
        assert_eq!(bytes, 63);
    }

    #[test]
    fn test_get_start_index() {
        // +0 from an empty file (0 lines/bytes) returns None
        assert_eq!(get_start_index(&PlusZero, 0), None);

        // +0 from a nonempty file returns an index that
        // is one less than the number of lines/bytes
        assert_eq!(get_start_index(&PlusZero, 1), Some(0));

        // Taking 0 lines/bytes returns None
        assert_eq!(get_start_index(&TakeNum(0), 1), None);

        // Taking any lines/bytes from an empty file returns None
        assert_eq!(get_start_index(&TakeNum(1), 0), None);

        // Taking more lines/bytes than is available returns None
        assert_eq!(get_start_index(&TakeNum(2), 1), None);

        // When starting line/byte is less than total lines/bytes,
        // return one less than starting number
        assert_eq!(get_start_index(&TakeNum(1), 10), Some(0));
        assert_eq!(get_start_index(&TakeNum(2), 10), Some(1));
        assert_eq!(get_start_index(&TakeNum(3), 10), Some(2));

        // When starting line/byte is negative and less than total,
        // return total - start
        assert_eq!(get_start_index(&TakeNum(-1), 10), Some(9));
        assert_eq!(get_start_index(&TakeNum(-2), 10), Some(8));
        assert_eq!(get_start_index(&TakeNum(-3), 10), Some(7));

        // When starting line/byte is negative and more than total,
        // return 0 to print the whole file
        assert_eq!(get_start_index(&TakeNum(-20), 10), Some(0));
    }
}


// Benchmarking
// To bench mark, use command `time tail 1M.txt > /dev/null`
// We don’t want to see the output from the command, so we redirect it to /dev/null, a special system device that ignores its input.
// The real time is wall clock time, measuring how long the process took from start to finish.
// The user time is how long the CPU spent in user mode outside the kernel
// The sys time is how long the CPU spent working inside the kernel

// To build a faster version of tail, create a release build using `cargo build --release`. The binary will be created at target​/⁠release/tailr

// To better benchmark, use hyperfine Rust crate. `cargo install hyperfine`
// Usage: hyperfine -i -L prg tail,target/release/tailr '{prg} 1M.txt > /dev/null'