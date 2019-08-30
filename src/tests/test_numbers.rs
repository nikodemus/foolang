use crate::evaluator::eval_str;
use crate::objects::Object;

#[test]
fn number_lt() {
    assert_eq!(eval_str("1 < 2.0"), Object::make_boolean(true));
    assert_eq!(eval_str("2 < 1.0"), Object::make_boolean(false));
    assert_eq!(eval_str("1 < 1.0"), Object::make_boolean(false));

    assert_eq!(eval_str("1.0 < 2"), Object::make_boolean(true));
    assert_eq!(eval_str("2.0 < 1"), Object::make_boolean(false));
    assert_eq!(eval_str("1.0 < 1"), Object::make_boolean(false));
}

#[test]
fn number_gt() {
    assert_eq!(eval_str("1 > 2.0"), Object::make_boolean(false));
    assert_eq!(eval_str("2 > 1.0"), Object::make_boolean(true));
    assert_eq!(eval_str("1 > 1.0"), Object::make_boolean(false));

    assert_eq!(eval_str("1.0 > 2"), Object::make_boolean(false));
    assert_eq!(eval_str("2.0 > 1"), Object::make_boolean(true));
    assert_eq!(eval_str("1.0 > 1"), Object::make_boolean(false));
}

#[test]
fn number_eq() {
    assert_eq!(eval_str("1 == 2.0"), Object::make_boolean(false));
    assert_eq!(eval_str("2 == 1.0"), Object::make_boolean(false));
    assert_eq!(eval_str("1 == 1.0"), Object::make_boolean(true));

    assert_eq!(eval_str("1.0 == 2"), Object::make_boolean(false));
    assert_eq!(eval_str("2.0 == 1"), Object::make_boolean(false));
    assert_eq!(eval_str("1.0 == 1"), Object::make_boolean(true));
}
