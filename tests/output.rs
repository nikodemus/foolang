use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*;
use std::process::Command; // Run programs // Used for writing assertions

type Test = Result<(), Box<std::error::Error>>;

#[test]
fn hello() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/hello.foo");
    cmd.assert().success().stdout("Hello world!\n");
    Ok(())
}

#[test]
fn hello_x() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/hello_x.foo");
    cmd.with_stdin()
        .buffer("Joe User\nXXXXX")
        .assert()
        .success()
        .stdout("What is your name?\nHello Joe User!\n");
    Ok(())
}

#[test]
fn old_stdout_print_no_flush() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("--eval").arg("System stdout print: 'hello world!'; newline; print: 'boing!'");
    cmd.assert().success().stdout(predicate::str::contains("hello world!\n"));
    Ok(())
}

#[test]
fn old_stdout_print_flush() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("--eval").arg("System stdout print: 'hello world!'; newline; print: 'boing!'; flush");
    cmd.assert().success().stdout(predicate::str::ends_with("hello world!\nboing!"));
    Ok(())
}

#[test]
fn old_repl() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("--load").arg("foo/repl.foo").arg("--eval").arg("REPL run");
    cmd.with_stdin()
        // The repl is currently a tad aggressive about newlines...
        // Should sniff @ at the beginning and require an empty line
        // after, plus throw out the input after an empty line, maybe.
        .buffer(
            "
            @class Foo []
            @class-method Foo a:a b:b ^(a + b) * 2
            Foo a: 1 b: 20
        ",
        )
        .assert()
        .success()
        .stdout("Foolang 0.1.0\n> #Foo\n> #a:b:\n> 42\n> ");
    Ok(())
}

#[test]
#[ignore] // takes a bit long, and will take longer as more are added
fn old_benchmarks() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("--load").arg("foo/benchmarks.foo").arg("--eval").arg("Benchmarks all run");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("EmptyLoop:"))
        .stdout(predicate::str::contains("Factorial:"))
        .stdout(predicate::str::contains("SumFloats:"))
        .stdout(predicate::str::contains("Ackermann:"))
        .stdout(predicate::str::contains("Fibonacci:"));
    Ok(())
}
