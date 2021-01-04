use crate::eval::Env;
use crate::objects::{Eval, Object, Vtable};

pub fn class_vtable() -> Vtable {
    Vtable::for_class("Time")
}

pub fn instance_vtable() -> Vtable {
    let vt = Vtable::for_instance("Time");
    vt.add_primitive_method_or_panic("real", time_real);
    vt.add_primitive_method_or_panic("system", time_system);
    vt.add_primitive_method_or_panic("user", time_user);
    vt
}

fn time_real(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_float(receiver.time().real))
}

fn time_system(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_float(receiver.time().system))
}

fn time_user(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_float(receiver.time().user))
}
