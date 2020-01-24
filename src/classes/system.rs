use std::{thread, time};

use getrandom;

use crate::eval::Env;
use crate::objects::{Eval, Object, Vtable};
use crate::unwind::Unwind;

pub fn vtable() -> Vtable {
    let mut vt = Vtable::new("System");
    vt.def("abort", system_abort);
    vt.def("clock", system_clock);
    vt.def("exit", system_exit);
    vt.def("exit:", system_exit_arg);
    vt.def("input", system_input);
    vt.def("output", system_output);
    vt.def("output:", system_output_arg);
    vt.def("random", system_random);
    vt.def("random:", system_random_arg);
    vt.def("sleep", system_sleep);
    vt.def("sleep:", system_sleep_arg);
    vt.def("window:", system_window);
    vt
}

fn system_abort(_receiver: &Object, _args: &[Object], _env: &Env) -> Eval {
    std::process::abort()
}

fn system_clock(_receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_clock())
}

fn system_exit(_receiver: &Object, _args: &[Object], _env: &Env) -> Eval {
    std::process::exit(0)
}

fn system_exit_arg(_receiver: &Object, args: &[Object], _env: &Env) -> Eval {
    std::process::exit(args[0].integer() as i32)
}

fn system_input(_receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_input("stdin", Box::new(std::io::stdin())))
}

fn system_output(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    match &receiver.system().output {
        None => Ok(env.foo.make_output("stdout", Box::new(std::io::stdout()))),
        Some(out) => Ok(out.clone()),
    }
}

fn system_output_arg(_receiver: &Object, args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_system(Some(args[0].clone())))
}

fn system_random(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    system_random_arg(receiver, &[env.foo.make_integer(32)], env)
}

fn system_random_arg(_receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let bytes = env.get("ByteArray").unwrap().send("new:", args, env)?;
    {
        let mut data = bytes.as_byte_array("in System#random internals")?.borrow_mut();
        if let Err(_) = getrandom::getrandom(&mut data) {
            return Unwind::error("Operating system could not provide random data.");
        }
    }
    Ok(bytes)
}

fn system_sleep(_receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    thread::sleep(time::Duration::from_millis(1));
    Ok(env.foo.make_boolean(true))
}

fn system_sleep_arg(_receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let period = args[0].integer();
    if period > -1 {
        let duration = time::Duration::from_millis(period as u64);
        thread::sleep(duration);
    }
    Ok(env.foo.make_boolean(true))
}

fn system_window(_receiver: &Object, args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_window(kiss3d::window::Window::new(args[0].string_as_str())))
}
