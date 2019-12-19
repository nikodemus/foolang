use crate::eval::Env;
use crate::objects::{Eval, Object, Vtable};

pub fn class_vtable() -> Vtable {
    Vtable::new("class Time")
}

pub fn instance_vtable() -> Vtable {
    let mut vt = Vtable::new("Time");
    vt.def("addTime:", time_add_time);
    vt.def("subTime:", time_sub_time);
    vt.def("-", time_sub);
    vt.def("+", time_add);
    vt.def("real", time_real);
    vt.def("system", time_system);
    vt.def("user", time_user);
    vt
}

fn time_sub_time(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = **args[0].time() - **receiver.time();
    Ok(env.foo.make_time(res))
}

fn time_add_time(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = **args[0].time() + **receiver.time();
    Ok(env.foo.make_time(res))
}

fn time_sub(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    args[0].send("subTime:", std::slice::from_ref(receiver), env)
}

fn time_add(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    args[0].send("addTime:", std::slice::from_ref(receiver), env)
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
