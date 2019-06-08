use crate::evaluator::{eval_str, load_str};
use crate::objects::Object;

#[test]
fn block_0_value() {
    assert_eq!(eval_str("{ 42 } value"), Object::make_integer(42));
}

#[test]
fn block_1_value() {
    assert_eq!(
        eval_str("{ :a | a + 1 } value: 41"),
        Object::make_integer(42)
    );
}

#[test]
fn block_2_value() {
    assert_eq!(
        eval_str("{ :a :b | b * a + 2 } value: 20 value: 2"),
        Object::make_integer(42)
    );
}

#[test]
fn block_closure() {
    let env = load_str(
        r#"
        @class F []
        @class-method F closeOver: value
           ^{ :x | value + x }
        @class-method F test
            ^(self closeOver: 40) value: 2
    "#,
    );
    assert_eq!(env.eval_str("F test"), Object::make_integer(42));
}

#[test]
fn block_closure_mutation() {
    let env = load_str(
        r#"
        @class F []
        @class-method F counter
           |x| x := 0.
           ^{ x := x + 1. x }
        @class-method F test
            |counter| counter := self counter.
            ^[counter value . counter value . counter value]
    "#,
    );
    assert_eq!(
        env.eval_str("F test"),
        Object::make_array(&[
            Object::make_integer(1),
            Object::make_integer(2),
            Object::make_integer(3)
        ])
    );
}

#[test]
fn return_from_method_block() {
    let env = load_str(
        r#"
        @class Foo []
        @class-method Foo test
            Foo boo: { ^42 }.
            ^31
        @class-method Foo boo: blk
            blk value
        "#,
    );
    assert_eq!(env.eval_str("Foo test"), Object::make_integer(42));
}

#[test]
fn block_repeat() {
    let env = load_str(
        r#"
        @class Foo []
        @class-method Foo test |x|
           x := 0.
           {
               x > 1000 ifTrue: { ^'lots' }.
               x := x + 1
           }
           repeat
        "#,
    );
    assert_eq!(env.eval_str("Foo test"), Object::make_string("lots"));
}
