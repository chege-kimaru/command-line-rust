use clap::{builder::PossibleValue, Arg, ArgAction, Command, Parser, ValueEnum};
use regex::Regex;
use walkdir::{DirEntry, WalkDir};
use anyhow::Result;

#[derive(Debug, Parser)]
#[command(author, version, about)]
/// Rust version of `find`
struct Args {
    /// Search path(s)
    #[arg(value_name = "PATH", default_value = ".")]
    paths: Vec<String>,

    /// Names
    #[arg(short('n'), long("name"), value_name = "NAME", value_parser(Regex::new), action(ArgAction::Append), num_args(0..))]
    names: Vec<Regex>,

    /// Entry type
    #[arg(short('t'), long("type"), value_name = "TYPE", value_parser(clap::value_parser!(EntryType)), action(ArgAction::Append), num_args(0..))]
    entry_types: Vec<EntryType>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
enum EntryType {
    Dir,
    File,
    Link
}

impl ValueEnum for EntryType {
    fn value_variants<'a>() -> &'a [Self] {
        &[EntryType::Dir, EntryType::File, EntryType::Link]
    }

    fn to_possible_value<'a>(&self) -> Option<PossibleValue> {
        Some(match self {
            EntryType::Dir => PossibleValue::new("d"),
            EntryType::File => PossibleValue::new("f"),
            EntryType::Link => PossibleValue::new("l"),
        })
    }
}


fn main() {
    if let Err(e) = run(Args::parse()) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<()> {

    let type_filter = |entry: &DirEntry| {
        args.entry_types.is_empty() || args.entry_types.iter().any(|e| {
            match e {
                EntryType::Dir => entry.file_type().is_dir(),
                EntryType::File => entry.file_type().is_file(),
                EntryType::Link => entry.file_type().is_symlink()
            }
        })
    };

    let name_filter = |entry: &DirEntry| {
        args.names.is_empty() || args.names.iter().any(|re| re.is_match(&entry.file_name().to_string_lossy()))
    };

    for path in &args.paths {
        // for entry in WalkDir::new(path) {
        //     match entry {
        //         Err(e) => eprintln!("{e}"),
        //         Ok(entry) => {
        //             if (args.entry_types.is_empty() || args.entry_types.iter().any(|e| {
        //                 match e {
        //                     EntryType::Dir => entry.file_type().is_dir(),
        //                     EntryType::File => entry.file_type().is_file(),
        //                     EntryType::Link => entry.file_type().is_symlink()
        //                 }
        //             })) && (
        //                 args.names.is_empty() || args.names.iter().any(|re| re.is_match(&entry.file_name().to_string_lossy()))
        //             ) {
        //                 println!("{}", entry.path().display());
        //             }
        //         },
        //     }
        // }

        let entries = WalkDir::new(path)
            .into_iter()
            .filter_map(|entry| {
                match entry {
                    Err(e) => {
                        eprintln!("{e}");
                        None
                    },
                    Ok(e) => Some(e)
                }
            })
            .filter(type_filter)
            .filter(name_filter)
            .map(|entry| entry.path().display().to_string())
            .collect::<Vec<_>>();
    
        println!("{}", entries.join("\n"));
    }
    Ok(())
}

fn _get_args() -> Args {
    let matches = Command::new("findr")
        .version("0.1.0")
        .author("Kevin Chege <chege.kimaru@gmail.com>")
        .about("Rust version of `find`")
        .arg(
            Arg::new("paths")
                .value_name("PATH")
                .help("Search paths")
                .default_value(".")
                .num_args(0..),
        )
        .arg(
            Arg::new("names")
                .value_name("NAME")
                .short('n')
                .long("name")
                .help("Name")
                .value_parser(Regex::new)
                .action(ArgAction::Append)
                .num_args(0..),
        )
        .arg(
            Arg::new("types")
                .value_name("TYPE")
                .short('t')
                .long("type")
                .help("Entry type")
                .value_parser(clap::value_parser!(EntryType))
                .action(ArgAction::Append)
                .num_args(0..),
        )
        .get_matches();
    Args {
        paths: matches.get_many("paths").unwrap().cloned().collect(),
        names: matches
            .get_many("names")
            .unwrap_or_default()
            .cloned()
            .collect(),
        entry_types: matches
            .get_many("types")
            .unwrap_or_default()
            .cloned()
            .collect(),
    }
}

// Regex
// File glob - *.csv regex - .*\.csv or .*[.]csv "." in regex means any character, so ".*" is zero or more characters
// ^ - match start $ - match end