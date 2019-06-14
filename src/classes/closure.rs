use crate::evaluator::{closure_apply, make_method_result, Eval, GlobalEnv};
use crate::objects::{Datum, Object};

pub fn method_apply(receiver: Object, args: Vec<Object>, global: &GlobalEnv) -> Eval {
    let closure = receiver.closure();
    closure_apply(receiver, &closure, &args, global)
}

pub fn method_until(receiver: Object, args: Vec<Object>, global: &GlobalEnv) -> Eval {
    assert!(args.len() == 1);
    let closure = receiver.closure();
    let test = args[0].closure();
    loop {
        let res = closure_apply(receiver.clone(), &closure, &vec![], global);
        if res.is_return() {
            return res;
        }
        let res = closure_apply(receiver.clone(), &test, &vec![], global);
        if res.is_return() || res.is_true() {
            return make_method_result(receiver, res.value());
        }
    }
}

pub fn method_repeat(receiver: Object, args: Vec<Object>, global: &GlobalEnv) -> Eval {
    assert!(args.len() == 0);
    match receiver.datum.clone() {
        Datum::Closure(closure) => loop {
            if let Eval::Return(val, to) =
                closure_apply(receiver.clone(), &closure, &vec![], global)
            {
                return Eval::Return(val, to);
            }
        },
        _ => panic!("Bad receiver for closure repeat!"),
    }
}

pub fn method_repeatwhilefalse(receiver: Object, args: Vec<Object>, global: &GlobalEnv) -> Eval {
    let closure = receiver.closure();
    loop {
        let res = closure_apply(receiver.clone(), &closure, &args, global);
        if res.is_return() {
            return res;
        }
        if !res.is_false() {
            return make_method_result(receiver, res.value());
        }
    }
}
