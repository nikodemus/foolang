use crate::eval::Env;
use crate::objects::{Eval, Object, Vtable};
use crate::time::TimeInfo;

pub fn class_vtable() -> Vtable {
    Vtable::new("class Clock")
}

pub fn instance_vtable() -> Vtable {
    let mut vt = Vtable::new("Clock");
    vt.def("time", clock_time);
    vt.def("toString", clock_to_string);
    vt
}

fn clock_time(_receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_time(TimeInfo::now()))
}

fn clock_to_string(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.into_string(format!("{}", receiver)))
}
