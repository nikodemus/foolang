use crate::evaluator::eval;
use crate::objects::Object;
use crate::parser::parse_expr;

#[test]
fn eval_string() {
    assert_eq!(eval(parse_expr("'foo'")), Object::make_string("foo"));
}

#[test]
fn string_new() {
    assert_eq!(eval(parse_expr("String new")), Object::make_string(""));
}

#[test]
fn string_append() {
    assert_eq!(
        eval(parse_expr(
            "{ |x| x := String new, x append: 'foo', x append: 'bar', x} value"
        )),
        Object::make_string("foobar")
    );
}

#[test]
fn string_clear() {
    assert_eq!(
        eval(parse_expr(
            "{ |x| x := String new, x append: 'foo', x clear append: 'bar', x} value"
        )),
        Object::make_string("bar")
    );
}
