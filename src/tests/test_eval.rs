use crate::evaluator::eval;
use crate::objects::Object;
use crate::parser::parse_expr;

#[test]
fn eval_number() {
    assert_eq!(eval(parse_expr("123")), Object::Integer(123));
    assert_eq!(eval(parse_expr("123.123")), Object::Float(123.123));
}

#[test]
fn eval_string() {
    assert_eq!(eval(parse_expr("'foo'")), Object::make_string("foo"));
}

#[test]
fn eval_character() {
    assert_eq!(eval(parse_expr("$x")), Object::make_char("x"));
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
        eval(parse_expr("#[1 2 3]")),
        Object::make_array(&[Object::Integer(1), Object::Integer(2), Object::Integer(3)])
    );
}

#[test]
fn eval_assign() {
    assert_eq!(
        eval(parse_expr("{ |x| x := 1 + 41. x } value")),
        Object::Integer(42)
    )
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

#[test]
fn eval_keyword() {
    assert_eq!(eval(parse_expr("100 gcd: 12")), Object::Integer(4));
}

#[test]
fn eval_global() {
    assert_eq!(eval(parse_expr("PI")), Object::Float(std::f64::consts::PI));
}

#[test]
fn eval_block() {
    assert_eq!(
        eval(parse_expr("{ :a | a + 1 } value: 41")),
        Object::Integer(42)
    );
    assert_eq!(
        eval(parse_expr("{ :a :b | b * a + 2 } a: 20 b: 2")),
        Object::Integer(42)
    );
}

#[test]
fn eval_cascade() {
    assert_eq!(eval(parse_expr("1 + 100; + 41")), Object::Integer(42));
}

/*
#[test]
fn eval_return() {
    let m = parse_method("double ^self * 2. 123123132");
    let env = Env::new();
    env.find_class("Integer").add_method("double", m);
    assert_eq!(
        eval_in_env(parse_expr("21 double"), env),
        Object::Integer(42));
}
*/
