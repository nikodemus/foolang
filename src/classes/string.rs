use crate::evaluator::{make_method_result, make_result, Eval, GlobalEnv};
use crate::objects::{Datum, Object};

pub fn class_method_new(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 0);
    make_method_result(receiver, Object::make_string(""))
}

pub fn method_append(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 1);
    match (&receiver.datum, &args[0].datum) {
        (Datum::String(s), Datum::String(more)) => {
            s.lock().unwrap().push_str(more.to_string().as_str());
            make_result(receiver)
        }
        _ => panic!("Bad arguments to 'String append:': #{:?}", args),
    }
}

pub fn method_clear(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 0);
    match &receiver.datum {
        Datum::String(s) => {
            s.lock().unwrap().clear();
            make_result(receiver)
        }
        _ => panic!("Bad receiver in 'String clear': #{:?}", args),
    }
}
