use std::{
    fs::File,
    io::{self, BufRead, BufReader, Write},
};

use anyhow::{anyhow, Result};
use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about)]
/// Rust version of `uniq`
struct Args {
    /// Input file
    #[arg(value_name = "IN_FILE", default_value = "-")]
    in_file: String,

    /// Output file
    #[arg(value_name = "OUT_FILE")]
    out_file: Option<String>,

    /// Show counts
    #[arg(short, long)]
    count: bool,
}

fn main() {
    if let Err(e) = run(Args::parse()) {
        eprint!("{e}");
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<()> {
    let mut file = open(&args.in_file).map_err(|e| anyhow!("{}: {e}", args.in_file))?;

    let mut out_file: Box<dyn Write> = match &args.out_file {
        Some(out_name) => Box::new(File::create(out_name)?),
        _ => Box::new(io::stdout()),
    };

    let mut print = |num: u64, text: &str| -> Result<()> {
        if num > 0 {
            if args.count {
                write!(out_file, "{num:>4} {text}")?;
            } else {
                write!(out_file, "{text}")?;
            }
        }
        Ok(())
    };

    let mut line = String::new();
    let mut previous = String::new();
    let mut count: u64 = 0;

    loop {
        let bytes = file.read_line(&mut line)?;

        // if count == 0 {
        //     previous = line.clone();
        // }

        // if line.trim_end() == previous.trim_end() {
        //     count += 1;
        // } else {
        //     if args.count {
        //         write!(out_file, "{count:>4} {previous}")?;
        //     } else {
        //         write!(out_file, "{previous}")?;
        //     }

        //     previous = line.clone();
        //     count = 1;
        // }

        // if bytes == 0 {
        //     break;
        // }

        if bytes == 0 {
            break;
        }

        if line.trim_end() != previous.trim_end() {
            print(count, &previous)?;
            previous = line.clone();
            count = 0;
        }

        count += 1;
        line.clear();
    }

    print(count, &previous)?;

    Ok(())
}

fn open(filename: &str) -> Result<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
