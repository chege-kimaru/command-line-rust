use clap::{Arg, ArgAction, Command};

fn main() {
    // println!("{:?}", std::env::args());

    let matches = Command::new("echor")
        .version("0.1.0")
        .author("Kevin Chege <chege.kimaru@gmail.com>")
        .about("Rust version of Rust")
        .arg(
            Arg::new("text")
                .value_name("TEXT")
                .help("Input text")
                .required(true)
                .num_args(1..),
        )
        .arg(
            Arg::new("omit_newline")
                .short('n')
                .action(ArgAction::SetTrue)
                .help("Do not print newline"),
        )
        .get_matches();

    // println!("{:?}", matches); // :? debug format. 
    // println!("{:#?}", matches); // :#? pretty debug format

    let vec: Vec<String> = matches.get_many("text").unwrap().cloned().collect();

    let omit_new_line = matches.get_flag("omit_newline");

    // let ending = if omit_new_line { "" } else {"\n"};
    // print!("{}{}", vec.join(" "), ending);
    print!("{}{}", vec.join(" "), if omit_new_line { "" } else {"\n"});
}

// echo Hello
// echo "Hello    World"
// man echo
// echo -n "Hello"
// echo Hello > hello
// echo -n Hello > hello-n
// diff hello hello-n

// cargo run -n Hello World - Fails as cargo thinks -n is its own argument, not the programs
// cargo run -- -n Hello World - -- Separates cargo arguments from program arguments

// du -shc . - Disk usage command

// cargo run --quiet

// cargo run 1>out 2>err redirect STDOUT to out and STDERR to err file
