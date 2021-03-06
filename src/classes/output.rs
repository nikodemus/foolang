use crate::eval::Env;
use crate::objects::{Eval, Object, Vtable};

pub fn class_vtable() -> Vtable {
    let vt = Vtable::for_class("Output");
    vt.add_primitive_method_or_panic("debug", class_output_debug);
    vt
}

pub fn instance_vtable() -> Vtable {
    let vt = Vtable::for_instance("Output");
    vt.add_primitive_method_or_panic("flush", output_flush);
    vt.add_primitive_method_or_panic("writeString:", output_write_string);
    vt.add_primitive_method_or_panic("toString", output_to_string);
    vt
}

fn output_flush(receiver: &Object, _args: &[Object], _env: &Env) -> Eval {
    receiver.output().flush();
    Ok(receiver.clone())
}

fn class_output_debug(_receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_output("debug", Box::new(std::io::stderr())))
}

fn output_write_string(receiver: &Object, args: &[Object], _env: &Env) -> Eval {
    // FIXME: Type-error if not string
    receiver.output().write(args[0].string_as_str());
    Ok(receiver.clone())
}

fn output_to_string(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.into_string(format!("{}", receiver)))
}
