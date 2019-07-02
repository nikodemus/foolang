use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*;
use std::process::Command; // Run programs // Used for writing assertions

#[test]
fn stdout_print_no_flush() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("--eval").arg("System stdout print: 'hello world!'; newline; print: 'boing!'");
    cmd.assert().success().stdout(predicate::str::contains("hello world!\n"));
    Ok(())
}

#[test]
fn stdout_print_flush() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("--eval").arg("System stdout print: 'hello world!'; newline; print: 'boing!'; flush");
    cmd.assert().success().stdout(predicate::str::ends_with("hello world!\nboing!"));
    Ok(())
}

#[test]
fn stdin_readline() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("--eval").arg(
        "
            {
                |out in|
                in := System stdin,
                out := System stdout,
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

#[test]
fn repl() -> Result<(), Box<std::error::Error>> {
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
fn benchmarks() -> Result<(), Box<std::error::Error>> {
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
