use anyhow::Result;
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use pretty_assertions::assert_eq;

// #[test]
// fn dies_no_args() {
//     let mut cmd = Command::cargo_bin("echor").unwrap();
//     cmd.assert().failure().stderr(predicate::str::contains("Usage"));
// }

// cargo test dies - will run all tests with the word dies

// #[test]
// fn runs() {
//     let mut cmd = Command::cargo_bin("echor").unwrap();
//     cmd.arg("Hello").assert().success();
// }

#[test]
fn dies_no_args() -> Result<()> {
    let mut cmd = Command::cargo_bin("echor")?;
    cmd.assert().failure().stderr(predicate::str::contains("Usage"));
    Ok(())
}

// #[test]
// fn hello1() -> Result<()> {
//     let outfile = "tests/expected/hello1.txt";
//     let expected = fs::read_to_string(outfile)?;
//     let mut cmd = Command::cargo_bin("echor")?;
//     cmd.arg("Hello there").assert().success().stdout(expected);
//     Ok(())
// }

// #[test]
// fn hello2() -> Result<()> {
//     let outfile = "tests/expected/hello2.txt";
//     let expected = fs::read_to_string(outfile)?;
//     let mut cmd = Command::cargo_bin("echor")?;
//     cmd.args(vec!["Hello", "there"]).assert().success().stdout(expected);
//     Ok(())
// }

fn run(args: &[&str], expected_file: &str) -> Result<()> {
    let expected = fs::read_to_string(expected_file)?;
    let output = Command::cargo_bin("echor")?.args(args).output().expect("Fail");
    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    assert_eq!(stdout, expected);
    Ok(())
}

#[test]
fn hello1() -> Result<()> {
    run(&["Hello there"], "tests/expected/hello1.txt")
}

#[test]
fn hello2() -> Result<()>{
    run(&["Hello", "there"], "tests/expected/hello2.txt")
}

#[test]
fn hello1_no_newline() -> Result<()>{
    run(&["Hello  there", "-n"], "tests/expected/hello1.n.txt")
}

#[test]
fn hello2_no_newline() -> Result<()>{
    run(&["-n", "Hello", "there"], "tests/expected/hello2.n.txt")
}