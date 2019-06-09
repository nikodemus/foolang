use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*;
use std::process::Command; // Run programs // Used for writing assertions

#[test]
fn stdout_print_no_flush() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("--eval")
        .arg("Output stdout print: 'hello world!'; newline; print: 'boing!'");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("hello world!\n"));
    Ok(())
}

#[test]
fn stdout_print_flush() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("--eval")
        .arg("Output stdout print: 'hello world!'; newline; print: 'boing!'; flush");
    cmd.assert()
        .success()
        .stdout(predicate::str::ends_with("hello world!\nboing!"));
    Ok(())
}
