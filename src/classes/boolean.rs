use crate::evaluator::{closure_apply, make_result, Eval, GlobalEnv};
use crate::objects::Object;

pub fn method_iffalse(receiver: Object, args: Vec<Object>, global: &GlobalEnv) -> Eval {
    assert!(args.len() == 1);
    if !receiver.boolean() {
        let closure = args[0].closure();
        closure_apply(receiver, &closure, &vec![], global)
    } else {
        make_result(receiver)
    }
}

pub fn method_iftrue(receiver: Object, args: Vec<Object>, global: &GlobalEnv) -> Eval {
    assert!(args.len() == 1);
    if receiver.boolean() {
        let closure = args[0].closure();
        closure_apply(receiver, &closure, &vec![], global)
    } else {
        make_result(receiver)
    }
}
