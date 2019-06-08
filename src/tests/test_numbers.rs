use crate::evaluator::{eval_str, load_str};
use crate::objects::Object;

#[test]
fn eval_integer() {
    assert_eq!(eval_str("123"), Object::make_integer(123));
}

#[test]
fn eval_float() {
    assert_eq!(eval_str("123.123"), Object::make_float(123.123));
}

#[test]
fn number_neg() {
    assert_eq!(eval_str("123 neg"), Object::make_integer(-123));
    assert_eq!(eval_str("123.123 neg"), Object::make_float(-123.123));
}

#[test]
fn number_add() {
    assert_eq!(eval_str("100 + 23"), Object::make_integer(123));
    assert_eq!(eval_str("100 + 23.32"), Object::make_float(123.32));
    assert_eq!(eval_str("100.0 + 23.32"), Object::make_float(123.32));
    assert_eq!(eval_str("100.0 + 23"), Object::make_float(123.0));
}

#[test]
fn number_sub() {
    assert_eq!(eval_str("100 - 23"), Object::make_integer(77));
    assert_eq!(eval_str("100 - 23.32"), Object::make_float(76.68));
    assert_eq!(eval_str("100.0 - 23.32"), Object::make_float(76.68));
    assert_eq!(eval_str("100.0 - 23"), Object::make_float(77.0));
}

#[test]
fn integer_gcd() {
    assert_eq!(eval_str("100 gcd: 12"), Object::make_integer(4));
}
