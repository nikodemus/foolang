use crate::evaluator::{make_method_result, Eval, GlobalEnv};
use crate::objects::{Datum, Object};

pub fn method_gcd(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 1);
    match receiver.datum.clone() {
        Datum::Integer(i) => match args[0].datum {
            Datum::Integer(j) => {
                make_method_result(receiver, Object::make_integer(num::integer::gcd(i, j)))
            }
            _ => panic!("Non-integer in gcd!"),
        },
        _ => panic!("Bad receiver for builtin gcd!"),
    }
}
