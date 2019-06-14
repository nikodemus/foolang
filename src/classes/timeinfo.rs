use crate::evaluator::{make_method_result, Eval, GlobalEnv};
use crate::objects::Object;

pub fn method_minus(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 1);
    let delta = (&*receiver.timeinfo()).to_owned() - (&*args[0].timeinfo()).to_owned();
    make_method_result(receiver, Object::into_timeinfo(delta))
}

pub fn method_realtime(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 0);
    let real = receiver.timeinfo().real;
    make_method_result(receiver, Object::make_float(real))
}

pub fn method_systemtime(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 0);
    let system = receiver.timeinfo().system;
    make_method_result(receiver, Object::make_float(system))
}

pub fn method_usertime(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 0);
    let user = receiver.timeinfo().user;
    make_method_result(receiver, Object::make_float(user))
}
