use clap::{Arg, ArgAction, Command, Parser};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

// #[derive(Debug)]
// struct Args {
//     files: Vec<String>,
//     number_lines: bool,
//     number_nonblank_lines: bool,
// }

#[derive(Debug, Parser)]
#[command(author, version, about)]
/// Rust version of `cat`
struct Args {
    /// Input files(s)
    #[arg(value_name = "FILE", default_value = "-")]
    files: Vec<String>,

    /// Number lines
    #[arg(short = 'n', long = "number", conflicts_with = "number_nonblank_lines")]
    number_lines: bool,

    /// Number non-blank lines
    #[arg(short = 'b', long = "number-nonblank")]
    number_nonblank_lines: bool,
}

fn main() {
    // let args = get_args();
    // let args = Args::parse();
    // println!("{args:#?}");

    if let Err(e) = run(Args::parse()) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<(), Box<dyn Error>> {
    for filename in args.files {
        match open(&filename) {
            Err(err) => eprintln!("Failed to open {filename}: {err}"),
            Ok(file) => {
                let mut prev_num = 0;
                for (line_num, line) in file.lines().enumerate() {
                    let line = line?;

                    if args.number_lines {
                        println!("{:>6}\t{line}", line_num + 1);
                    } else if args.number_nonblank_lines {
                        if line.is_empty() {
                            println!();
                        } else {
                            prev_num += 1;
                            println!("{prev_num:>6}\t{line}");
                        }
                    } else {
                        println!("{line}");
                    }
                }
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

// powershell equivalent for: cargo run -q -- -n tests/inputs/*.txt :
// cargo run -q -- -n (Get-ChildItem .\tests\inputs\*.txt)

// {:>6} indicates the width of the field as six characters with the text aligned to the right. (You can use < for left-justified and ^ for centered text.)

// use the | to pipe STDOUT from the first command to the STDIN of the second command
// cat tests/inputs/fox.txt | cargo run
// The below also works in unix where < is the input
// cargo run -q -- - < tests/inputs/fox.txt

// fn get_args() -> Args {
//     let matches = Command::new("catr")
//         .version("0.1.0")
//         .author("Kevin Chege <chege.kimaru@gmail.com>")
//         .about("Rust version of `cat`")
//         .arg(
//             Arg::new("files")
//                 .value_name("FILE")
//                 .default_value("-")
//                 .help("Input file(s)")
//                 .num_args(1..),
//         )
//         .arg(
//             Arg::new("number")
//                 .long("number")
//                 .short('n')
//                 .action(ArgAction::SetTrue)
//                 .help("Number lines")
//                 .conflicts_with("number_nonblank"),
//         )
//         .arg(
//             Arg::new("number_nonblank")
//                 .long("number-nonblank")
//                 .short('b')
//                 .action(ArgAction::SetTrue)
//                 .help("Number non-blank lines"),
//         )
//         .get_matches();

//     Args {
//         files: matches.get_many("files").unwrap().cloned().collect(),
//         number_lines: matches.get_flag("number"),
//         number_nonblank_lines: matches.get_flag("number_nonblank"),
//     }
// }
