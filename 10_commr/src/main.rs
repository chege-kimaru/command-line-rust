use std::{
    fs::File,
    io::{self, BufRead, BufReader},
};
use std::cmp::Ordering::*;

use anyhow::{anyhow, bail, Ok, Result};
use clap::{Arg, ArgAction, Command, Parser};

use crate::Column::*;

/// Rust version of `comm`
#[derive(Debug, Parser)]
#[command(about, author, version)]
pub struct Args {
    /// Input file 1
    #[arg(value_name = "FILE1")]
    file1: String,

    /// Input file 2
    #[arg(value_name = "FILE2")]
    file2: String,

    /// Suppress printing of column 1
    #[arg(short = '1', action = ArgAction::SetFalse)]
    show_col1: bool,

    /// Suppress printing of column 2
    #[arg(short = '2', action = ArgAction::SetFalse)]
    show_col2: bool,

    /// Suppress printing of column 3
    #[arg(short = '3', action = ArgAction::SetFalse)]
    show_col3: bool,

    /// Case insensitive comparison of lines
    #[arg(short)]
    insensitive: bool,

    /// Output delimiter
    #[arg(
        short,
        long = "output-delimiter",
        value_name = "DELIM",
        default_value = "\t"
    )]
    delimiter: String,
}

enum Column<'a> {
    Col1(&'a str),
    Col2(&'a str),
    Col3(&'a str),
}

fn main() {
    if let Err(e) = run(Args::parse()) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<()> {
    let file1 = &args.file1;
    let file2 = &args.file2;

    if file1 == "-" && file2 == "-" {
        bail!(r#"Both input files cannot be STDIN ("-")"#);
    }

    let case = |line: String| {
        if args.insensitive {
            line.to_lowercase()
        } else {
            line
        }
    };

    let mut lines1 = open(file1)?.lines().map_while(Result::ok).map(case); // Open the files, create iterators that remove errors, and then map the lines through the case closure. Result::ok is a shorter version of |line| line.ok()
    let mut lines2 = open(file2)?.lines().map_while(Result::ok).map(case);

    // let mut curr1 = lines1.next();
    // let mut curr2 = lines2.next();

    // loop {
    //     match &curr1 {
    //         Some(line1) => match &curr2 {
    //             Some(line2) => {
    //                 if line1 == line2 {
    //                     if args.show_col3 {
    //                         println!(
    //                             "{}{}{line1}",
    //                             if args.show_col1 { &args.delimiter } else { "" },
    //                             if args.show_col2 { &args.delimiter } else { "" }
    //                         );
    //                     }
    //                     curr1 = lines1.next();
    //                     curr2 = lines2.next();
    //                 } else if line1 < line2 {
    //                     if args.show_col1 {
    //                         println!("{line1}");
    //                     }
    //                     curr1 = lines1.next();
    //                 } else if line2 < line1 {
    //                     if args.show_col2 {
    //                         println!(
    //                             "{}{line2}",
    //                             if args.show_col1 { &args.delimiter } else { "" }
    //                         );
    //                     }
    //                     curr2 = lines2.next();
    //                 }
    //             }
    //             None => {
    //                 if args.show_col1 {
    //                     println!("{line1}");
    //                 }
    //                 curr1 = lines1.next();
    //             }
    //         },
    //         None => match &curr2 {
    //             Some(line2) => {
    //                 if args.show_col2 {
    //                     println!(
    //                         "{}{line2}",
    //                         if args.show_col1 { &args.delimiter } else { "" }
    //                     );
    //                 }
    //                 curr2 = lines2.next();
    //             }
    //             None => {
    //                 break;
    //             }
    //         },
    //     }
    // }

    let print = |col: Column| {
        let mut columns = vec![];
        match col {
            Col1(val) => {
                if args.show_col1 {
                    columns.push(val);
                }
            },
            Col2(val) => {
                if args.show_col2 {
                    if args.show_col1 {
                        columns.push("");
                    }
                    columns.push(val);
                }
            }, 
            Col3(val) => {
                if args.show_col3 {
                    if args.show_col1 {
                        columns.push("");
                    }
                    if args.show_col2 {
                        columns.push("");
                    }
                    columns.push(val);
                }
            }
        }

        if !columns.is_empty() {
            println!("{}", columns.join(&args.delimiter));
        }
    };

    let mut line1 = lines1.next();
    let mut line2 = lines2.next();

    while line1.is_some() || line2.is_some() {
        match (&line1, &line2) {
            (Some(val1), Some(val2)) => match val1.cmp(val2) {
                Equal => {
                    print(Col3(val1));
                    line1 = lines1.next();
                    line2 = lines2.next();
                },
                Less => {
                    print(Col1(val1));
                    line1 = lines1.next();
                },
                Greater => {
                    print(Col2(val2));
                    line2 = lines2.next();
                }
            },
            (Some(val1), None) => {
                print(Col1(val1));
                line1 = lines1.next();
            }
            (None, Some(val2)) => {
                print(Col2(val2));
                line2 = lines2.next();
            }
            _ => (),
        }
    }

    Ok(())
}

fn open(filename: &str) -> Result<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(
            File::open(filename).map_err(|e| anyhow!("{filename}: {e}"))?,
        ))),
    }
}

fn _get_args() -> Args {
    let matches = Command::new("commr")
        .version("0.1.0")
        .author("Kevin Chege <kevinchege@gmail.com>")
        .about("Rust version of `comm`")
        .arg(
            Arg::new("file1")
                .value_name("FILE1")
                .help("Input file 1")
                .required(true),
        )
        .arg(
            Arg::new("file2")
                .value_name("FILE2")
                .help("Input file 2")
                .required(true),
        )
        .arg(
            Arg::new("suppress_col1")
                .short('1')
                .action(ArgAction::SetTrue)
                .help("Suppress printing of column 1"),
        )
        .arg(
            Arg::new("suppress_col2")
                .short('2')
                .action(ArgAction::SetTrue)
                .help("Suppress printing of column 2"),
        )
        .arg(
            Arg::new("suppress_col3")
                .short('3')
                .action(ArgAction::SetTrue)
                .help("Suppress printing of column 3"),
        )
        .arg(
            Arg::new("insensitive")
                .short('i')
                .action(ArgAction::SetTrue)
                .help("Case-insensitive comparison of lines"),
        )
        .arg(
            Arg::new("delimiter")
                .short('d')
                .long("output-delimiter")
                .value_name("DELIM")
                .help("Output delimiter")
                .default_value("\t"),
        )
        .get_matches();
    Args {
        file1: matches.get_one("file1").cloned().unwrap(),
        file2: matches.get_one("file2").cloned().unwrap(),
        show_col1: !matches.get_flag("suppress_col1"),
        show_col2: !matches.get_flag("suppress_col2"),
        show_col3: !matches.get_flag("suppress_col3"),
        insensitive: matches.get_flag("insensitive"),
        delimiter: matches.get_one("delimiter").cloned().unwrap(),
    }
}

// comm tests/inputs/file1.txt tests/inputs/file2.txt | sed "s/\t/--->/g"
// The sed (stream editor) command s// will substitute values, replacing the string between the first pair of slashes with the string between the second pair. The final g is the global flag to substitute every occurrence.
