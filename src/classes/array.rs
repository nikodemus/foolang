use crate::evaluator::{closure_apply, make_method_result, Eval, GlobalEnv};
use crate::objects::{Datum, Object};

pub fn method_do(receiver: Object, args: Vec<Object>, global: &GlobalEnv) -> Eval {
    assert!(args.len() == 1);
    match &receiver.datum {
        Datum::Array(v) => {
            let closure = args[0].closure();
            for each in v.lock().unwrap().iter() {
                let res = closure_apply(receiver.clone(), &closure, &vec![each.to_owned()], global);
                if res.is_return() {
                    return res;
                }
            }
        }
        _ => panic!("TypeError: {} is not an Array", receiver),
    }
    make_method_result(receiver.clone(), receiver)
}

pub fn method_inject_into(receiver: Object, args: Vec<Object>, global: &GlobalEnv) -> Eval {
    assert!(args.len() == 2);
    match &receiver.datum {
        Datum::Array(v) => {
            let mut value = args[0].to_owned();
            let closure = args[1].closure();
            for each in v.lock().unwrap().iter() {
                let res = closure_apply(
                    receiver.clone(),
                    &closure,
                    &vec![value, each.to_owned()],
                    global,
                );
                if res.is_return() {
                    return res;
                } else {
                    value = res.value()
                }
            }
            make_method_result(receiver, value)
        }
        _ => panic!("TypeError: {} is not an Array", receiver),
    }
}

pub fn method_push(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 1);
    match &receiver.datum {
        Datum::Array(array) => {
            let mut v = array.lock().unwrap();
            v.push(args[0].to_owned());
        }
        _ => panic!("TypeError: {} is not an Array", receiver),
    }
    make_method_result(receiver.clone(), receiver)
}
