use crate::eval::Env;
use crate::objects::{Eval, Object, Vtable};

pub fn instance_vtable() -> Vtable {
    let mut vt = Vtable::new("String");
    vt.def("append:", string_append);
    vt.def("appendToString:", string_append_to_string);
    vt.def("newline", string_newline);
    vt.def("toString", string_to_string);
    vt
}

pub fn class_vtable() -> Vtable {
    let mut vt = Vtable::new("class String");
    vt.def("new", class_string_new);
    vt
}

fn class_string_new(_receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_string(""))
}

fn string_append(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    args[0].send("appendToString:", &[receiver.clone()], env)
}

fn string_append_to_string(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let mut s = args[0].string_as_str().to_string();
    s.push_str(receiver.string_as_str());
    Ok(env.foo.into_string(s))
}

fn string_newline(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    let mut s = receiver.string_as_str().to_string();
    s.push_str("\n");
    Ok(env.foo.into_string(s))
}

fn string_to_string(receiver: &Object, _args: &[Object], _env: &Env) -> Eval {
    Ok(receiver.clone())
}
