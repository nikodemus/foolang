use crate::evaluator::{make_method_result, Eval, GlobalEnv};
use crate::objects::Object;

pub fn class_method_compiler(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 0);
    make_method_result(receiver, Object::make_compiler())
}
