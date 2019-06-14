use crate::evaluator::{make_method_result, Eval, GlobalEnv};
use crate::objects::Object;

pub fn eq(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 1);
    let boolean = Object::make_boolean(receiver == args[0]);
    make_method_result(receiver, boolean)
}

pub fn tostring(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 0);
    let string = Object::into_string(format!("{}", &receiver));
    make_method_result(receiver, string)
}
