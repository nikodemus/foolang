use crate::eval::Env;
use crate::objects::{Eval, Object, Vtable};
use crate::unwind::Unwind;

use std::str::FromStr;

pub fn class_vtable() -> Vtable {
    let vt = Vtable::new("Float class");
    vt.add_primitive_method_or_panic("parse:", float_class_parse_);
    vt
}

fn float_class_parse_(_receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let s = args[0].as_str()?;
    match f64::from_str(s) {
        Ok(f) => Ok(env.foo.make_float(f)),
        Err(_) => Unwind::error(&format!("Cannot parse as float: {}", s)),
    }
}

pub fn instance_vtable() -> Vtable {
    let vt = Vtable::new("Float");
    vt.add_primitive_method_or_panic("floatAdd:", float_float_add);
    vt.add_primitive_method_or_panic("floatDiv:", float_float_div);
    vt.add_primitive_method_or_panic("floatEq:", float_float_eq);
    vt.add_primitive_method_or_panic("floatGt:", float_float_gt);
    vt.add_primitive_method_or_panic("floatGte:", float_float_gte);
    vt.add_primitive_method_or_panic("floatLt:", float_float_lt);
    vt.add_primitive_method_or_panic("floatLte:", float_float_lte);
    vt.add_primitive_method_or_panic("floatMul:", float_float_mul);
    vt.add_primitive_method_or_panic("floatSub:", float_float_sub);
    vt.add_primitive_method_or_panic("isFinite", float_is_finite);
    vt.add_primitive_method_or_panic("isInfinite", float_is_infinite);
    vt.add_primitive_method_or_panic("prefix-", float_neg);
    vt.add_primitive_method_or_panic("sqrt", float_sqrt);
    vt.add_primitive_method_or_panic("truncate", float_truncate);
    vt.add_primitive_method_or_panic("round", float_round);
    vt
}

// FUNDAMENTAL METHODS

fn float_round(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_integer(receiver.float().round() as i64))
}

fn float_is_infinite(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_boolean(receiver.float().is_infinite()))
}

fn float_is_finite(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_boolean(receiver.float().is_finite()))
}

fn float_truncate(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_integer(receiver.float().trunc() as i64))
}

fn float_float_add(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = args[0].float() + receiver.float();
    Ok(env.foo.make_float(res))
}

fn float_float_div(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = receiver.float() / args[0].float();
    Ok(env.foo.make_float(res))
}

fn float_float_mul(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = receiver.float() * args[0].float();
    Ok(env.foo.make_float(res))
}

fn float_float_sub(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = receiver.float() - args[0].float();
    Ok(env.foo.make_float(res))
}

fn float_float_eq(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = receiver.float() == args[0].float();
    Ok(env.foo.make_boolean(res))
}

fn float_float_gt(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = receiver.float() > args[0].float();
    Ok(env.foo.make_boolean(res))
}

fn float_float_gte(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = receiver.float() >= args[0].float();
    Ok(env.foo.make_boolean(res))
}

fn float_float_lt(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = receiver.float() < args[0].float();
    Ok(env.foo.make_boolean(res))
}

fn float_float_lte(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = receiver.float() <= args[0].float();
    Ok(env.foo.make_boolean(res))
}

fn float_sqrt(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_float(receiver.float().sqrt()))
}

fn float_neg(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_float(-receiver.float()))
}
