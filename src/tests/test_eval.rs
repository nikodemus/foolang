use crate::evaluator::eval;
use crate::objects::Object;
use crate::parser::parse_expr;

#[test]
fn eval_number() {
    assert_eq!(eval(parse_expr("123")), Object::Integer(123));
    assert_eq!(eval(parse_expr("123.123")), Object::Float(123.123));
}

#[test]
fn eval_unary() {
    assert_eq!(eval(parse_expr("123 neg")), Object::Integer(-123));
    assert_eq!(eval(parse_expr("123.123 neg")), Object::Float(-123.123));
}

#[test]
fn eval_binary() {
    assert_eq!(eval(parse_expr("100 + 23 - 1")), Object::Integer(122));
    assert_eq!(eval(parse_expr("100 + 23.32 - 2")), Object::Float(121.32));
}
