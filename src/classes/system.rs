use std::{thread, time};

use crate::objects::{Eval, Foolang, Object, Vtable};

pub fn vtable() -> Vtable {
    let mut vt = Vtable::new("System");
    vt.def("abort", system_abort);
    vt.def("clock", system_clock);
    vt.def("exit", system_exit);
    vt.def("exit:", system_exit_arg);
    vt.def("input", system_input);
    vt.def("output", system_output);
    vt.def("sleep", system_sleep);
    vt.def("sleep:", system_sleep_arg);
    vt
}

fn system_abort(_receiver: &Object, _args: &[Object], _foo: &Foolang) -> Eval {
    std::process::abort()
}

fn system_clock(_receiver: &Object, _args: &[Object], foo: &Foolang) -> Eval {
    Ok(foo.make_clock())
}

fn system_exit(_receiver: &Object, _args: &[Object], _foo: &Foolang) -> Eval {
    std::process::exit(0)
}

fn system_exit_arg(_receiver: &Object, args: &[Object], _foo: &Foolang) -> Eval {
    std::process::exit(args[0].integer() as i32)
}

fn system_input(_receiver: &Object, _args: &[Object], foo: &Foolang) -> Eval {
    Ok(foo.make_input("stdin", Box::new(std::io::stdin())))
}

fn system_output(_receiver: &Object, _args: &[Object], foo: &Foolang) -> Eval {
    Ok(foo.make_output("stdout", Box::new(std::io::stdout())))
}

fn system_sleep(_receiver: &Object, _args: &[Object], foo: &Foolang) -> Eval {
    thread::sleep(time::Duration::from_millis(1));
    Ok(foo.make_boolean(true))
}

fn system_sleep_arg(_receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let period = args[0].integer();
    if period > -1 {
        let duration = time::Duration::from_millis(period as u64);
        thread::sleep(duration);
    }
    Ok(foo.make_boolean(true))
}
