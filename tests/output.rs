use assert_cmd::Command;
use predicates;
use predicates::prelude::*;
use serial_test::serial;

type Test = Result<(), Box<dyn std::error::Error>>;

#[test]
fn test_self_hosted_foolang() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/impl/test_foolang.foo");
    cmd.assert().success().stdout("All tests ok!\n");
    Ok(())
}

#[test]
fn test_self_hosted_prelude() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/impl/test_prelude.foo");
    cmd.assert().success().stdout("All tests ok!\n");
    Ok(())
}

#[test]
#[serial]
fn test_self_hosted_transpiler0() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/impl/test_transpile.foo");
    cmd.arg("--use=foo/lib");
    cmd.arg("--");
    cmd.arg("name");
    cmd.assert().success();
    Ok(())
}

#[test]
#[serial]
fn test_self_hosted_transpiler1() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/impl/test_transpile.foo");
    cmd.arg("--use=foo/lib");
    cmd.arg("--");
    cmd.arg("transpile1");
    cmd.assert().success();
    Ok(())
}

#[test]
#[serial]
fn test_self_hosted_transpiler2() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/impl/test_transpile.foo");
    cmd.arg("--use=foo/lib");
    cmd.arg("--");
    cmd.arg("transpile2");
    cmd.assert().success();
    Ok(())
}

#[test]
#[serial]
fn test_self_hosted_transpiler3() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/impl/test_transpile.foo");
    cmd.arg("--use=foo/lib");
    cmd.arg("--");
    cmd.arg("transpile3");
    cmd.assert().success();
    Ok(())
}

#[test]
#[serial]
fn test_self_hosted_transpiler4() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/impl/test_transpile.foo");
    cmd.arg("--use=foo/lib");
    cmd.arg("--");
    cmd.arg("transpile4");
    cmd.assert().success();
    Ok(())
}

#[test]
fn example_hello() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/examples/hello.foo");
    cmd.assert().success().stdout("Hello world!\n");
    Ok(())
}

#[test]
fn example_hello_x() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/examples/hello_x.foo")
        .write_stdin("Joe User\nXXXXX")
        .assert()
        .success()
        .stdout("What is your name?\nHello Joe User!\n");
    Ok(())
}

#[test]
fn test_exit_zero() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test_exit_zero.foo");
    cmd.assert().success().stdout("");
    Ok(())
}

#[test]
fn test_exit_42() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test_exit_42.foo");
    cmd.assert().failure().code(42).stdout("");
    Ok(())
}

#[test]
fn test_test() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test.foo");
    cmd.arg("--use=foo/lib");
    cmd.arg("--use=foo/lang");
    cmd.assert().success();
    Ok(())
}

#[test]
fn test_abort() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test_abort.foo");
    cmd.assert().failure().stdout("");
    Ok(())
}

#[test]
fn test_print_no_flush() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test_print_no_flush.foo");
    cmd.assert().success().stdout("");
    Ok(())
}

#[test]
fn test_print_flush() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test_print_flush.foo");
    cmd.assert().success().stdout("Foo");
    Ok(())
}

#[test]
fn test_class_comment1() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test_class_comment1.foo");
    cmd.assert().success().stdout("ok\n");
    Ok(())
}

#[test]
fn test_class_comment2() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test_class_comment2.foo");
    cmd.assert().success().stdout("ok\n");
    Ok(())
}

#[test]
fn test_class_comment3() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test_class_comment3.foo");
    cmd.assert().success().stdout("ok\n");
    Ok(())
}

#[test]
fn test_bad_class() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test_bad_class.foo");
    cmd.assert().failure().code(1).stdout(predicates::str::contains(
        "ERROR: Not valid in value position: }
003    method baz
004      }
         ^ Not valid in value position: }
005 end

",
    ));
    Ok(())
}

#[test]
fn test_define_let_leak() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test_define_let_leak.foo");
    cmd.assert().failure().code(1).stdout(predicates::str::contains(
        " ERROR: Unbound variable: y
006     direct method run: command in: system
007         y!
            ^ Unbound variable: y
008 end",
    ));
    Ok(())
}

#[test]
fn test_unbound_variable_location() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test_unbound_variable_location.foo");
    cmd.assert().failure().code(1).stdout(predicates::str::contains(
        "ERROR: Unbound variable: oops
002     direct method oops
003         oops!
            ^^^^ Unbound variable: oops
004 end",
    ));
    Ok(())
}

