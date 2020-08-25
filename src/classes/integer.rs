use crate::eval::Env;
use crate::objects::{Eval, Object, Vtable};

pub fn vtable() -> Vtable {
    let vt = Vtable::for_instance("Integer");
    vt.add_primitive_method_or_panic("asFloat", integer_as_float);
    vt.add_primitive_method_or_panic("integerAdd:", integer_integer_add);
    vt.add_primitive_method_or_panic("integerDiv:", integer_integer_div);
    vt.add_primitive_method_or_panic("integerEq:", integer_integer_eq);
    vt.add_primitive_method_or_panic("integerGt:", integer_integer_gt);
    vt.add_primitive_method_or_panic("integerGte:", integer_integer_gte);
    vt.add_primitive_method_or_panic("integerLt:", integer_integer_lt);
    vt.add_primitive_method_or_panic("integerLte:", integer_integer_lte);
    vt.add_primitive_method_or_panic("integerMul:", integer_integer_mul);
    vt.add_primitive_method_or_panic("integerSub:", integer_integer_sub);
    vt.add_primitive_method_or_panic("toString", integer_to_string);
    vt.add_primitive_method_or_panic("prefix-", integer_neg);
    vt
}

fn integer_as_float(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_float(receiver.integer() as f64))
}

fn integer_integer_add(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = receiver.integer() + args[0].integer();
    Ok(env.foo.make_integer(res))
}

fn integer_integer_div(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let div = args[0].integer();
    if div == 0 {
        match env.get("DivideByZero") {
            None => panic!("DivideByZero not defined"),
            Some(obj) => return obj.send("raise:", std::slice::from_ref(receiver), env),
        }
    }
    Ok(env.foo.make_integer(receiver.integer() / div))
}

fn integer_integer_eq(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = receiver.integer() == args[0].integer();
    Ok(env.foo.make_boolean(res))
}

fn integer_integer_gt(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = receiver.integer() > args[0].integer();
    Ok(env.foo.make_boolean(res))
}

fn integer_integer_gte(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = receiver.integer() >= args[0].integer();
    Ok(env.foo.make_boolean(res))
}

fn integer_integer_lt(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = receiver.integer() < args[0].integer();
    Ok(env.foo.make_boolean(res))
}

fn integer_integer_lte(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = receiver.integer() <= args[0].integer();
    Ok(env.foo.make_boolean(res))
}

fn integer_integer_mul(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = receiver.integer() * args[0].integer();
    Ok(env.foo.make_integer(res))
}

fn integer_integer_sub(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = receiver.integer() - args[0].integer();
    Ok(env.foo.make_integer(res))
}

fn integer_neg(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_integer(-receiver.integer()))
}

fn integer_to_string(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_string(&receiver.integer().to_string()))
}
