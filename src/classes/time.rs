use crate::eval::Env;
use crate::objects::{Eval, Object, Vtable};
use crate::time::TimeInfo;

pub fn class_vtable() -> Vtable {
    let vt = Vtable::for_class("Time");
    vt.add_primitive_method_or_panic("user:system:real:", class_time_user_system_real_);
    vt
}

pub fn instance_vtable() -> Vtable {
    let vt = Vtable::for_instance("Time");
    vt.add_primitive_method_or_panic("real", time_real);
    vt.add_primitive_method_or_panic("system", time_system);
    vt.add_primitive_method_or_panic("user", time_user);
    vt
}

fn class_time_user_system_real_(_receiver: &Object, args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_time(TimeInfo {
        user: args[0].float(),
        system: args[1].float(),
        real: args[2].float(),
    }))
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