/**
 *  FIXME: type error source locations regressed when Object#typecheck: appeared
 *  on Foolang side.

#[test]
fn test_value_type_error_location() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test_value_type_error_location.foo");
    cmd.assert().failure().code(1).stdout(predicates::str::contains(
        "ERROR: String expected, got Integer: 14
007     direct method oops
008         14::String!
            ^^ String expected, got Integer: 14
009 end",
    ));
    Ok(())
}

#[test]
fn test_slot_type_error_location() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test_slot_type_error_location.foo");
    cmd.assert().failure().code(1).stdout(predicates::str::contains(
        "ERROR: String expected, got Integer: 123
014     method oops
015         slot = 123!
                   ^^^ String expected, got Integer: 123
016 end",
    ));
    Ok(())
}

#[test]
fn test_var_type_error_location() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test_var_type_error_location.foo");
    cmd.assert().failure().code(1).stdout(predicates::str::contains(
        "ERROR: String expected, got Integer: 12312
020         let x::String = \"OK\".
021         x = 12312!
                ^^^^^ String expected, got Integer: 12312
022 end",
    ));
    Ok(())
}

#[test]
fn test_var_init_type_error_location() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test_var_init_type_error_location.foo");
    cmd.assert().failure().code(1).stdout(predicates::str::contains(
        "ERROR: String expected, got Integer: 123124
025     direct method oops
026         let x::String = 123124.
                            ^^^^^^ String expected, got Integer: 123124
027         x",
    ));
    Ok(())
}

#[test]
fn test_method_arg_type_error_location() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test_method_arg_type_error_location.foo");
    cmd.assert().failure().code(1).stdout(predicates::str::contains(
        "ERROR: String expected, got Integer: 42
031     direct method oops
032         self oops: 42!
                 ^^^^^^^^ String expected, got Integer: 42
033     direct method oops: x::String",
    ));
    Ok(())
}

#[test]
fn test_block_arg_type_error_location() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test_block_arg_type_error_location.foo");
    cmd.assert().failure().code(1).stdout(predicates::str::contains(
        "ERROR: String expected, got Integer: 42
038     direct method oops
039         { |x::String| x } value: 42!
                              ^^^^^^^^^ String expected, got Integer: 42
040 end",
    ));
    Ok(())
}
*/

#[test]
fn test_import_error_location() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test_import_error_location.foo");
    cmd.assert().failure().code(1).stdout(predicates::str::contains(
        " ERROR: Cannot import ThisClassDoesNotExist: not defined in module
001 import .errorLocationTests.ThisClassDoesNotExist
    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Cannot import ThisClassDoesNotExist: not defined in module",
    ));
    Ok(())
}

#[test]
fn test_expr_at_toplevel_error_location() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test_expr_at_toplevel_error_location.foo");
    cmd.assert().failure().code(1).stdout(predicates::str::contains(
        "ERROR: Expression at toplevel
001 1::Integer
       ^^^^^^^ Expression at toplevel",
    ));
    Ok(())
}

#[test]
fn test_redefinition_error_location() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test_redefinition_error_location.foo");
    cmd.assert().failure().code(1).stdout(predicates::str::contains(
        "ERROR: Cannot redefine Integer
001 class Integer {}
    ^^^^^ Cannot redefine Integer
002 end",
    ));
    Ok(())
}

#[test]
fn test_undefined_value_type_error_location() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test_undefined_value_type_error_location.foo");
    cmd.assert().failure().code(1).stdout(predicates::str::contains(
        "ERROR: Undefined type: 'UndefinedType'
045     direct method oops: x
046         x::UndefinedType!
               ^^^^^^^^^^^^^ Undefined type: 'UndefinedType'
047 end",
    ));
    Ok(())
}

#[test]
fn test_undefined_var_type_error_location() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test_undefined_var_type_error_location.foo");
    // FIXME: Error points to variable, not the type
    cmd.assert().failure().code(1).stdout(predicates::str::contains(
        "ERROR: Undefined type: 'UndefinedType'
050     direct method oops
051         let x::UndefinedType = \"OK\".
                ^ Undefined type: 'UndefinedType'
052         x",
    ));
    Ok(())
}

#[test]
fn test_undefined_interface_error_location() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test_undefined_interface_error_location.foo");
    // FIXME: Error points to class
    cmd.assert().failure().code(1).stdout(predicates::str::contains(
        " ERROR: Undefined interface: UndefinedInterface
001 class BadClass {}
    ^^^^^ Undefined interface: UndefinedInterface
002     is UndefinedInterface",
    ));
    Ok(())
}

#[test]
fn test_panic_location() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test_panic_location.foo");
    // FIXME: Error points to class
    cmd.assert().failure().code(1).stdout(predicates::str::contains(
        "ERROR: This here
056     direct method oops
057         panic \"This here\"!
            ^^^^^ This here
058 end",
    ));
    Ok(())
}

#[test]
fn test_does_not_understand_location() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test_does_not_understand_location.foo");
    // FIXME: Error points to class
    cmd.assert().failure().code(1).stdout(predicates::str::contains(
        "ERROR: DoesNotUnderstandError classOf does not understand: noSuchMethod []
061     direct method oops
062         self noSuchMethod!
                 ^^^^^^^^^^^^ DoesNotUnderstandError classOf does not understand: noSuchMethod []
063 end",
    ));
    Ok(())
}

