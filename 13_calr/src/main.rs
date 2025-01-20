use ansi_term::Style;
use anyhow::{bail, Result};
use chrono::{Datelike, Duration, Local, Month, NaiveDate};
use clap::Parser;
use itertools::izip;

const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const LINE_WIDTH: usize = 22;

#[derive(Debug, Parser)]
#[command(author, about, version)]
/// Rust version of `cal`
struct Args {
    /// Year (1-9999)
    #[arg(value_name = "YEAR", value_parser = clap::value_parser!(i32).range(1..=9999))]
    year: Option<i32>,

    /// Month name or number (1-12)
    #[arg(value_name = "MONTH", short)]
    month: Option<String>,

    /// SHow whole current year
    #[arg(short = 'y', long = "year", conflicts_with_all = ["month", "year"])]
    show_current_year: bool,
}

// fn get_args() -> Args {
//     let matches = Command::new("calr")
//         .version("0.1.0")
//         .author("Ken Youens-Clark <kyclark@gmail.com>")
//         .about("Rust version of `cal`")
//         .arg(
//             Arg::new("year")
//                 .value_name("YEAR")
//                 .value_parser(clap::value_parser!(i32).range(1..=9999))
//                 .help("Year (1-9999)"),
//         )
//         .arg(
//             Arg::new("month")
//                 .value_name("MONTH")
//                 .short('m')
//                 .help("Month name or number (1-12)"),
//         )
//         .arg(
//             Arg::new("show_current_year")
//                 .value_name("SHOW_YEAR")
//                 .short('y')
//                 .long("year")
//                 .help("Show whole current year")
//                 .conflicts_with_all(["month", "year"])
//                 .action(ArgAction::SetTrue),
//         )
//         .get_matches();

//     Args {
//         year: matches.get_one("year").cloned(),
//         month: matches.get_one("month").cloned(),
//         show_current_year: matches.get_flag("show_current_year"),
//     }
// }

