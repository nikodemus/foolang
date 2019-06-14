use crate::evaluator::{make_method_result, Eval, GlobalEnv};
use crate::objects::{Datum, Object};

pub fn method_readline(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 0);
    let line = match &receiver.datum {
        Datum::Input(input) => match input.read_line() {
            Some(s) => Object::into_string(s),
            None => Object::make_boolean(false),
        },
        _ => panic!("Bad receiver for Input readline: {}", receiver),
    };
    make_method_result(receiver, line)
}
