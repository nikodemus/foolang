use crate::evaluator::{make_method_result, Eval, GlobalEnv, MethodImpl};
use crate::objects::{Datum, Object};

pub fn method_createinstance(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 1);
    let class = receiver.class();
    let slots = args[0].vec();
    make_method_result(receiver, Object::make_instance(class.id.to_owned(), slots))
}

pub fn method_help(receiver: Object, args: Vec<Object>, global: &GlobalEnv) -> Eval {
    assert!(args.len() == 1);
    match &args[0].datum {
        Datum::Symbol(name) => {
            if let MethodImpl::Evaluator(m) = &global.find_method(&receiver.class, &name) {
                if let Some(s) = &m.docstring {
                    return make_method_result(receiver, Object::make_string(s));
                }
            }
        }
        _ => panic!("Bad argument to help:!"),
    }
    make_method_result(receiver, Object::make_string("No help available."))
}