fn main() {
    if let Err(e) = run(Args::parse()) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<()> {
    let today = Local::now().date_naive();
    let mut month = args.month.map(parse_month).transpose()?;
    let mut year = args.year;

    if args.show_current_year {
        month = None;
        year = Some(today.year());
    } else if month.is_none() && year.is_none() {
        month = Some(today.month());
        year = Some(today.year());
    }
    let year = year.unwrap_or(today.year());

    match month {
        Some(month) => {
            let lines = format_month(year, month, true, today);
            println!("{}", lines.join("\n")); 
        },
        _ => {
            // My solution
            // println!("{year:>32}");
            // let mut month_num = 1;
            // loop {
            //     let month1 = format_month(year, month_num, false, today);
            //     let month2 = format_month(year, month_num + 1, false, today);
            //     let month3 = format_month(year, month_num + 2, false, today);

            //     let zipped: Vec<_> = month1
            //         .iter()
            //         .zip(month2)
            //         .zip(month3)
            //         .map(|((a, b), c)| (a, b, c))
            //         .collect();

            //     for line in zipped {
            //         println!("{}{}{}", line.0, line.1, line.2);
            //     }

            //     month_num += 3;

            //     if month_num < 12 {
            //         println!("");
            //     } else {
            //         break;
            //     }
            // }
            println!("{year:>32}");

            let months: Vec<_> = (1..=12)
                .map(|month| format_month(year, month, false, today))
                .collect();

            for (i, chunk) in months.chunks(3).enumerate() {
                if let [m1, m2, m3] = chunk {
                    for lines in izip!(m1, m2, m3) { // Use itertools::izip to create an iterator that combines the lines from the three months.
                        println!("{}{}{}", lines.0, lines.1, lines.2);
                    }
                    if i < 3 { // If not on the last set of months, print a newline to separate the groupings.
                        println!();
                    }
                }
            }
        }
    }

    Ok(())
}

fn parse_month(month: String) -> Result<u32> {
    match month.parse() {
        Ok(num) => {
            if (1..=12).contains(&num) {
                Ok(num)
            } else {
                bail!(r#"month "{num}" not in the range 1 through 12"#)
            }
        }
        _ => {
            let lower = &month.to_lowercase();
            let matches: Vec<_> = MONTH_NAMES
                .iter()
                .enumerate()
                .filter_map(|(i, name)| {
                    if name.to_lowercase().starts_with(lower) {
                        Some(i + 1)
                    } else {
                        None
                    }
                })
                .collect();

            if matches.len() == 1 {
                Ok(matches[0] as u32)
            } else {
                bail!(r#"Invalid month "{month}""#)
            }
        }
    }
}

// My solution
// Tis will fail one condition not in the tests: The string Ju has 2 matches. It is not enough to disambiguate June and July
// fn parse_month(month: String) -> Result<u32> {
//     match month.parse::<u32>() {
//         Ok(num) => {
//             if num >= 1 && num <= 12 {
//                 Ok(num)
//             } else {
//                 bail!(r#"month "{num}" not in the range 1 through 12"#)
//             }
//         },
//         _ => {
//             let index = MONTH_NAMES.iter().find_position(|m| m.to_lowercase().starts_with(&month.to_lowercase()));
//             if let Some(index) = index {
//                 Ok((index.0 as u32 + 1))
//             } else {
//                 bail!(r#"Invalid month "{}""#, &month);
//             }
//         }
//     }
// }

// My solution
// fn format_month(year: i32, month: u32, print_year: bool, today: NaiveDate) -> Vec<String> {
//     let mut rows = vec![];

//     let month_name = MONTH_NAMES.get((month - 1) as usize).unwrap().to_string();
//     let header = if print_year {
//         format!("{month_name} {year}")
//     } else {
//         format!("{month_name}")
//     };

//     rows.push(format!("{:^20}  ", header));
//     rows.push("Su Mo Tu We Th Fr Sa  ".to_string());

//     let mut date = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
//     let mut week_day = date.weekday().number_from_sunday();

//     let mut row = format!("{} 1 ", "   ".repeat((week_day - 1) as usize));
//     let mut date_num = 1; // Date of the month
//     let mut rows_count = 1; // Loop from row 1 to 6

//     loop {
//         // dbg!(rows_count, week_day);

//         if rows_count > 6 {
//             break;
//         }

//         if week_day >= 7 {
//             rows.push(format!("{row} "));
//             week_day = 0;
//             row.clear();
//             rows_count += 1;
//         }

//         date_num += 1;
//         week_day += 1;
//         date += Duration::days(1);

//         let curr_date = if date > last_day_in_month(year, month) {
//             format!("{:>2} ", "")
//         } else {
//             if date == today { Style::new().reverse().paint(format!("{date_num:>2}")).to_string() + " " } else { format!("{date_num:>2} ") }
//         };
//         row += &curr_date;
//     }

//     rows
// }

fn format_month(year: i32, month: u32, print_year: bool, today: NaiveDate) -> Vec<String> {
    let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    // Initialize a mutable Vec<String> with a buffer of the days from Sunday until the start of the month.
    let mut days: Vec<String> = (1..first.weekday().number_from_sunday()) // Note, no equal in range
        .map(|_| "  ".to_string()) // 2 spaces
        .collect();

    let is_today = |day: u32| {
        year == today.year() && month == today.month() && day == today.day()
    };

    let last = last_day_in_month(year, month);
    days.extend((first.day()..=last.day()).map(|num| {
        let fmt = format!("{num:>2}");
        if is_today(num) {
            Style::new().reverse().paint(fmt).to_string()
        } else {
            fmt
        }
    }));

    let month_name = MONTH_NAMES[month as usize - 1];
    let mut lines = Vec::with_capacity(8);
    lines.push(format!(
        "{:^20}  ", // 2 trailing spaces
        if print_year {
            format!("{month_name} {year}")
        } else {
            month_name.to_string()
        }
    ));

    lines.push("Su Mo Tu We Th Fr Sa  ".to_string()); // 2 trailing spaces

    for week in days.chunks(7) {
        lines.push(format!(
            "{:width$}  ", // 2 trailing spaces // width$ is a named argument
            week.join(" "),
            width = LINE_WIDTH - 2
        ));
    }

    while lines.len() < 8 {
        lines.push(" ".repeat(LINE_WIDTH));
    }

    lines
}

// My solution
// fn last_day_in_month(year: i32, month: u32) -> NaiveDate {
//     let (next_year, next_month) = if month == 12 {
//         (year + 1, 1)
//     } else {
//         (year, month + 1)
//     };
//     let first_day_next_month = NaiveDate::from_ymd_opt(next_year, next_month, 1).unwrap();
//     first_day_next_month - Duration::days(1)
// }

fn last_day_in_month(year: i32, month: u32) -> NaiveDate {
    // The first day of the next month...
    // If this is December, then advance the year by one and set the month to January.
    let (y, m) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };

    NaiveDate::from_ymd_opt(y, m, 1)
        .unwrap()
        .pred_opt() // get the previous calendar date
        .unwrap()
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::{format_month, last_day_in_month, parse_month};

    #[test]
    fn test_parse_month() {
        let res = parse_month("1".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1u32);

        let res = parse_month("12".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 12u32);

        let res = parse_month("jan".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1u32);

        let res = parse_month("0".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"month "0" not in the range 1 through 12"#
        );

        let res = parse_month("13".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"month "13" not in the range 1 through 12"#
        );

        let res = parse_month("foo".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"Invalid month "foo""#);
    }

    #[test]
    fn test_format_month() {
        let today = NaiveDate::from_ymd_opt(0, 1, 1).unwrap();
        let leap_february = vec![
            "   February 2020      ",
            "Su Mo Tu We Th Fr Sa  ",
            "                   1  ",
            " 2  3  4  5  6  7  8  ",
            " 9 10 11 12 13 14 15  ",
            "16 17 18 19 20 21 22  ",
            "23 24 25 26 27 28 29  ",
            "                      ",
        ];
        assert_eq!(format_month(2020, 2, true, today), leap_february);

        let may = vec![
            "        May           ",
            "Su Mo Tu We Th Fr Sa  ",
            "                1  2  ",
            " 3  4  5  6  7  8  9  ",
            "10 11 12 13 14 15 16  ",
            "17 18 19 20 21 22 23  ",
            "24 25 26 27 28 29 30  ",
            "31                    ",
        ];
        assert_eq!(format_month(2020, 5, false, today), may);

        let april_hl = vec![
            "     April 2021       ",
            "Su Mo Tu We Th Fr Sa  ",
            "             1  2  3  ",
            " 4  5  6 \u{1b}[7m 7\u{1b}[0m  8  9 10  ",
            "11 12 13 14 15 16 17  ",
            "18 19 20 21 22 23 24  ",
            "25 26 27 28 29 30     ",
            "                      ",
        ];
        let today = NaiveDate::from_ymd_opt(2021, 4, 7).unwrap();
        assert_eq!(format_month(2021, 4, true, today), april_hl);
    }

    #[test]
    fn test_last_day_in_month() {
        assert_eq!(
            last_day_in_month(2020, 1),
            NaiveDate::from_ymd_opt(2020, 1, 31).unwrap()
        );
        assert_eq!(
            last_day_in_month(2020, 2),
            NaiveDate::from_ymd_opt(2020, 2, 29).unwrap()
        );
        assert_eq!(
            last_day_in_month(2020, 4),
            NaiveDate::from_ymd_opt(2020, 4, 30).unwrap()
        );
    }
}
