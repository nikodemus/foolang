use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*;
use std::process::Command; // Run programs // Used for writing assertions

#[test]
fn stdout_print_no_flush() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("--eval")
        .arg("System stdout print: 'hello world!'; newline; print: 'boing!'");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("hello world!\n"));
    Ok(())
}

#[test]
fn stdout_print_flush() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("--eval")
        .arg("System stdout print: 'hello world!'; newline; print: 'boing!'; flush");
    cmd.assert()
        .success()
        .stdout(predicate::str::ends_with("hello world!\nboing!"));
    Ok(())
}

#[test]
fn stdin_readline() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("--eval").arg(
        "
            {
                |out in|
                in := System stdin.
                out := System stdout.
                out print: ('1: ' append: (in readline)); newline;
                    print: ('2: ' append: (in readline)); newline
            } value
        ",
    );
    cmd.with_stdin()
        .buffer("this is line 1\nthis is line 2\nthis is tailing stuff")
        .assert()
        .success()
        .stdout("1: this is line 1\n2: this is line 2\n");
    Ok(())
}
