use crate::evaluator::eval;
use crate::objects::Object;
use crate::parser::parse_expr;

#[test]
fn eval_character() {
    assert_eq!(eval(parse_expr("$x")), Object::make_character("x"));
}

#[test]
fn eval_symbol() {
    assert_eq!(
        eval(parse_expr("#foo:bar:")),
        Object::make_symbol("foo:bar:")
    );
}

#[test]
fn eval_array() {
    assert_eq!(
        eval(parse_expr("#[1, 2, 3]")),
        Object::make_array(&[
            Object::make_integer(1),
            Object::make_integer(2),
            Object::make_integer(3)
        ])
    );
}

#[test]
fn eval_assign() {
    assert_eq!(
        eval(parse_expr("{ |x| x := 1 + 41, x } value")),
        Object::make_integer(42)
    )
}

#[test]
fn eval_unary() {
    assert_eq!(eval(parse_expr("123 neg")), Object::make_integer(-123));
    assert_eq!(
        eval(parse_expr("123.123 neg")),
        Object::make_float(-123.123)
    );
}

#[test]
fn eval_binary() {
    assert_eq!(eval(parse_expr("100 + 23 - 1")), Object::make_integer(122));
    assert_eq!(
        eval(parse_expr("100 + 23.32 - 2")),
        Object::make_float(121.32)
    );
}

#[test]
fn eval_keyword() {
    assert_eq!(eval(parse_expr("100 gcd: 12")), Object::make_integer(4));
}

#[test]
fn eval_global() {
    assert_eq!(
        eval(parse_expr("PI")),
        Object::make_float(std::f64::consts::PI)
    );
}

#[test]
fn eval_cascade() {
    assert_eq!(eval(parse_expr("1 + 100; + 41")), Object::make_integer(42));
}

#[test]
fn eval_array_ctor() {
    assert_eq!(
        eval(parse_expr("[1, 1+1, 3.0 neg neg]")),
        Object::make_array(&[
            Object::make_integer(1),
            Object::make_integer(2),
            Object::make_float(3.0),
        ])
    );
}
