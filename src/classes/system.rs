use crate::evaluator::{make_method_result, Eval, GlobalEnv};
use crate::objects::Object;
use crate::time::TimeInfo;

pub fn class_method_stdin(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 0);
    make_method_result(receiver, Object::make_input(Box::new(std::io::stdin())))
}

pub fn class_method_stdout(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 0);
    make_method_result(receiver, Object::make_output(Box::new(std::io::stdout())))
}

pub fn class_method_timeinfo(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 0);
    make_method_result(receiver, Object::into_timeinfo(TimeInfo::now()))
}
