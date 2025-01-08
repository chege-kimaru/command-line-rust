use std::{
    ffi::OsStr, fs::{self, File}, io::{BufRead, BufReader}, path::PathBuf
};

use anyhow::{anyhow, bail, Result};
use clap::{arg, command, Arg, ArgAction, Command, Parser};
use rand::{rngs::StdRng, seq::SliceRandom, RngCore, SeedableRng};
use regex::RegexBuilder;
use walkdir::WalkDir;

#[derive(Debug, Parser)]
#[command(about, author, version)]
/// Rust version of `fortune`
struct Args {
    /// Input files or subdirectories
    #[arg(value_name = "FILE", required(true))]
    sources: Vec<String>,

    /// Pattern
    #[arg(value_name = "PATTERN", short = 'm', long)]
    pattern: Option<String>,

    /// Case-insensitive pattern matching
    #[arg(short, long)]
    insensitive: bool,

    /// Random seed
    #[arg(value_name = "SEED", short, long, value_parser(clap::value_parser!(u64)))]
    seed: Option<u64>,
}

#[derive(Debug)]
struct Fortune {
    source: String,
    text: String,
}

fn main() {
    if let Err(e) = run(Args::parse()) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<()> {
    let pattern = args
        .pattern
        .map(|val: String| {
            RegexBuilder::new(val.as_str())
                .case_insensitive(args.insensitive)
                .build()
                .map_err(|_| anyhow!(r#"Invalid --pattern "{val}""#))
        })
        .transpose()?;

    let files = find_files(&args.sources)?;
    let fortunes = read_fortunes(&files)?;

    match pattern {
        Some(pattern) => {
            let mut prev_source = None;
            for fortune in fortunes
                .into_iter()
                .filter(|fortune| pattern.is_match(&fortune.text))
            {
                if prev_source.as_ref().map_or(true, |s| s != &fortune.source) {
                    eprintln!("({})\n%", fortune.source);
                    prev_source = Some(fortune.source.clone());
                }    
                println!("{}\n%", fortune.text);         
            }
        },
        _ => {
            println!(
                "{}",
                pick_fortune(&fortunes, args.seed)
                    .or_else(|| Some("No fortunes found".to_string()))
                    .unwrap()
            )
        }
    }
    Ok(())
}

// My solution
// fn run(args: Args) -> Result<()> {
//     let pattern = args
//         .pattern
//         .map(|val: String| {
//             RegexBuilder::new(val.as_str())
//                 .case_insensitive(args.insensitive)
//                 .build()
//                 .map_err(|_| anyhow!(r#"Invalid --pattern "{val}""#))
//         })
//         .transpose()?;
//     let files = find_files(&args.sources)?;
//     let fortunes = read_fortunes(&files)?;

//     match pattern {
//         Some(pattern) => {
//             let mut current_file: String = "".to_string();
//             for fortune in fortunes {
//                 if pattern.is_match(&fortune.text) {
//                     if &current_file != &fortune.source {
//                         eprintln!("({})\n%", PathBuf::from(&fortune.source).file_name().unwrap().to_string_lossy());
//                         current_file = fortune.source.clone();
//                     }
//                     println!("{}\n%", &fortune.text);
//                 }
//             }
//         },
//         _ => {
//             if let Some(fortune) = pick_fortune(&fortunes, args.seed) {
//                 println!("{}", fortune);
//             } else {
//                 println!("No fortunes found");
//             }
//         }
//     }

//     Ok(())
// }

fn pick_fortune(fortunes: &[Fortune], seed: Option<u64>) -> Option<String> {
   let mut rng: Box<dyn RngCore> = match seed {
        Some(val) => Box::new(StdRng::seed_from_u64(val)),
        _ => Box::new(rand::thread_rng()),
   };
   
   fortunes.choose(&mut rng).map(|f| f.text.to_string())
}

// My solution
// fn pick_fortune(fortunes: &[Fortune], seed: Option<u64>) -> Option<String> {
//     if let Some(s) = seed {
//         let mut rng = rand::rngs::StdRng::seed_from_u64(s);
//         if let Some(fortune) = fortunes.choose(&mut rng) {
//             return Some((&fortune.text).to_string())
//         }
//     } else {
//         let mut rng = rand::thread_rng();
//         if let Some(fortune) = fortunes.choose(&mut rng) {
//             return Some((&fortune.text).to_string())
//         }
//     }
//     None
// }

fn read_fortunes(paths: &[PathBuf]) -> Result<Vec<Fortune>> {
    let mut fortunes = vec![];
    let mut buffer = vec![];

    for path in paths {
        let basename = path.file_name().unwrap().to_string_lossy().into_owned(); // Convert Path::file_name from OsStr to String, using the lossy version in case this is not valid UTF-8. The result is a clone-on-write smart pointer, so use Cow::into_owned to clone the data if it is not already owned.
        let file = File::open(path).map_err(|e| anyhow!("{}: {e}", path.to_string_lossy()))?;

        for line in BufReader::new(file).lines().map_while(Result::ok) {
            if line == "%" {
                if !buffer.is_empty() {
                    fortunes.push(Fortune {
                        source: basename.clone(),
                        text: buffer.join("\n"),
                    });
                    buffer.clear();
                }
            } else {
                buffer.push(line.to_string());
            }
        }
    }

    Ok(fortunes)
}

// My solution
// fn read_fortunes(paths: &[PathBuf]) -> Result<Vec<Fortune>> {
//     let mut fortunes = Vec::new();

//     for path in paths {
//         let mut file = BufReader::new(File::open(path)?);
//         let mut buf = Vec::new();
//         loop {
//             let bytes_read = file.read_until(b'%', &mut buf)?;
//             if bytes_read == 0 {
//                 break;
//             }

//             let text = String::from_utf8_lossy(&buf)
//                 .trim_end_matches("%")
//                 .trim()
//                 .to_string();

//             if !text.is_empty() {
//                 fortunes.push(Fortune {
//                     source: path.display().to_string(),
//                     text,
//                 });
//             }

//             buf.clear();
//         }
//     }

//     Ok(fortunes)
// }

fn find_files(paths: &[String]) -> Result<Vec<PathBuf>> {
    let dat = OsStr::new("dat"); // OsStr is a Rust type for an operating system’s preferred representation of a string that might not be a valid UTF-8 string. The type OsStr is borrowed, and the owned version is OsString
    let mut files = vec![];

    for path in paths {
        match fs::metadata(path) {
            Err(e) => bail!("{path}: {}", e),
            Ok(_) => {
                files.extend( // Use Vec::extend to add the results from WalkDir to the results.
                    WalkDir::new(path) // Use walkdir::WalkDir to find all the entries from the starting path.
                            .into_iter()
                            .filter_map(Result::ok) // This will ignore any errors for unreadable files or directories, which is the behavior of the original program.
                            .filter(|e| e.file_type().is_file() && e.path().extension() != Some(dat))
                            .map(|e| e.path().into()) // The walkdir::DirEntry::path function returns a Path, so convert it into a PathBuf.

                );
            }
        }
    }

    files.sort();
    files.dedup();

    Ok(files)
}

// My solution
// fn find_files(paths: &[String]) -> Result<Vec<PathBuf>> {
//     let mut files = Vec::new();

//     // Ignore .dat and hidden files e.g. .gitkeep
//     let is_valid_file = |path: &PathBuf| {
//         !(path.display().to_string().ends_with(".dat")
//             || path.file_name().unwrap().to_string_lossy().starts_with("."))
//     };

//     for path_str in paths {
//         let path = PathBuf::from(path_str);

//         match fs::metadata(path_str) {
//             Ok(_) => {
//                 if path.is_file() {
//                     if is_valid_file(&path) {
//                         files.push(path);
//                     }
//                 } else if path.is_dir() {
//                     for entry in fs::read_dir(&path)? {
//                         let entry = entry?;
//                         let entry_path = entry.path();

//                         if entry_path.is_file() {
//                             if is_valid_file(&entry_path) {
//                                 files.push(entry_path);
//                             }
//                         } else if entry_path.is_dir() {
//                             files.extend(find_files(&[entry_path.display().to_string()])?);
//                         }
//                     }
//                 }
//             }
//             Err(e) => {
//                 bail!("{path_str}: {}", e)
//             }
//         }
//     }

//     files.sort();
//     files.dedup();

//     Ok(files)
// }

fn _get_args() -> Args {
    let matches = Command::new("fortuner")
        .version("0.1.0")
        .author("Ken Youens-Clark <kyclark@gmail.com>")
        .about("Rust version of `fortune`")
        .arg(
            Arg::new("sources")
                .value_name("FILE")
                .num_args(1..)
                .required(true)
                .help("Input files or directories"),
        )
        .arg(
            Arg::new("pattern")
                .value_name("PATTERN")
                .short('m')
                .long("pattern")
                .help("Pattern"),
        )
        .arg(
            Arg::new("insensitive")
                .short('i')
                .long("insensitive")
                .help("Case-insensitive pattern matching")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("seed")
                .value_name("SEED")
                .short('s')
                .long("seed")
                .value_parser(clap::value_parser!(u64))
                .help("Random seed"),
        )
        .get_matches();

    Args {
        sources: matches.get_many("sources").unwrap().cloned().collect(),
        seed: matches.get_one("seed").cloned(),
        pattern: matches.get_one("pattern").cloned(),
        insensitive: matches.get_flag("insensitive"),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{Fortune, pick_fortune, find_files, read_fortunes};

    #[test]
    fn test_find_files() {
        // Verify that the function finds a file known to exist
        let res = find_files(&["./tests/inputs/jokes".to_string()]);
        assert!(res.is_ok());

        let files = res.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(
            files.get(0).unwrap().to_string_lossy(),
            "./tests/inputs/jokes"
        );

        // Fails to find a bad file
        let res = find_files(&["/path/does/not/exist".to_string()]);
        assert!(res.is_err());

        // Finds all the input files, excludes ".dat"
        let res = find_files(&["./tests/inputs".to_string()]);
        assert!(res.is_ok());

        // Check number and order of files
        let files = res.unwrap();
        assert_eq!(files.len(), 5); // Use 4 if you are ignoring hidden files eg .gitkeep in tests/inputs/empty
        let first = files.get(0).unwrap().display().to_string();
        assert!(first.contains("ascii-art"));
        let last = files.last().unwrap().display().to_string();
        assert!(last.contains("quotes"));

        // Test for multiple sources, path must be unique and sorted
        let res = find_files(&[
            "./tests/inputs/jokes".to_string(),
            "./tests/inputs/ascii-art".to_string(),
            "./tests/inputs/jokes".to_string(),
        ]);
        assert!(res.is_ok());
        let files = res.unwrap();
        assert_eq!(files.len(), 2);
        if let Some(filename) = files.first().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "ascii-art".to_string());
        }
        if let Some(filename) = files.last().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "jokes".to_string())
        }
    }

    #[test]
    fn test_read_fortunes() {
        // One input file
        let res = read_fortunes(&[PathBuf::from("./tests/inputs/jokes")]);
        assert!(res.is_ok());

        if let Ok(fortunes) = res {
            // Correct number and sorting
            assert_eq!(fortunes.len(), 6);
            assert_eq!(
                fortunes.first().unwrap().text,
                "Q. What do you call a head of lettuce in a shirt and tie?\n\
                A. Collared greens."
            );
            assert_eq!(
                fortunes.last().unwrap().text,
                "Q: What do you call a deer wearing an eye patch?\n\
                A: A bad idea (bad-eye deer)."
            );
        }

        // Multiple input files
        let res = read_fortunes(&[
            PathBuf::from("./tests/inputs/jokes"),
            PathBuf::from("./tests/inputs/quotes"),
        ]);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 11);
    }

    #[test]
    fn test_pick_fortune() {
        // Create a slice of fortunes
        let fortunes = &[
            Fortune {
                source: "fortunes".to_string(),
                text: "You cannot achieve the impossible without \
                      attempting the absurd."
                    .to_string(),
            },
            Fortune {
                source: "fortunes".to_string(),
                text: "Assumption is the mother of all screw-ups."
                    .to_string(),
            },
            Fortune {
                source: "fortunes".to_string(),
                text: "Neckties strangle clear thinking.".to_string(),
            },
        ];
        assert_eq!(
            pick_fortune(fortunes, Some(1)).unwrap(),
            "Neckties strangle clear thinking.".to_string()
        );
    }
}

// Run mk-dat.sh which uses the program strfile to create index files for randomly selecting the text records. Once ran, the input files will have companion .dat files
// strfile reads a file containing groups of lines separated by a line containing a single percent '%' sign (or other specified delimiter character) and creates a data file which contains a header structure and a table of file offsets for each group of lines. This allows random access of the strings.

// fortune -m 'Mark Twain' tests/inputs/ 1>out 2>err - Direct STDOUT to out file and TSDERR to err file

// cargo add anyhow rand regex walkdir clap --features clap/derive
// cargo add --dev assert_cmd predicates pretty_assertions

// PRNG - pseudorandom number generator

// Just as String is an owned, modifiable version of &str, PathBuf is an owned, modifiable version of Path
// The type OsStr is borrowed, and the owned version is OsString