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
fn test_exit_zero() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/exit_zero.foo");
    cmd.assert().success().stdout("");
    Ok(())
}

#[test]
fn test_exit_42() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/exit_42.foo");
    cmd.assert().failure().code(42).stdout("");
    Ok(())
}

#[test]
fn test_abort() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/abort.foo");
    cmd.assert().failure().stdout("");
    Ok(())
}

#[test]
fn test_print_no_flush() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/print_no_flush.foo");
    cmd.assert().success().stdout("");
    Ok(())
}

#[test]
fn test_print_flush() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/print_flush.foo");
    cmd.assert().success().stdout("Foo");
    Ok(())
}

#[test]
fn test_bad_class() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/bad_class.foo");
    cmd.assert().success().stdout(
        "ERROR: Not valid in value position
002    method baz
003      }
         ^ Not valid in value position
004 end

",
    );
    Ok(())
}

#[test]
fn repl() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/repl.foo");
    cmd.with_stdin()
        .buffer(
            r#"class Point { x, y }
                  method + other
                     Point x: x + other x
                           y: y + other y
                  method toString
                     "{x}@{y}"
               end
               { let p1 = Point x: 1 y: 2,
                 let p2 = Point x: 100 y: 200,
                 p1 + p2 } value
               system output println: "Hi!"
              "#,
        )
        .assert()
        .success()
        .stdout(
            r#"Foolang 0.2.0
> #<class Point>
> 101@202
> Hi!
#<Output stdout>
> "#,
        );
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
