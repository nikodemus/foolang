use crate::eval::Env;
use crate::objects::{Eval, Object, Vtable};

pub fn vtable() -> Vtable {
    let mut vt = Vtable::new("Boolean");
    vt.def("and:", boolean_and);
    vt.def("ifFalse:", boolean_if_false);
    vt.def("ifTrue:", boolean_if_true);
    vt.def("ifTrue:ifFalse:", boolean_if_true_if_false);
    vt.def("not", boolean_not);
    vt.def("or:", boolean_or);
    vt.def("toString", boolean_to_string);
    vt
}

fn boolean_and(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    if receiver.boolean() && args[0].boolean() {
        Ok(env.foo.make_boolean(true))
    } else {
        Ok(env.foo.make_boolean(false))
    }
}

fn boolean_if_true(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    if receiver.boolean() {
        args[0].send("value", &[], env)
    } else {
        Ok(receiver.clone())
    }
}

fn boolean_if_false(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    if receiver.boolean() {
        Ok(receiver.clone())
    } else {
        args[0].send("value", &[], env)
    }
}

fn boolean_if_true_if_false(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    if receiver.boolean() {
        args[0].send("value", &[], env)
    } else {
        args[1].send("value", &[], env)
    }
}

fn boolean_not(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_boolean(!receiver.boolean()))
}

fn boolean_or(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    if receiver.boolean() || args[0].boolean() {
        Ok(env.foo.make_boolean(true))
    } else {
        Ok(env.foo.make_boolean(false))
    }
}

fn boolean_to_string(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    if receiver.boolean() {
        Ok(env.foo.make_string("True"))
    } else {
        Ok(env.foo.make_string("False"))
    }
}
