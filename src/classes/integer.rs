use crate::eval::Env;
use crate::objects::{Eval, Object, Vtable};

pub fn vtable() -> Vtable {
    let vt = Vtable::new("Integer");
    vt.add_primitive_method_or_panic("asFloat", integer_as_float);
    vt.add_primitive_method_or_panic("addInteger:", integer_add_integer);
    vt.add_primitive_method_or_panic("divInteger:", integer_div_integer);
    vt.add_primitive_method_or_panic("equalInteger:", integer_equal_integer);
    vt.add_primitive_method_or_panic("greaterThanInteger:", integer_greater_than_integer);
    vt.add_primitive_method_or_panic(
        "greaterThanOrEqualInteger:",
        integer_greater_than_or_equal_integer,
    );
    vt.add_primitive_method_or_panic("lessThanInteger:", integer_less_than_integer);
    vt.add_primitive_method_or_panic("lessThanOrEqualInteger:", integer_less_than_or_equal_integer);
    vt.add_primitive_method_or_panic("mulInteger:", integer_mul_integer);
    vt.add_primitive_method_or_panic("subInteger:", integer_sub_integer);
    vt.add_primitive_method_or_panic("toString", integer_to_string);
    vt.add_primitive_method_or_panic("prefix-", integer_neg);
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
