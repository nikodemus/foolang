use crate::evaluator::{closure_apply, make_method_result, Eval, GlobalEnv};
use crate::objects::{Datum, Object};

pub fn method_eq(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 1);
    make_method_result(
        receiver.clone(),
        match receiver.datum {
            Datum::Integer(i) => match args[0].datum {
                Datum::Integer(j) => Object::make_boolean(i == j),
                Datum::Float(j) => Object::make_boolean((i as f64) == j),
                _ => Object::make_boolean(false),
            },
            Datum::Float(i) => match args[0].datum {
                Datum::Integer(j) => Object::make_boolean(i == (j as f64)),
                Datum::Float(j) => Object::make_boolean(i == j),
                _ => panic!("Bad argument to Float ==: {}", args[0]),
            },
            _ => panic!("Bad receiver in method_number_eq: {}", receiver),
        },
    )
}

pub fn method_gt(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 1);
    make_method_result(
        receiver.clone(),
        match receiver.datum {
            Datum::Integer(i) => match args[0].datum {
                Datum::Integer(j) => Object::make_boolean(i > j),
                Datum::Float(j) => Object::make_boolean((i as f64) > j),
                _ => panic!("Bad argument to Integer >: {}", args[0]),
            },
            Datum::Float(i) => match args[0].datum {
                Datum::Integer(j) => Object::make_boolean(i > (j as f64)),
                Datum::Float(j) => Object::make_boolean(i > j),
                _ => panic!("Bad argument to Float >: {}", args[0]),
            },
            _ => panic!("Bad receiver in method_number_gt: {}", receiver),
        },
    )
}

pub fn method_lt(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 1);
    make_method_result(
        receiver.clone(),
        match receiver.datum {
            Datum::Integer(i) => match args[0].datum {
                Datum::Integer(j) => Object::make_boolean(i < j),
                Datum::Float(j) => Object::make_boolean((i as f64) < j),
                _ => panic!("Bad argument to Integer <: {}", args[0]),
            },
            Datum::Float(i) => match args[0].datum {
                Datum::Integer(j) => Object::make_boolean(i < (j as f64)),
                Datum::Float(j) => Object::make_boolean(i < j),
                _ => panic!("Bad argument to Float <: {}", args[0]),
            },
            _ => panic!("Bad receiver in method_number_lt: {}", receiver),
        },
    )
}

pub fn method_minus(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 1);
    make_method_result(
        receiver.clone(),
        match receiver.datum {
            Datum::Integer(i) => match args[0].datum {
                Datum::Integer(j) => Object::make_integer(i - j),
                Datum::Float(j) => Object::make_float((i as f64) - j),
                _ => panic!("Bad argument to Integer -: {}", args[0]),
            },
            Datum::Float(i) => match args[0].datum {
                Datum::Integer(j) => Object::make_float(i - (j as f64)),
                Datum::Float(j) => Object::make_float(i - j),
                _ => panic!("Bad argument to Float -: {}", args[0]),
            },
            _ => panic!("Bad receiver in method_number_minus: {}", receiver),
        },
    )
}

pub fn method_mul(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 1);
    make_method_result(
        receiver.clone(),
        match receiver.datum {
            Datum::Integer(i) => match args[0].datum {
                Datum::Integer(j) => Object::make_integer(i * j),
                Datum::Float(j) => Object::make_float((i as f64) * j),
                _ => panic!("Bad argument to Integer *: {}", args[0]),
            },
            Datum::Float(i) => match args[0].datum {
                Datum::Integer(j) => Object::make_float(i * (j as f64)),
                Datum::Float(j) => Object::make_float(i * j),
                _ => panic!("Bad argument to Float *: {}", args[0]),
            },
            _ => panic!("Bad receiver in method_number_mul: {}", receiver),
        },
    )
}

pub fn method_neg(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 0);
    make_method_result(
        receiver.clone(),
        match receiver.datum {
            Datum::Integer(i) => Object::make_integer(-i),
            Datum::Float(i) => Object::make_float(-i),
            _ => panic!("Bad receiver for neg!"),
        },
    )
}

pub fn method_plus(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 1);
    make_method_result(
        receiver.clone(),
        match receiver.datum {
            Datum::Integer(i) => match args[0].datum {
                Datum::Integer(j) => Object::make_integer(i + j),
                Datum::Float(j) => Object::make_float((i as f64) + j),
                _ => panic!("Bad argument to Integer +: {}", args[0]),
            },
            Datum::Float(i) => match args[0].datum {
                Datum::Integer(j) => Object::make_float(i + (j as f64)),
                Datum::Float(j) => Object::make_float(i + j),
                _ => panic!("Bad argument to Float +: {}", args[0]),
            },
            _ => panic!("Bad receiver in method_number_plus: {}", receiver),
        },
    )
}

pub fn method_to_do(receiver: Object, args: Vec<Object>, global: &GlobalEnv) -> Eval {
    assert!(args.len() == 2);
    let closure = args[1].closure();
    match &receiver.datum {
        Datum::Integer(i) => {
            let from = *i;
            let to = args[0].integer();
            for x in from..=to {
                let res = closure_apply(
                    receiver.clone(),
                    &closure,
                    &vec![Object::make_integer(x)],
                    global,
                );
                if res.is_return() {
                    return res;
                }
            }
        }
        Datum::Float(f) => {
            let mut x = *f;
            let end = args[0].float();
            while x <= end {
                let res = closure_apply(
                    receiver.clone(),
                    &closure,
                    &vec![Object::make_float(x)],
                    global,
                );
                if res.is_return() {
                    return res;
                }
                x += 1.0;
            }
        }
        _ => panic!("Bad receiver in method_number_to_do: {}", receiver),
    }
    make_method_result(receiver.clone(), receiver)
}