#[test]
fn test_interface_unimplemented() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test_interface_unimplemented.foo");
    cmd.assert()
        .failure()
        .code(1)
        .stdout(predicates::str::contains("ERROR: C#quux unimplemented, required by interface I"));
    Ok(())
}

#[test]
fn test_interface_bad_signature() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test_interface_bad_signature.foo");
    cmd.assert().failure().code(1).stdout(predicates::str::contains(
        "ERROR: C#quux is () -> Any, interface I specifies () -> Integer",
    ));
    Ok(())
}

#[test]
fn test_interface_ok() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
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
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test_interface_typecheck.foo");
    cmd.assert()
        .failure()
        .stdout(predicates::str::contains("YesI: True"))
        .stdout(predicates::str::contains("Type error: I expected, got NotI: #<NotI>"));
    Ok(())
}

#[test]
fn test_interface_inheritance() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
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
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test_import_x_local.foo");
    cmd.assert().failure().code(123).stdout("");
    Ok(())
}

#[test]
fn test_class_comments() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test_class_comments.foo");
    cmd.assert().success().stdout("");
    Ok(())
}

#[test]
fn test_import_x() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test_import_x.foo");
    cmd.arg("--use");
    cmd.arg("foo/tests/x.foo");
    cmd.assert().failure().code(123).stdout("");
    Ok(())
}

#[test]
fn test_import_x_no_use_no_local() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test_import_x.foo");
    cmd.assert()
        .failure()
        .code(1)
        .stdout(predicates::str::contains("FATAL - ERROR: Unknown module: x.foo"));
    Ok(())
}

#[test]
fn test_import_x_identity() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test_import_x_Identity.foo");
    cmd.arg("--use");
    cmd.arg("foo/tests/x.foo");
    cmd.assert().failure().code(100).stdout("");
    Ok(())
}

#[test]
fn test_import_x_star() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test_import_x_star.foo");
    cmd.arg("--use");
    cmd.arg("foo/tests/x.foo");
    cmd.assert().failure().code(42).stdout("");
    Ok(())
}

#[test]
fn test_import_bar_y() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test_import_bar_y.foo");
    cmd.arg("--use");
    cmd.arg("foo/tests/bar");
    cmd.assert().failure().code(111).stdout("");
    Ok(())
}

#[test]
fn test_prefixed_import() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test_prefixed_import.foo");
    cmd.assert().failure().stdout(predicates::str::contains(
        "X = eks
Y = why
FATAL - ERROR: Unbound variable: _Y
007         out println: \"Y = {Y value}\".
008         out println: \"_Y = {_Y}\".
                                ^^ Unbound variable: _Y
009         sys exit",
    ));
    Ok(())
}

#[test]
fn test_prelude1() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test_prelude.foo");
    cmd.assert().failure().code(2).stdout("");
    Ok(())
}

#[test]
fn test_prelude2() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
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
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test_array_let1.foo");
    cmd.assert().failure().code(1).stdout(predicates::str::contains(
        "ERROR: Unbound variable: x
002    direct method run: command in: system
003       system output println: [let x = 42. x + x, x]!
                                                     ^ Unbound variable: x
004 end

",
    ));
    Ok(())
}

#[test]
fn test_array_let2() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/tests/test_array_let2.foo");
    cmd.assert().failure().code(1).stdout(predicates::str::contains(
        "ERROR: Unbound variable: x
003       let a = [let x = 42. x, 123].
004       system output println: x!
                                 ^ Unbound variable: x
005 end
",
    ));
    Ok(())
}

#[test]
fn test_repl() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/repl.foo")
        .write_stdin(
            r#"class Point { x y }
                  direct method displayOn: stream
                     stream print: "<class Point>"!
                  method + other
                     Point x: x + other x
                           y: y + other y!
                  method displayOn: stream
                     stream print: "{x}@{y}"!
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
            r#"Foolang 0.1.0
> <class Point>
> 101@202
> Hi!
#<Output stdout>
> 1
> 42
> 42
> [43]
> PANIC: Unbound variable: inside
001                inside
                   ^^^^^^ Unbound variable: inside

> PANIC: Unbound variable: inside2
001                [let inside2 = 42. inside2 + 2, inside2]
                                                   ^^^^^^^ Unbound variable: inside2
"#,
        ));
    Ok(())
}

#[test]
fn test_benchmarks() -> Test {
    let mut cmd = Command::cargo_bin("foo")?;
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
    let mut cmd = Command::cargo_bin("foo")?;
    cmd.arg("foo/examples/flying.foo");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("TAKEOFF"))
        .stdout(predicate::str::contains("LANDING"))
        .stdout(predicate::str::contains("TOUCHDOWN"));
    Ok(())
}
