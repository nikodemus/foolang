use crate::evaluator::GlobalEnv;
use crate::objects::Object;
use crate::parser::*;

#[test]
fn load_empty_class_and_class_methods() {
    let prog = parse_program(
        r#"
        @class Truth []
        @class-method Truth theAnswer
           "...to life, universe, and everything!"
            ^42
        @class-method Truth whatItMeans
            ^self theAnswer
        @class-method Truth whatItMeansReally
            ^{ self theAnswer } value
        @class Falsehood []
        @class-method Falsehood theAnswer
            ^13
    "#,
    );
    let mut env = GlobalEnv::new();
    env.load(prog);
    assert_eq!(
        env.eval(parse_expr("Truth theAnswer")),
        Object::make_integer(42)
    );
    assert_eq!(
        env.eval(parse_expr("Truth whatItMeans")),
        Object::make_integer(42)
    );
    assert_eq!(
        env.eval(parse_expr("Truth whatItMeansReally")),
        Object::make_integer(42)
    );
    assert_eq!(
        env.eval(parse_expr("Falsehood theAnswer")),
        Object::make_integer(13)
    );
    assert_eq!(
        env.eval(parse_expr("Truth help: #theAnswer")),
        Object::make_string("...to life, universe, and everything!")
    );
}

#[test]
fn load_box() {
    let prog = parse_program(
        r#"
        @class Box [val]
        @class-method Box new: value
           "Create a Box instance holding the specified value."
           ^self createInstance: [value]
        @method Box value
            ^val
        @method Box value: newval
            val := newval
    "#,
    );
    let mut env = GlobalEnv::new();
    env.load(prog);
    assert_eq!(
        env.eval(parse_expr(
            "{ |x| x := Box new: 40. x value: 2 + x value. x value } value"
        )),
        Object::make_integer(42)
    );
}

#[test]
fn load_block_closure() {
    let prog = parse_program(
        r#"
        @class F []
        @class-method F closeOver: value
           ^{ :x | value + x }
        @class-method F test
            ^(self closeOver: 40) call: 2
    "#,
    );
    let mut env = GlobalEnv::new();
    env.load(prog);
    assert_eq!(env.eval(parse_expr("F test")), Object::make_integer(42));
}
