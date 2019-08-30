use crate::eval::utils::{eval_ok, eval_str};
use crate::unwind::Unwind;
use crate::unwind::{Error, Location, SimpleError};

#[test]
fn test_float() {
    assert_eq!(eval_ok("1.2").float(), 1.2);
}

#[test]
fn test_bad_float() {
    assert_eq!(
        eval_str("1.2.3"),
        Err(Unwind::Exception(
            Error::SimpleError(SimpleError {
                what: "Unknown operator",
            }),
            Location {
                span: Some(3..4),
                context: Some(concat!("001 1.2.3\n", "       ^ Unknown operator\n").to_string())
            }
        ))
    );
}

#[test]
fn test_float_prefix() {
    assert_eq!(eval_ok("let x = -42.0, -x").float(), 42.0);
    assert_eq!(eval_ok("let x = 42.0, -x").float(), -42.0);
}

#[test]
fn test_float_div() {
    assert_eq!(eval_ok("10.0 / 5.0").float(), 2.0);
    assert_eq!(eval_ok("10.0 / 5").float(), 2.0);
}

#[test]
fn test_float_sub() {
    assert_eq!(eval_ok("10.0 - 5.0").float(), 5.0);
    assert_eq!(eval_ok("10.0 - 5").float(), 5.0);
}

#[test]
fn test_float_add() {
    assert_eq!(eval_ok("10.0 + 5.0").float(), 15.0);
    assert_eq!(eval_ok("10.0 + 5").float(), 15.0);
}

#[test]
fn test_float_mul() {
    assert_eq!(eval_ok("10.0 * 5.0").float(), 50.0);
    assert_eq!(eval_ok("10.0 * 5").float(), 50.0);
}

#[test]
fn test_float_as_float() {
    assert_eq!(eval_ok("42.3 asFloat").float(), 42.3);
}

#[test]
fn test_float_as_integer() {
    assert_eq!(eval_ok("42.1 asInteger").integer(), 42);
    assert_eq!(eval_ok("41.9 asInteger").integer(), 42);
}

#[test]
fn test_addmul_float() {
    assert_eq!(eval_ok("1.0 + 1.0 * 2.0").float(), 3.0);
}

#[test]
fn test_float_lt() {
    assert_eq!(eval_ok("1.0 < 2.0").boolean(), true);
    assert_eq!(eval_ok("2.0 < 1.0").boolean(), false);
    assert_eq!(eval_ok("1.0 < 1.0").boolean(), false);
}

#[test]
fn test_float_gt() {
    assert_eq!(eval_ok("1.0 > 2.0").boolean(), false);
    assert_eq!(eval_ok("2.0 > 1.0").boolean(), true);
    assert_eq!(eval_ok("1.0 > 1.0").boolean(), false);
}
