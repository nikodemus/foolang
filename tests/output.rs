use assert_cmd::Command;
use predicates;
use predicates::prelude::*;

type Test = Result<(), Box<dyn std::error::Error>>;

#[test]
fn example_hello() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/examples/hello.foo");
    cmd.assert().success().stdout("Hello world!\n");
    Ok(())
}

#[test]
fn example_hello_x() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/examples/hello_x.foo")
        .write_stdin("Joe User\nXXXXX")
        .assert()
        .success()
        .stdout("What is your name?\nHello Joe User!\n");
    Ok(())
}

#[test]
fn test_exit_zero() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/tests/test_exit_zero.foo");
    cmd.assert().success().stdout("");
    Ok(())
}

#[test]
fn test_exit_42() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/tests/test_exit_42.foo");
    cmd.assert().failure().code(42).stdout("");
    Ok(())
}

#[test]
fn test_test() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/tests/test.foo");
    cmd.arg("--use=foo/lib");
    cmd.assert().success();
    Ok(())
}

#[test]
fn test_abort() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/tests/test_abort.foo");
    cmd.assert().failure().stdout("");
    Ok(())
}

#[test]
fn test_print_no_flush() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/tests/test_print_no_flush.foo");
    cmd.assert().success().stdout("");
    Ok(())
}

#[test]
fn test_print_flush() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/tests/test_print_flush.foo");
    cmd.assert().success().stdout("Foo");
    Ok(())
}

#[test]
fn test_bad_class() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/tests/test_bad_class.foo");
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
fn test_interface_unimplemented() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/tests/test_interface_unimplemented.foo");
    cmd.assert()
        .failure()
        .code(1)
        .stdout(predicates::str::contains("ERROR: C#quux unimplemented, required by interface I"));
    Ok(())
}

#[test]
fn test_interface_bad_signature() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/tests/test_interface_bad_signature.foo");
    cmd.assert().failure().code(1).stdout(predicates::str::contains(
        "ERROR: C#quux is () -> Any, interface I specifies () -> Integer",
    ));
    Ok(())
}

#[test]
fn test_interface_ok() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/tests/test_interface_ok.foo");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("foo = C#foo"))
        .stdout(predicates::str::contains("bar = I#bar"))
        .stdout(predicates::str::contains("quux = 42"));
    Ok(())
}

#[test]
fn test_interface_typecheck() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/tests/test_interface_typecheck.foo");
    cmd.assert()
        .failure()
        .stdout(predicates::str::contains("YesI: True"))
        .stdout(predicates::str::contains("FATAL - ERROR: I expected, got: NotI"));
    Ok(())
}

#[test]
fn test_interface_inheritance() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/tests/test_interface_inheritance.foo");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("foo: 0 => I0 ok"))
        .stdout(predicates::str::contains("bar: 1 => I1 ok"))
        .stdout(predicates::str::contains("quux: 2 => I ok"));
    Ok(())
}

#[test]
fn test_import_x_local() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/tests/test_import_x_local.foo");
    cmd.assert().failure().code(123).stdout("");
    Ok(())
}

#[test]
fn test_import_x() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/tests/test_import_x.foo");
    cmd.arg("--use");
    cmd.arg("foo/tests/x.foo");
    cmd.assert().failure().code(123).stdout("");
    Ok(())
}

#[test]
fn test_import_x_no_use_no_local() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/tests/test_import_x.foo");
    cmd.assert()
        .failure()
        .code(1)
        .stdout(predicates::str::contains("FATAL - ERROR: Unknown module: x.foo"));
    Ok(())
}

#[test]
fn test_import_x_identity() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/tests/test_import_x_Identity.foo");
    cmd.arg("--use");
    cmd.arg("foo/tests/x.foo");
    cmd.assert().failure().code(100).stdout("");
    Ok(())
}

#[test]
fn test_import_x_star() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/tests/test_import_x_star.foo");
    cmd.arg("--use");
    cmd.arg("foo/tests/x.foo");
    cmd.assert().failure().code(42).stdout("");
    Ok(())
}

#[test]
fn test_import_bar_y() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/tests/test_import_bar_y.foo");
    cmd.arg("--use");
    cmd.arg("foo/tests/bar");
    cmd.assert().failure().code(111).stdout("");
    Ok(())
}

#[test]
fn test_prelude1() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/tests/test_prelude.foo");
    cmd.assert().failure().code(2).stdout("");
    Ok(())
}

#[test]
fn test_prelude2() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/tests/test_prelude.foo");
    cmd.arg("--prelude");
    cmd.arg("foo/tests/empty.foo");
    cmd.assert()
        .failure()
        .code(1)
        .stdout(predicates::str::contains("ERROR: 1 does not understand: + [1]"));
    Ok(())
}

#[test]
fn test_array_let1() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/tests/test_array_let1.foo");
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
    cmd.arg("foo/tests/test_array_let2.foo");
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
    cmd.arg("foo/tests/test_benchmarks.foo");
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
fn example_flying() -> Test {
    let mut cmd = Command::cargo_bin("foolang")?;
    cmd.arg("foo/examples/flying.foo");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("TAKEOFF"))
        .stdout(predicate::str::contains("LANDING"))
        .stdout(predicate::str::contains("TOUCHDOWN"));
    Ok(())
}
