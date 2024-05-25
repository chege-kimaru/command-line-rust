use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;

use anyhow::Result;
// use clap::{Arg, Command, Parser};
use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about)]
/// Rust version of `head`
struct Args {
    /// Input file(s)
    #[arg(default_value = "-", value_name = "FILE")]
    files: Vec<String>,

    /// Number of lines
    #[arg(default_value = "10", short = 'n', long = "lines", value_parser = clap::value_parser!(u64).range(1..) )]
    lines: u64,

    /// Number of bytes
    #[arg(short = 'c', long, value_name = "BYTES", conflicts_with = "lines", value_parser = clap::value_parser!(u64).range(1..))]
    bytes: Option<u64>, // long value defaults to name of the field while short value defaults to first letter of the field name
}

fn main() {
    if let Err(e) = run(Args::parse()) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<(), Box<dyn Error>> {
    let num_files = args.files.len();

    for (file_num, filename) in args.files.iter().enumerate() {
        match open(&filename) {
            Err(err) => eprintln!("{filename}: {err}"),
            Ok(mut file) => {
                if num_files > 1 {
                    // println!("{}==> {} <==", relative_path(&filename)?.display());
                    println!("{}==> {filename} <==", if file_num > 0 { "\n" } else { "" });
                }

                if let Some(num_bytes) = args.bytes {
                    let mut buffer = vec![0; num_bytes as usize]; // buffer of size num_bytes filled with zeros
                    let bytes_read = file.read(&mut buffer)?;
                    // Read upto bytes_read in case num_bytes > bytes read
                    print!("{}", String::from_utf8_lossy(&buffer[..bytes_read]));

                    // let mut contents = String::new();
                    // file.read_to_string(&mut contents)?; // Dangerous for files larger than computer memory
                    // let bytes = contents.as_bytes();
                    // print!("{}", String::from_utf8_lossy(&bytes[..num_bytes as usize])); // This will fail for empty files as bytes will be of length zero yet we are trying to access till num_bytes

                    // // let bytes: Result<Vec<_>, _> = file.bytes().take(num_bytes as usize).collect(); // The _ is used to tell compiler to infer the type. We specify type as Vec as compiler infers type of bytes as slice with unknown size. This can also be written as below:
                    // let bytes = file.bytes().take(num_bytes as usize).collect::<Result<Vec<_>, _>>();
                    // print!("{}", String::from_utf8_lossy(&bytes?));
                } else {
                    // for line in file.lines().take(args.lines as usize) {
                    //     println!("{}", line?);
                    // }
                    for _ in 0..args.lines {
                        let mut line = String::new();
                        let bytes = file.read_line(&mut line)?;
                        // The filehandle will return zero bytes when it reaches the end of the file
                        if bytes == 0 {
                            break;
                        }
                        print!("{line}");
                        line.clear();
                    }
                };

                // println!();
            }
        }
    }
    Ok(())
}

fn open(filename: &str) -> Result<Box<dyn BufRead>, Box<dyn Error>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

// fn get_args() -> Args {
//     let matches = Command::new("headr")
//         .version("0.1.0")
//         .author("Kevin Chege <chege.kimaru@gmail.com>")
//         .about("Rust version of `head`")
//         .arg(
//             Arg::new("lines")
//                 .short('n')
//                 .long("lines")
//                 .value_name("LINES")
//                 .help("Number of lines")
//                 .value_parser(clap::value_parser!(u64).range(1..))
//                 .default_value("10"),
//         )
//         .arg(
//             Arg::new("bytes")
//                 .short('c')
//                 .long("bytes")
//                 .value_name("BYTES")
//                 .conflicts_with("lines")
//                 .value_parser(clap::value_parser!(u64).range(1..))
//                 .help("Number of bytes"),
//         )
//         .arg(
//             Arg::new("files")
//                 .value_name("FILE")
//                 .help("Input file(s)")
//                 .num_args(0..)
//                 .default_value("-"),
//         )
//         .get_matches();

//     Args {
//         files: matches.get_many("files").unwrap().cloned().collect(),
//         lines: matches.get_one("lines").cloned().unwrap(),
//         bytes: matches.get_one("bytes").cloned(),
//     }
// }

// Windows new line is CRLF (carriage return line feed) (\r\n)
// Linux new line is LF (line feed) (\n)
