use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    num::NonZeroUsize,
    ops::Range,
};

use anyhow::{anyhow, bail, Result};
use clap::{Arg, ArgGroup, Command, Parser};
use csv::{ReaderBuilder, StringRecord, WriterBuilder};
use regex::Regex;

#[derive(Debug, Parser)]
#[command(author, version, about)]
/// Rust version of `cut`
struct Args {
    /// Input file(s)
    #[arg(value_name = "FILES", default_value = "-")]
    files: Vec<String>,

    /// Field delimiter
    #[arg(value_name = "DELIMITER", default_value = "\t", long, short)]
    delimiter: String,

    #[command(flatten)]
    extract: ArgsExtract,
}
#[derive(Debug, clap::Args)]
#[group(required = true, multiple = false)]
struct ArgsExtract {
    /// Selected fields
    #[arg(value_name = "FIELDS", short, long)]
    fields: Option<String>,

    /// Selected bytes
    #[arg(value_name = "BYTES", short, long)]
    bytes: Option<String>,

    /// Selected chars
    #[arg(value_name = "CHARS", short, long)]
    chars: Option<String>,
}

type PositionList = Vec<Range<usize>>;

#[derive(Debug)]
enum Extract {
    Fields(PositionList),
    Bytes(PositionList),
    Chars(PositionList),
}

