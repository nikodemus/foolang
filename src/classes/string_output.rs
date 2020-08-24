use crate::eval::Env;
use crate::objects::{Eval, Object, Vtable};

pub fn instance_vtable() -> Vtable {
    let vt = Vtable::for_instance("StringOutput");
    vt.add_primitive_method_or_panic("writeString:", string_output_write_string);
    vt.add_primitive_method_or_panic("content", string_output_content);
    vt
}

pub fn class_vtable() -> Vtable {
    let vt = Vtable::for_class("StringOutput");
    vt.add_primitive_method_or_panic("new", class_string_output_new);
    vt
}

fn class_string_output_new(_receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_string_output())
}

fn string_output_write_string(receiver: &Object, args: &[Object], _env: &Env) -> Eval {
    receiver.string_output().write(args[0].as_str()?);
    Ok(receiver.clone())
}

fn string_output_content(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.into_string(receiver.string_output().content()))
}
