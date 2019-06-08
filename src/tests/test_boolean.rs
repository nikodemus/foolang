use crate::evaluator::eval_str;
use crate::objects::Object;

#[test]
fn eval_true() {
    assert_eq!(eval_str("true"), Object::make_boolean(true));
}

#[test]
fn eval_false() {
    assert_eq!(eval_str("false"), Object::make_boolean(false));
}