fn main() {
    if let Err(e) = run(Args::parse()) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<()> {
    let delim_bytes = args.delimiter.as_bytes();
    if delim_bytes.len() != 1 {
        bail!(r#"--delim "{}" must be a single byte"#, args.delimiter); // Raw string. Starts with r followed by 0 or more #, followed by ". " can be used inside it without escaping \"
    }
    let delimiter: u8 = *delim_bytes.first().unwrap();

    let extract = if let Some(fields) = args.extract.fields.map(parse_pos).transpose()?
    // Transposes an Option of a Result into a Result of an Option
    {
        Extract::Fields(fields)
    } else if let Some(bytes) = args.extract.bytes.map(parse_pos).transpose()? {
        Extract::Bytes(bytes)
    } else if let Some(chars) = args.extract.chars.map(parse_pos).transpose()? {
        Extract::Chars(chars)
    } else {
        unreachable!("Must have --fields, --bytes, or --chars");
    };

    for filename in args.files {
        match open(&filename) {
            Err(err) => eprint!("{filename}: {err}"),
            Ok(file) => {
                match &extract {
                    Extract::Fields(field_pos) => {
                        let mut reader = ReaderBuilder::new()
                            .delimiter(delimiter)
                            .has_headers(false)
                            .from_reader(file);

                        let mut wtr = WriterBuilder::new()
                            .delimiter(delimiter)
                            .from_writer(io::stdout());

                        for record in reader.records() {
                            wtr.write_record(extract_fields(&record?, field_pos))?;
                        }
                    },
                    Extract::Bytes(bytes) => {
                        for line in file.lines() {
                            println!("{}", extract_bytes(&line?, bytes));
                        }
                    },
                    Extract::Chars(chars) => {
                        for line in file.lines() {
                            println!("{}", extract_chars(&line?, chars))
                        }
                    },
                }
            }
        }
    }

    Ok(())
}

fn parse_pos(range: String) -> Result<PositionList> {
    let range_re = Regex::new(r"^(\d+)-(\d+)$").unwrap(); // The parentheses values will be captured
    range
        .split(',')
        .into_iter()
        .map(|val| {
            parse_index(val).map(|n| n..n + 1).or_else(|e| {
                range_re.captures(val).ok_or(e).and_then(|captures| {
                    let n1 = parse_index(&captures[1])?;
                    let n2 = parse_index(&captures[2])?;
                    if n1 >= n2 {
                        bail!(
                            "First number in range ({}) \
                            must be lower than second number ({})",
                            n1 + 1,
                            n2 + 1
                        );
                    }
                    Ok(n1..n2 + 1)
                })
            })
        })
        .collect::<Result<_, _>>()
        .map_err(From::from)
}

fn parse_index(input: &str) -> Result<usize> {
    let value_error = || anyhow!(r#"illegal list value: "{input}""#);
    input
        .starts_with('+')
        .then(|| Err(value_error()))
        .unwrap_or_else(|| {
            input
                .parse::<NonZeroUsize>()
                .map(|n| usize::from(n) - 1)
                .map_err(|_| value_error())
        })
}

fn open(filename: &str) -> Result<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

fn extract_chars(line: &str, char_pos: &[Range<usize>]) -> String {
    let chars: Vec<_> = line.chars().collect();
    // let mut selected: Vec<char> = vec![];

    // for range in char_pos.iter().cloned() {
    //     // for i in range {
    //     //     if let Some(val) = chars.get(i) {
    //     //         selected.push(*val);
    //     //     }
    //     // }

    //     selected.extend(range.filter_map(|i| chars.get(i))); // filter map yields only values for which the supplied closure returns Some(value)
    // }

    // selected.iter().collect()

    // char_pos
    //     .iter()
    //     .cloned()
    //     .map(|range| range.filter_map(|i| chars.get(i)))
    //     .flatten() // To remove nested structures
    //     .collect()

    char_pos
        .iter()
        .cloned()
        .flat_map(|range| range.filter_map(|i| chars.get(i))) // flat map combibes map and flatten
        .collect()
}

fn extract_bytes(line: &str, byte_pos: &[Range<usize>]) -> String {
    let bytes = line.as_bytes();
    let selected: Vec<_> = byte_pos
        .iter()
        .cloned()
        .flat_map(|range| range.filter_map(|i| bytes.get(i)).copied()) // get returns a byte reference (&u8). We need copied to convert (create a copy of the element) to a byte (u8) as String::from_utf16_lossy expectes a slice of bytes, not byte references
        .collect();
    String::from_utf8_lossy(&selected).into_owned() // Use Cow::into_owned to clone the data, if needed
}

fn extract_fields(record: &StringRecord, field_pos: &[Range<usize>]) -> Vec<String> {
    field_pos
        .iter()
        .cloned()
        .flat_map(|range| range.filter_map(|i| record.get(i)))
        .map(String::from)
        .collect()
}

// To return Vec<&str> which is slightly more memory efficient as it will not make string copies, use life times
fn _extract_fields<'a>(
    record: &'a StringRecord,
    field_pos: &[Range<usize>],
) -> Vec<&'a str> {
    field_pos
        .iter()
        .cloned()
        .flat_map(|range| range.filter_map(|i| record.get(i)))
        .collect()
}

// fn parse_pos(range: String) -> Result<PositionList> {
//     let mut res: PositionList = Vec::new();

//     if range.is_empty() {
//         bail!(r#"illegal list value: """#);
//     }

//     if range.contains("+") {
//         bail!(r#"illegal list value: "{range}""#);
//     }

//     let positions: Vec<&str> = range.split(",").collect();
//     if positions.len() == 0 {
//         bail!("illegal list value: {range}");
//     }
//     for position in positions {
//         if position.contains("-") {
//             let indeces: Vec<&str> = position.split("-").collect();
//             if indeces.len() != 2 {
//                 bail!("illegal list value: {range}");
//             }
//             for index in &indeces {
//                 if let Err(_e) = index.parse::<usize>() {
//                     bail!(r#"illegal list value: "{position}""#);
//                 }
//                 let index: usize = index.parse().unwrap();
//                 if index <= 0 {
//                     bail!(r#"illegal list value: "{index}""#);
//                 }
//             }
//             let index1: usize = indeces[0].parse().unwrap();
//             let index2: usize = indeces[1].parse().unwrap();

//             if index2 <= index1 {
//                 bail!("First number in range ({index1}) must be lower than second number ({index2})");
//             }

//             res.push(index1 - 1 .. index2);
//         } else {
//             if let Err(_e) = position.parse::<usize>() {
//                 bail!(r#"illegal list value: "{position}""#);
//             }
//             let index: usize = position.parse().unwrap();
//             if index <= 0 {
//                 bail!(r#"illegal list value: "{index}""#);
//             }
//             res.push(index -1 .. index);
//         }
//     }
//     Ok(res)
// }

fn _get_args() -> Args {
    let matches = Command::new("cutr")
        .version("0.1.0")
        .author("Kevin Chege <kevinchege@gmail.com>")
        .about("Rust version of `cut`")
        .arg(
            Arg::new("files")
                .value_name("FILES")
                .help("Input file(s)")
                .num_args(0..)
                .default_value("-"),
        )
        .arg(
            Arg::new("delimiter")
                .value_name("DELIMITER")
                .short('d')
                .long("delim")
                .help("Field delimiter")
                .default_value("\t"),
        )
        .arg(
            Arg::new("fields")
                .value_name("FIELDS")
                .short('f')
                .long("fields")
                .help("Selected fields"),
        )
        .arg(
            Arg::new("bytes")
                .value_name("BYTES")
                .short('b')
                .long("bytes")
                .help("Selected bytes"),
        )
        .arg(
            Arg::new("chars")
                .value_name("CHARS")
                .short('c')
                .long("chars")
                .help("Selected characters"),
        )
        .group(
            ArgGroup::new("extract")
                .args(["fields", "bytes", "chars"])
                .required(true)
                .multiple(false),
        )
        .get_matches();
    Args {
        files: matches.get_many("files").unwrap().cloned().collect(),
        delimiter: matches.get_one("delimiter").cloned().unwrap(),
        extract: ArgsExtract {
            fields: matches.get_one("fields").cloned(),
            bytes: matches.get_one("bytes").cloned(),
            chars: matches.get_one("chars").cloned(),
        },
    }
}

#[cfg(test)]
mod unit_tests {
    use csv::StringRecord;

    use crate::extract_fields;

    use super::{extract_bytes, extract_chars, parse_pos};

    #[test]
    fn test_parse_pos() {
        // The empty string is an error
        assert!(parse_pos("".to_string()).is_err());

        // Zero is an error
        let res = parse_pos("0".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "0""#);

        let res = parse_pos("0-1".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "0""#);

        // A leading "+" is an error
        let res = parse_pos("+1".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "+1""#,);

        let res = parse_pos("+1-2".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"illegal list value: "+1-2""#,
        );

        let res = parse_pos("1-+2".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"illegal list value: "1-+2""#,
        );

        // Any non-number is an error
        let res = parse_pos("a".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "a""#);

        let res = parse_pos("1,a".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "a""#);

        let res = parse_pos("1-a".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "1-a""#,);

        let res = parse_pos("a-1".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "a-1""#,);

        // Wonky ranges
        let res = parse_pos("-".to_string());
        assert!(res.is_err());

        let res = parse_pos(",".to_string());
        assert!(res.is_err());

        let res = parse_pos("1,".to_string());
        assert!(res.is_err());

        let res = parse_pos("1-".to_string());
        assert!(res.is_err());

        let res = parse_pos("1-1-1".to_string());
        assert!(res.is_err());

        let res = parse_pos("1-1-a".to_string());
        assert!(res.is_err());

        // First number must be less than second
        let res = parse_pos("1-1".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (1) must be lower than second number (1)"
        );

        let res = parse_pos("2-1".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (2) must be lower than second number (1)"
        );

        // All the following are acceptable
        let res = parse_pos("1".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);

        let res = parse_pos("01".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);

        let res = parse_pos("1,3".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);

        let res = parse_pos("001,0003".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);

        let res = parse_pos("1-3".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);

        let res = parse_pos("0001-03".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);

        let res = parse_pos("1,7,3-5".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 6..7, 2..5]);

        let res = parse_pos("15,19-20".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![14..15, 18..20]);
    }

    #[test]
    fn test_extract_chars() {
        assert_eq!(extract_chars("", &[0..1]), "".to_string());
        assert_eq!(extract_chars("ábc", &[0..1]), "á".to_string());
        assert_eq!(extract_chars("ábc", &[0..1, 2..3]), "ác".to_string());
        assert_eq!(extract_chars("ábc", &[0..3]), "ábc".to_string());
        assert_eq!(extract_chars("ábc", &[2..3, 1..2]), "cb".to_string());
        assert_eq!(extract_chars("ábc", &[0..1, 1..2, 4..5]), "áb".to_string());
    }

    #[test]
    fn test_extract_bytes() {
        assert_eq!(extract_bytes("ábc", &[0..1]), "�".to_string());
        assert_eq!(extract_bytes("ábc", &[0..2]), "á".to_string());
        assert_eq!(extract_bytes("ábc", &[0..3]), "áb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..4]), "ábc".to_string());
        assert_eq!(extract_bytes("ábc", &[3..4, 2..3]), "cb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..2, 5..6]), "á".to_string());
    }

    #[test]
    fn test_extract_fields() {
        let rec = StringRecord::from(vec!["Captain", "Sham", "12345"]);
        assert_eq!(extract_fields(&rec, &[0..1]), &["Captain"]);
        assert_eq!(extract_fields(&rec, &[1..2]), &["Sham"]);
        assert_eq!(extract_fields(&rec, &[0..1, 2..3]), &["Captain", "12345"]);
        assert_eq!(extract_fields(&rec, &[0..1, 3..4]), &["Captain"]);
        assert_eq!(extract_fields(&rec, &[1..2, 0..1]), &["Sham", "Captain"]);
    }
}
