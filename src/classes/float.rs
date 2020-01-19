use crate::eval::Env;
use crate::objects::{Eval, Object, Vtable};

pub fn vtable() -> Vtable {
    let mut vt = Vtable::new("Float");
    // FUNDAMENTAL
    vt.def("addFloat:", float_add_float);
    vt.def("divFloat:", float_div_float);
    vt.def("equalFloat:", float_equal_float);
    vt.def("greaterThanFloat:", float_greater_than_float);
    vt.def("greaterThanOrEqualFloat:", float_greater_than_or_equal_float);
    vt.def("lessThanFloat:", float_less_than_float);
    vt.def("lessThanOrEqualFloat:", float_less_than_or_equal_float);
    vt.def("mulFloat:", float_mul_float);
    vt.def("prefix-", float_neg);
    vt.def("subFloat:", float_sub_float);
    vt.def("toString", float_to_string);
    vt.def("asInteger", float_as_integer);
    vt
}

// FUNDAMENTAL METHODS

fn float_as_integer(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_integer(receiver.float().round() as i64))
}

fn float_add_float(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = args[0].float() + receiver.float();
    Ok(env.foo.make_float(res))
}

fn float_div_float(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = args[0].float() / receiver.float();
    Ok(env.foo.make_float(res))
}

fn float_mul_float(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = args[0].float() * receiver.float();
    Ok(env.foo.make_float(res))
}

fn float_sub_float(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = args[0].float() - receiver.float();
    Ok(env.foo.make_float(res))
}

fn float_equal_float(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = receiver.float() == args[0].float();
    Ok(env.foo.make_boolean(res))
}

fn float_greater_than_float(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = receiver.float() > args[0].float();
    Ok(env.foo.make_boolean(res))
}

fn float_greater_than_or_equal_float(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = receiver.float() >= args[0].float();
    Ok(env.foo.make_boolean(res))
}

fn float_less_than_float(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = receiver.float() < args[0].float();
    Ok(env.foo.make_boolean(res))
}

fn float_less_than_or_equal_float(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = receiver.float() <= args[0].float();
    Ok(env.foo.make_boolean(res))
}

fn float_neg(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_float(-receiver.float()))
}

fn float_to_string(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_string(&receiver.float().to_string()))
}
