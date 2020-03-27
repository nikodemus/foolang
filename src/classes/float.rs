use crate::eval::Env;
use crate::objects::{Eval, Object, Vtable};

pub fn vtable() -> Vtable {
    let vt = Vtable::new("Float");
    // FUNDAMENTAL
    vt.add_primitive_method_or_panic("addFloat:", float_add_float);
    vt.add_primitive_method_or_panic("divFloat:", float_div_float);
    vt.add_primitive_method_or_panic("equalFloat:", float_equal_float);
    vt.add_primitive_method_or_panic("greaterThanFloat:", float_greater_than_float);
    vt.add_primitive_method_or_panic("greaterThanOrEqualFloat:", float_greater_than_or_equal_float);
    vt.add_primitive_method_or_panic("lessThanFloat:", float_less_than_float);
    vt.add_primitive_method_or_panic("lessThanOrEqualFloat:", float_less_than_or_equal_float);
    vt.add_primitive_method_or_panic("mulFloat:", float_mul_float);
    vt.add_primitive_method_or_panic("prefix-", float_neg);
    vt.add_primitive_method_or_panic("subFloat:", float_sub_float);
    vt.add_primitive_method_or_panic("asInteger", float_as_integer);
    vt.add_primitive_method_or_panic("sqrt", float_sqrt);
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

fn float_sqrt(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_float(receiver.float().sqrt()))
}
