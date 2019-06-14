use crate::evaluator::{make_method_result, Eval, GlobalEnv};
use crate::objects::{Datum, Object};

pub fn method_print(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 1);
    match &receiver.datum {
        Datum::Output(out) => match &args[0].datum {
            Datum::String(s) => {
                out.write(s.lock().unwrap().as_bytes());
            }
            _ => {
                panic!("Bad argument to Output print: {}", args[0]);
            }
        },
        _ => panic!("Bad receiver for Output print: {}", receiver),
    }
    make_method_result(receiver.clone(), receiver)
}

pub fn method_newline(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 0);
    match &receiver.datum {
        Datum::Output(out) => {
            out.write("\n".as_bytes());
        }
        _ => panic!("Bad receiver for Output newline: {}", receiver),
    }
    make_method_result(receiver.clone(), receiver)
}

pub fn method_flush(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 0);
    match &receiver.datum {
        Datum::Output(out) => {
            out.flush();
        }
        _ => panic!("Bad receiver for Output flush: {}", receiver),
    }
    make_method_result(receiver.clone(), receiver)
}
