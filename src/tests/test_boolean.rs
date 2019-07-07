use crate::evaluator::{eval_str, load_str};
use crate::objects::Object;

#[test]
fn eval_true() {
    assert_eq!(eval_str("true"), Object::make_boolean(true));
}

#[test]
fn eval_false() {
    assert_eq!(eval_str("false"), Object::make_boolean(false));
}

#[test]
fn boolean_iftrue() {
    let env = load_str(
        r#"
        @class Foo []
        @class-method Foo test: x |y|
           y := 0,
           x ifTrue: { y := 1 },
           ^y
        "#,
    );
    assert_eq!(env.eval_str("Foo test: true"), Object::make_integer(1));
    assert_eq!(env.eval_str("Foo test: false"), Object::make_integer(0));
}
