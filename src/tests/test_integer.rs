use crate::eval::utils::{eval_ok, eval_str};
use crate::unwind::Unwind;
use crate::unwind::{Error, Location, SimpleError};

#[test]
fn test_decimal_integer() {
    assert_eq!(eval_ok("123").integer(), 123);
}

#[test]
fn test_bad_decimal_integer() {
    assert_eq!(
        eval_str("1x3"),
        Err(Unwind::Exception(
            Error::SimpleError(SimpleError {
                what: "Malformed number".to_string(),
            }),
            Location {
                span: Some(0..3),
                context: Some(concat!("001 1x3\n", "    ^^^ Malformed number\n").to_string())
            }
        ))
    );
}

#[test]
fn test_hex_integer() {
    assert_eq!(eval_ok("0xFFFF").integer(), 0xFFFF);
}

#[test]
fn test_bad_hex_integer() {
    assert_eq!(
        eval_str("0x1x3"),
        Err(Unwind::Exception(
            Error::SimpleError(SimpleError {
                what: "Malformed hexadecimal number".to_string(),
            }),
            Location {
                span: Some(0..5),
                context: Some(
                    concat!("001 0x1x3\n", "    ^^^^^ Malformed hexadecimal number\n").to_string()
                )
            }
        ))
    );
}

#[test]
fn test_binary_integer() {
    assert_eq!(eval_ok("0b101").integer(), 0b101);
}

#[test]
fn test_bad_binary_integer() {
    assert_eq!(
        eval_str("0b123"),
        Err(Unwind::Exception(
            Error::SimpleError(SimpleError {
                what: "Malformed binary number".to_string(),
            }),
            Location {
                span: Some(0..5),
                context: Some(
                    concat!("001 0b123\n", "    ^^^^^ Malformed binary number\n").to_string()
                )
            }
        ))
    );
}

#[test]
fn test_add_integer() {
    assert_eq!(eval_ok("1 + 1").integer(), 2);
    assert_eq!(eval_ok("1 + 1.0").float(), 2.0);
}

#[test]
fn test_sub_integer() {
    assert_eq!(eval_ok("10 - 5").integer(), 5);
    assert_eq!(eval_ok("10 - 5.0").float(), 5.0);
}

#[test]
fn test_div_integer() {
    assert_eq!(eval_ok("10 / 5").integer(), 2);
    assert_eq!(eval_ok("10 / 5.0").float(), 2.0);
}

#[test]
fn test_mul_integer() {
    assert_eq!(eval_ok("10 * 5").integer(), 50);
    assert_eq!(eval_ok("10 * 5.0").float(), 50.0);
}

#[test]
fn test_addmul_integer() {
    assert_eq!(eval_ok("1 + 1 * 2").integer(), 3);
}

#[test]
fn test_integer_as_integer() {
    assert_eq!(eval_ok("42 asInteger").integer(), 42);
}

#[test]
fn test_integer_as_float() {
    assert_eq!(eval_ok("42 asFloat").float(), 42.0);
}

#[test]
fn test_integer_equal() {
    assert_eq!(eval_ok("1 == 2").boolean(), false);
    assert_eq!(eval_ok("2 == 1").boolean(), false);
    assert_eq!(eval_ok("1 == 1").boolean(), true);

    assert_eq!(eval_ok("1 == 1.0").boolean(), true);
    assert_eq!(eval_ok("1 == 1.1").boolean(), false);
    assert_eq!(eval_ok("1 == 0.9").boolean(), false);
}

#[test]
fn test_integer_less_than() {
    assert_eq!(eval_ok("1 < 2").boolean(), true);
    assert_eq!(eval_ok("2 < 1").boolean(), false);
    assert_eq!(eval_ok("1 < 1").boolean(), false);

    assert_eq!(eval_ok("1 < 2.0").boolean(), true);
    assert_eq!(eval_ok("2 < 1.0").boolean(), false);
    assert_eq!(eval_ok("1 < 1.0").boolean(), false);
}

#[test]
fn test_integer_less_than_or_equal() {
    assert_eq!(eval_ok("1 <= 2").boolean(), true);
    assert_eq!(eval_ok("2 <= 1").boolean(), false);
    assert_eq!(eval_ok("1 <= 1").boolean(), true);

    assert_eq!(eval_ok("1 <= 2.0").boolean(), true);
    assert_eq!(eval_ok("2 <= 1.0").boolean(), false);
    assert_eq!(eval_ok("1 <= 1.0").boolean(), true);
}

#[test]
fn test_integer_greater_than() {
    assert_eq!(eval_ok("1 > 2").boolean(), false);
    assert_eq!(eval_ok("2 > 1").boolean(), true);
    assert_eq!(eval_ok("1 > 1").boolean(), false);

    assert_eq!(eval_ok("1 > 2.0").boolean(), false);
    assert_eq!(eval_ok("2 > 1.0").boolean(), true);
    assert_eq!(eval_ok("1 > 1.0").boolean(), false);
}

#[test]
fn test_integer_greater_than_or_equal() {
    assert_eq!(eval_ok("1 >= 2").boolean(), false);
    assert_eq!(eval_ok("2 >= 1").boolean(), true);
    assert_eq!(eval_ok("1 >= 1").boolean(), true);

    assert_eq!(eval_ok("1 >= 2.0").boolean(), false);
    assert_eq!(eval_ok("2 >= 1.0").boolean(), true);
    assert_eq!(eval_ok("1 >= 1.0").boolean(), true);
}

#[test]
fn test_integer_times() {
    assert_eq!(
        eval_ok(
            "let x = 1.
             3 times: { x = x + x }.
             x"
        )
        .integer(),
        1 + 1 + 2 + 4
    )
}

#[test]
fn test_integer_prefix_minus() {
    assert_eq!(eval_ok("let x = -42. -x").integer(), 42);
    assert_eq!(eval_ok("let x = 42. -x").integer(), -42);
}
