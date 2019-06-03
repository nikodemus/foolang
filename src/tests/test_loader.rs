use crate::evaluator::GlobalEnv;
use crate::objects::Object;
use crate::parser::*;

#[test]
fn load_empty_class_and_class_methods() {
    let prog = parse_program(
        "
        @class Truth []
        @class-method Truth theAnswer
            ^42
        @class Falsehood []
        @class-method Falsehood theAnswer
            ^13
    ",
    );
    let mut env = GlobalEnv::new();
    env.load(prog);
    assert_eq!(
        env.eval(parse_expr("Truth theAnswer")),
        Object::make_integer(42)
    );
    assert_eq!(
        env.eval(parse_expr("Falsehood theAnswer")),
        Object::make_integer(13)
    );
}

#[test]
fn load_full_class() {
    let prog = parse_program(
        "
        @class Box [_value]
        @class-method Box new: value |box|
           ^self createInstance: [value]
        @method Box value
            ^_value
        @method Box value: newval
            _value := newval
    ",
    );
    let mut env = GlobalEnv::new();
    env.load(prog);
    assert_eq!(
        env.eval(parse_expr(
            "{ |x| x := Box new: 40. x value: x value + 2. x value } value"
        )),
        Object::make_integer(42)
    );
}
