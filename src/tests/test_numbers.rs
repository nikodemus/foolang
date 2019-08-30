use crate::evaluator::eval_str;
use crate::objects::Object;

#[test]
fn eval_float() {
    assert_eq!(eval_str("123.123"), Object::make_float(123.123));
}

#[test]
fn number_neg() {
    assert_eq!(eval_str("123.123 neg"), Object::make_float(-123.123));
}

#[test]
fn number_add() {
    assert_eq!(eval_str("100.0 + 23.32"), Object::make_float(123.32));
    assert_eq!(eval_str("100.0 + 23"), Object::make_float(123.0));
}

#[test]
fn number_sub() {
    assert_eq!(eval_str("100.0 - 23.32"), Object::make_float(76.68));
    assert_eq!(eval_str("100.0 - 23"), Object::make_float(77.0));
}

#[test]
fn number_lt() {
    assert_eq!(eval_str("1 < 2.0"), Object::make_boolean(true));
    assert_eq!(eval_str("2 < 1.0"), Object::make_boolean(false));
    assert_eq!(eval_str("1 < 1.0"), Object::make_boolean(false));

    assert_eq!(eval_str("1.0 < 2"), Object::make_boolean(true));
    assert_eq!(eval_str("2.0 < 1"), Object::make_boolean(false));
    assert_eq!(eval_str("1.0 < 1"), Object::make_boolean(false));

    assert_eq!(eval_str("1.0 < 2.0"), Object::make_boolean(true));
    assert_eq!(eval_str("2.0 < 1.0"), Object::make_boolean(false));
    assert_eq!(eval_str("1.0 < 1.0"), Object::make_boolean(false));
}

#[test]
fn number_gt() {
    assert_eq!(eval_str("1 > 2.0"), Object::make_boolean(false));
    assert_eq!(eval_str("2 > 1.0"), Object::make_boolean(true));
    assert_eq!(eval_str("1 > 1.0"), Object::make_boolean(false));

    assert_eq!(eval_str("1.0 > 2"), Object::make_boolean(false));
    assert_eq!(eval_str("2.0 > 1"), Object::make_boolean(true));
    assert_eq!(eval_str("1.0 > 1"), Object::make_boolean(false));

    assert_eq!(eval_str("1.0 > 2.0"), Object::make_boolean(false));
    assert_eq!(eval_str("2.0 > 1.0"), Object::make_boolean(true));
    assert_eq!(eval_str("1.0 > 1.0"), Object::make_boolean(false));
}

#[test]
fn number_eq() {
    assert_eq!(eval_str("1 == 2.0"), Object::make_boolean(false));
    assert_eq!(eval_str("2 == 1.0"), Object::make_boolean(false));
    assert_eq!(eval_str("1 == 1.0"), Object::make_boolean(true));

    assert_eq!(eval_str("1.0 == 2"), Object::make_boolean(false));
    assert_eq!(eval_str("2.0 == 1"), Object::make_boolean(false));
    assert_eq!(eval_str("1.0 == 1"), Object::make_boolean(true));

    assert_eq!(eval_str("1.0 == 2.0"), Object::make_boolean(false));
    assert_eq!(eval_str("2.0 == 1.0"), Object::make_boolean(false));
    assert_eq!(eval_str("1.0 == 1.0"), Object::make_boolean(true));
}
