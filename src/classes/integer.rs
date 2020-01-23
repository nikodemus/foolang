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

fn integer_to_string(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_string(&receiver.integer().to_string()))
}
