use assert_cmd::Command;
use predicates;
use predicates::prelude::*;

type Test = Result<(), Box<dyn std::error::Error>>;

#[test]
fn hello() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/hello.foo");
    cmd.assert().success().stdout("Hello world!\n");
    Ok(())
}

#[test]
fn hello_x() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/hello_x.foo")
        .write_stdin("Joe User\nXXXXX")
        .assert()
        .success()
        .stdout("What is your name?\nHello Joe User!\n");
    Ok(())
}

#[test]
fn test_exit_zero() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/exit_zero.foo");
    cmd.assert().success().stdout("");
    Ok(())
}

#[test]
fn test_exit_42() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/exit_42.foo");
    cmd.assert().failure().code(42).stdout("");
    Ok(())
}

#[test]
fn test_test() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/test.foo");
    cmd.assert().success();
    Ok(())
}

#[test]
fn test_abort() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/abort.foo");
    cmd.assert().failure().stdout("");
    Ok(())
}

#[test]
fn test_print_no_flush() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/print_no_flush.foo");
    cmd.assert().success().stdout("");
    Ok(())
}

#[test]
fn test_print_flush() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/print_flush.foo");
    cmd.assert().success().stdout("Foo");
    Ok(())
}

#[test]
fn test_bad_class() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/bad_class.foo");
    cmd.assert().failure().code(1).stdout(predicates::str::contains(
        "ERROR: Not valid in value position
003    method baz
004      }
         ^ Not valid in value position
005 end

",
    ));
    Ok(())
}

#[test]
fn test_import_x_local() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/import_x_local.foo");
    cmd.assert().failure().code(123).stdout("");
    Ok(())
}

#[test]
fn test_import_x() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/import_x.foo");
    cmd.arg("--use");
    cmd.arg("foo/x.foo");
    cmd.assert().failure().code(123).stdout("");
    Ok(())
}

#[test]
fn test_import_x_identity() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/import_x_Identity.foo");
    cmd.arg("--use");
    cmd.arg("foo/x.foo");
    cmd.assert().failure().code(100).stdout("");
    Ok(())
}

#[test]
fn test_import_x_star() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/import_x_star.foo");
    cmd.arg("--use");
    cmd.arg("foo/x.foo");
    cmd.assert().failure().code(42).stdout("");
    Ok(())
}

#[test]
fn test_import_bar_y() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/import_bar_y.foo");
    cmd.arg("--use");
    cmd.arg("bar");
    cmd.assert().failure().code(111).stdout("");
    Ok(())
}

#[test]
fn test_prelude1() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/test_prelude.foo");
    cmd.assert().failure().code(2).stdout("");
    Ok(())
}

#[test]
fn test_prelude2() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/test_prelude.foo");
    cmd.arg("--prelude");
    cmd.arg("foo/empty.foo");
    cmd.assert()
        .failure()
        .code(1)
        .stdout(predicates::str::contains("ERROR: 1 does not understand: + [1]"));
    Ok(())
}

#[test]
fn test_array_let() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/array_let.foo");
    cmd.assert().failure().code(1).stdout(predicates::str::contains(
        "ERROR: Unbound variable: x
002    class method run: command in: system
003       system output println: [let x = 42. x + x, x]
                                                     ^ Unbound variable: x
004 end

",
    ));
    Ok(())
}

#[test]
fn test_array_let2() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/array_let2.foo");
    cmd.assert().failure().code(1).stdout(predicates::str::contains(
        "ERROR: Unbound variable: x
003       let a = [let x = 42. x, 123].
004       system output println: x
                                 ^ Unbound variable: x
005 end
",
    ));
    Ok(())
}

#[test]
fn test_repl() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/repl.foo")
        .write_stdin(
            r#"class Point { x y }
                  class method displayOn: stream
                     stream print: "<class Point>"
                  method + other
                     Point x: x + other x
                           y: y + other y
                  method displayOn: stream
                     stream print: "{x}@{y}"
               end
               { let p1 = Point x: 1 y: 2.
                 let p2 = Point x: 100 y: 200.
                 p1 + p2 } value
               system output println: "Hi!"
               let z = 1
               z = z + 41
               z
               [let inside = 42. inside + 1]
               inside
               [let inside2 = 42. inside2 + 2, inside2]
              "#,
        )
        .assert()
        .success()
        .stdout(predicates::str::contains(
            r#"Foolang 0.0.0
> <class Point>
> 101@202
> Hi!
#<Output stdout>
> 1
> 42
> 42
> [43]
> ERROR: Unbound variable: inside
001                inside
                   ^^^^^^ Unbound variable: inside

> ERROR: Unbound variable: inside2
001                [let inside2 = 42. inside2 + 2, inside2]
                                                   ^^^^^^^ Unbound variable: inside2
"#,
        ));
    Ok(())
}

#[test]
fn test_benchmarks() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/benchmarks.foo");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("EmptyLoop:"))
        .stdout(predicate::str::contains("Factorial:"))
        .stdout(predicate::str::contains("SumFloats:"))
        .stdout(predicate::str::contains("Ackermann:"))
        .stdout(predicate::str::contains("Fibonacci:"));
    Ok(())
}

#[test]
#[ignore]
fn test_flying() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/flying.foo");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("TAKEOFF"))
        .stdout(predicate::str::contains("LANDING"))
        .stdout(predicate::str::contains("TOUCHDOWN"));
    Ok(())
}
