// Note, this code should be run on Unix environment. If on windows, use WSL
// Run set-test-perms.sh before running the tests


mod owner;

use std::{fs, os::unix::fs::MetadataExt, path::PathBuf};

use anyhow::Result;
use chrono::{DateTime, Local};
use clap::{Arg, ArgAction, Command, Parser};
use owner::Owner;
use tabular::{Row, Table};
use users::{get_group_by_gid, get_user_by_uid};

#[derive(Debug, Parser)]
#[command(author, version, about)]
/// Rust version of `ls`
struct Args {
    /// Files and/or directories
    #[arg(value_name = "PATH", default_value = ".")]
    paths: Vec<String>,

    /// Long listing
    #[arg(long, short)]
    long: bool,

    /// Show all files
    #[arg(short = 'a', long = "all")]
    show_hidden: bool,
}

fn main() {
    if let Err(e) = run(Args::parse()) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<()> {
    let paths = find_files(&args.paths, args.show_hidden)?;

    if args.long {
        println!("{}", format_output(&paths)?);
    } else {
        // My solution
        // println!(
        //     "{}",
        //     paths
        //         .iter()
        //         .map(|p| p.display().to_string())
        //         .collect::<Vec<_>>()
        //         .join("\n")
        // );

        for path in paths {
            println!("{}", path.display());
        }
    }

    // for n in 0..=7 {
    //     println!("{n} = {n:03b}"); // Print the value n as is and in binary format to three places using leading zeros.
    // }

    Ok(())
}

fn format_output(paths: &[PathBuf]) -> Result<String> {
    //               1   2    3    4    5    6    7    8
    let fmt = "{:<}{:<} {:>} {:<} {:<} {:>} {:<} {:<}";
    let mut table = Table::new(fmt);

    for path in paths {
        let metadata = path.metadata()?;

        let uid = metadata.uid();
        let user = get_user_by_uid(uid)
            .map(|u| u.name().to_string_lossy().into_owned())
            .unwrap_or_else(|| uid.to_string());

        let gid = metadata.gid();
        let group = get_group_by_gid(gid)
            .map(|g| g.name().to_string_lossy().into_owned())
            .unwrap_or_else(|| gid.to_string());

        let file_type = if path.is_dir() { "d" } else { "-" };
        let perms = format_mode(metadata.mode());
        let modified: DateTime<Local> = DateTime::from(metadata.modified()?);

        table.add_row(
            Row::new()
                .with_cell(file_type) // 1 "d" or "-"
                .with_cell(perms) // 2 permissions
                .with_cell(metadata.nlink()) // 3 number of links
                .with_cell(user) // 4 user name
                .with_cell(group) // 5 group name
                .with_cell(metadata.len()) // 6 size
                .with_cell(modified.format("%b %d %y %H:%M")) // 7 modification
                .with_cell(path.display()), // 8 path
        );
    }

    Ok(format!("{table}"))
}

// My solution
/// Given a file mode in octal format like 0o751,
/// return a string like "rwxr-x--x"
// fn format_mode(mode: u32) -> String {
//     format!(
//         "{}{}{}{}{}{}{}{}{}",
//         if mode & 0o400 > 0 { "r" } else { "-" },
//         if mode & 0o200 > 0 { "w" } else { "-" },
//         if mode & 0o100 > 0 { "x" } else { "-" },
//         if mode & 0o040 > 0 { "r" } else { "-" },
//         if mode & 0o020 > 0 { "w" } else { "-" },
//         if mode & 0o010 > 0 { "x" } else { "-" },
//         if mode & 0o004 > 0 { "r" } else { "-" },
//         if mode & 0o002 > 0 { "w" } else { "-" },
//         if mode & 0o001 > 0 { "x" } else { "-" },
//     )
//     // This is operation is known as masking. if you & the values 0o700 (111) and 0o200 (010), the write bits in position 2 are both set and so the result is 0o200 (010). The other bits can’t be set because the zeros in 0o200 will mask or hide those values, hence the term masking for this operation. If you & the values 0o400 (100) and 0o200 (010), the result is 0 because none of the three positions contains a 1 in both operands
// }

/// Given a file mode in octal format like 0o751,
/// return a string like "rwxr-x--x"
fn format_mode(mode: u32) -> String {
    format!(
        "{}{}{}",
        mk_triple(mode, Owner::User),
        mk_triple(mode, Owner::Group),
        mk_triple(mode, Owner::Other),
    )
}

/// Given an octal number like 0o500 and an [`Owner`],
/// return a string like "r-x"
fn mk_triple(mode: u32, owner: Owner) -> String {
    let [read, write, execute] = owner.masks();
    format!(
        "{}{}{}",
        if mode & read == 0 { "-" } else { "r" },
        if mode & write == 0 { "-" } else { "w" },
        if mode & execute == 0 { "-" } else { "x" },
    )
}

// My solution
// fn find_files(paths: &[String], show_hidden: bool) -> Result<Vec<PathBuf>> {
//     let mut files = vec![];

//     for path in paths {
//         match fs::metadata(path) {
//             Err(e) => eprintln!("{path}: {e}"),
//             Ok(metadata) => {
//                 if metadata.is_dir() {
//                     files.extend(
//                         fs::read_dir(path)?
//                         .map_while(Result::ok)
//                         .map(|entry| entry.path())
//                         .filter(|path|
//                             show_hidden || (!show_hidden && !path.file_name().unwrap().to_string_lossy().starts_with("."))
//                         )
//                     )
//                 } else {
//                     files.push(PathBuf::from(path));
//                 }
//             }
//         }
//     }

//     Ok(files)
// }

fn find_files(paths: &[String], show_hidden: bool) -> Result<Vec<PathBuf>> {
    let mut results = vec![];
    for name in paths {
        match fs::metadata(name) {
            Err(e) => eprintln!("{name}: {e}"),
            Ok(meta) => {
                if meta.is_dir() {
                    for entry in fs::read_dir(name)? {
                        let entry = entry?;
                        let path = entry.path();
                        let is_hidden = path.file_name().map_or(false, |file_name| {
                            file_name.to_string_lossy().starts_with('.')
                        });
                        if !is_hidden || show_hidden {
                            results.push(entry.path());
                        }
                    }
                } else {
                    results.push(PathBuf::from(name));
                }
            }
        }
    }
    Ok(results)
}

fn _get_args() -> Args {
    let matches = Command::new("lsr")
        .version("0.1.0")
        .author("Ken Youens-Clark <kyclark@gmail.com>")
        .about("Rust version of `ls`")
        .arg(
            Arg::new("paths")
                .value_name("PATH")
                .help("Files and/or directories")
                .default_value(".")
                .num_args(0..),
        )
        .arg(
            Arg::new("long")
                .action(ArgAction::SetTrue)
                .help("Long listing")
                .short('l')
                .long("long"),
        )
        .arg(
            Arg::new("all")
                .action(ArgAction::SetTrue)
                .help("Show all files")
                .short('a')
                .long("all"),
        )
        .get_matches();

    Args {
        paths: matches.get_many("paths").unwrap().cloned().collect(),
        long: matches.get_flag("long"),
        show_hidden: matches.get_flag("all"),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use pretty_assertions::assert_eq;

    use crate::format_output;

    use super::{find_files, format_mode, mk_triple, Owner};

    #[test]
    fn test_find_files() {
        // Find all nonhidden entries in a directory
        let res = find_files(&["tests/inputs".to_string()], false);
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            [
                "tests/inputs/bustle.txt",
                "tests/inputs/dir",
                "tests/inputs/empty.txt",
                "tests/inputs/fox.txt",
            ]
        );

        // Find all entries in a directory
        let res = find_files(&["tests/inputs".to_string()], true);
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            [
                "tests/inputs/.hidden",
                "tests/inputs/bustle.txt",
                "tests/inputs/dir",
                "tests/inputs/empty.txt",
                "tests/inputs/fox.txt",
            ]
        );

        // Any existing file should be found even if hidden
        let res = find_files(&["tests/inputs/.hidden".to_string()], false);
        assert!(res.is_ok());
        let filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        assert_eq!(filenames, ["tests/inputs/.hidden"]);

        // Test multiple path arguments
        let res = find_files(
            &[
                "tests/inputs/bustle.txt".to_string(),
                "tests/inputs/dir".to_string(),
            ],
            false,
        );
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            ["tests/inputs/bustle.txt", "tests/inputs/dir/spiders.txt"]
        );
    }

    #[test]
    fn test_find_files_hidden() {
        let res = find_files(&["tests/inputs".to_string()], true);
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            [
                "tests/inputs/.hidden",
                "tests/inputs/bustle.txt",
                "tests/inputs/dir",
                "tests/inputs/empty.txt",
                "tests/inputs/fox.txt",
            ]
        );
    }

    #[test]
    fn test_format_mode() {
        assert_eq!(format_mode(0o755), "rwxr-xr-x");
        assert_eq!(format_mode(0o421), "r---w---x");
    }

    fn long_match(
        line: &str,
        expected_name: &str,
        expected_perms: &str,
        expected_size: Option<&str>,
    ) {
        let parts: Vec<_> = line.split_whitespace().collect();
        assert!(parts.len() > 0 && parts.len() <= 10); // 10 intead of 7 because the mofified date has white spaces

        let perms = parts.get(0).unwrap();
        assert_eq!(perms, &expected_perms);

        if let Some(size) = expected_size {
            // Directory sizes are not tested, so this is an optional argument.
            let file_size = parts.get(4).unwrap();
            assert_eq!(file_size, &size)
        }

        let display_name = parts.last().unwrap();
        assert_eq!(display_name, &expected_name);
    }

    #[test]
    fn test_format_output_one() {
        let bustle_path = "tests/inputs/bustle.txt";
        let bustle = PathBuf::from(bustle_path);

        let res = format_output(&[bustle]);
        assert!(res.is_ok());

        let out = res.unwrap();
        let lines: Vec<&str> = out.split("\n").filter(|s| !s.is_empty()).collect();
        assert_eq!(lines.len(), 1);

        let line1 = lines.first().unwrap();
        long_match(&line1, bustle_path, "-rw-r--r--", Some("193"));
    }

    #[test]
    fn test_format_output_two() {
        let res = format_output(&[
            PathBuf::from("tests/inputs/dir"),
            PathBuf::from("tests/inputs/empty.txt"),
        ]);
        assert!(res.is_ok());

        let out = res.unwrap();
        let mut lines: Vec<&str> = out.split("\n").filter(|s| !s.is_empty()).collect();
        lines.sort();
        assert_eq!(lines.len(), 2);

        let empty_line = lines.remove(0);
        long_match(
            &empty_line,
            "tests/inputs/empty.txt",
            "-rw-r--r--",
            Some("0"),
        );

        let dir_line = lines.remove(0);
        long_match(&dir_line, "tests/inputs/dir", "drwxr-xr-x", None);
    }

    #[test]
    fn test_mk_triple() {
        assert_eq!(mk_triple(0o751, Owner::User), "rwx");
        assert_eq!(mk_triple(0o751, Owner::Group), "r-x");
        assert_eq!(mk_triple(0o751, Owner::Other), "--x");
        assert_eq!(mk_triple(0o600, Owner::Other), "---");
    }
}


// Execute cargo doc --open --document-private-items to have Cargo create documentation for your code