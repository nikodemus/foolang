use crate::eval::Env;
use crate::objects::{Eval, Object, Vtable};

pub fn instance_vtable() -> Vtable {
    let mut vt = Vtable::new("String");
    vt.def("appendToString:", string_append_to_string);
    vt.def("toString", string_to_string);
    vt.def("size", string_size);
    vt.def("do:", string_do);
    vt.def("at:", string_at);
    vt.def("stringEqual:", string_equal);
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

fn string_do(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    for ch in receiver.string_as_str().chars() {
        args[0].send("value:", &[env.foo.make_string(&ch.to_string())], env)?;
    }
    Ok(receiver.clone())
}

fn string_append_to_string(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let mut s = args[0].string_as_str().to_string();
    s.push_str(receiver.string_as_str());
    Ok(env.foo.into_string(s))
}

fn string_to_string(receiver: &Object, _args: &[Object], _env: &Env) -> Eval {
    Ok(receiver.clone())
}

fn string_size(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_integer(receiver.string_as_str().len() as i64))
}

fn string_at(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let data: &str = receiver.string_as_str();
    let i = (args[0].integer() - 1) as usize;
    Ok(env.foo.make_string(&data[i..i + 1]))
}

fn string_equal(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_boolean(receiver.string_as_str() == args[0].string_as_str()))
}
