use crate::eval::Env;
use crate::objects::{Eval, Object, Vtable};

pub fn vtable() -> Vtable {
    let mut vt = Vtable::new("Integer");
    vt.def("asFloat", integer_as_float);
    vt.def("addInteger:", integer_add_integer);
    vt.def("divInteger:", integer_div_integer);
    vt.def("equalInteger:", integer_equal_integer);
    vt.def("greaterThanInteger:", integer_greater_than_integer);
    vt.def("greaterThanOrEqualInteger:", integer_greater_than_or_equal_integer);
    vt.def("lessThanInteger:", integer_less_than_integer);
    vt.def("lessThanOrEqualInteger:", integer_less_than_or_equal_integer);
    vt.def("mulInteger:", integer_mul_integer);
    vt.def("subInteger:", integer_sub_integer);
    vt.def("toString", integer_to_string);
    vt.def("prefix-", integer_neg);
    vt.def("to:", integer_to);
    vt.def("to:do:", integer_to_do);
    vt.def("times:", integer_times);
    vt
}

fn integer_as_float(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_float(receiver.integer() as f64))
}

fn integer_add_integer(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = args[0].integer() + receiver.integer();
    Ok(env.foo.make_integer(res))
}

fn integer_div_integer(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = args[0].integer() / receiver.integer();
    Ok(env.foo.make_integer(res))
}

fn integer_equal_integer(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = receiver.integer() == args[0].integer();
    Ok(env.foo.make_boolean(res))
}

fn integer_greater_than_integer(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = receiver.integer() > args[0].integer();
    Ok(env.foo.make_boolean(res))
}

fn integer_greater_than_or_equal_integer(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = receiver.integer() >= args[0].integer();
    Ok(env.foo.make_boolean(res))
}

fn integer_less_than_integer(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = receiver.integer() < args[0].integer();
    Ok(env.foo.make_boolean(res))
}

fn integer_less_than_or_equal_integer(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = receiver.integer() <= args[0].integer();
    Ok(env.foo.make_boolean(res))
}

fn integer_mul_integer(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = args[0].integer() * receiver.integer();
    Ok(env.foo.make_integer(res))
}

fn integer_sub_integer(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = args[0].integer() - receiver.integer();
    Ok(env.foo.make_integer(res))
}

fn integer_neg(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_integer(-receiver.integer()))
}

fn integer_times(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let mut start = receiver.integer();
    let block = args[0].clone();
    while start > 0 {
        block.send("value", &[], env)?;
        start -= 1;
    }
    Ok(receiver.clone())
}

fn integer_to(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let start = receiver.integer();
    // FIXME: Panics if argument is not an integer
    let end = args[0].integer();
    Ok(env.foo.make_interval(start, end))
}

fn integer_to_do(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let start = receiver.integer();
    let end = args[0].integer();
    let block = args[1].clone();
    let mut i = start;
    let step = if start < end {
        1
    } else {
        -1
    };
    loop {
        block.send("value:", &[env.foo.make_integer(i)], env)?;
        if i == end {
            break;
        }
        i += step;
    }
    Ok(receiver.clone())
}

fn integer_to_string(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_string(&receiver.integer().to_string()))
}
