use std::process::Command;
use std::{thread, time};

use getrandom;

use crate::classes;
use crate::eval::Env;
use crate::objects::{Eval, Object, Vtable};
use crate::unwind::Unwind;

pub fn vtable() -> Vtable {
    let vt = Vtable::for_instance("System");
    vt.add_primitive_method_or_panic("abort", system_abort);
    vt.add_primitive_method_or_panic("clock", system_clock);
    vt.add_primitive_method_or_panic("command:", system_command);
    vt.add_primitive_method_or_panic("currentDirectory", system_current_directory);
    vt.add_primitive_method_or_panic("exit", system_exit);
    vt.add_primitive_method_or_panic("exit:", system_exit_arg);
    vt.add_primitive_method_or_panic("files", system_files);
    vt.add_primitive_method_or_panic("getenv:", system_getenv);
    vt.add_primitive_method_or_panic("input", system_input);
    vt.add_primitive_method_or_panic("isWindows", system_is_windows);
    vt.add_primitive_method_or_panic("isUnix", system_is_unix);
    vt.add_primitive_method_or_panic("output", system_output);
    vt.add_primitive_method_or_panic("output:", system_output_arg);
    vt.add_primitive_method_or_panic("random", system_random);
    vt.add_primitive_method_or_panic("random:", system_random_arg);
    vt.add_primitive_method_or_panic("sleep", system_sleep);
    vt.add_primitive_method_or_panic("sleep:", system_sleep_arg);
    vt
}

fn system_abort(_receiver: &Object, _args: &[Object], _env: &Env) -> Eval {
    // FIXME: This used to be std::process::abort(), but that started hanging
    // in Azure Pipelines tests.
    std::process::exit(1)
}

fn system_clock(_receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_clock())
}

fn system_command(_receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd").args(&["/C", args[0].as_str()?]).output()
    } else {
        Command::new("sh").args(&["-c", args[0].as_str()?]).output()
    };
    match output {
        Ok(output) => {
            let ok = output.status.success();
            let stdout = std::str::from_utf8(&output.stdout).expect("Command stdout was not UTF-8");
            let stderr = std::str::from_utf8(&output.stderr).expect("Command stderr was not UTF-8");
            env.find_global_or_unwind("Record")?.send(
                "ok:stdout:stderr:",
                &[
                    env.foo.make_boolean(ok),
                    env.foo.make_string(stdout),
                    env.foo.make_string(stderr),
                ],
                env,
            )
        }
        Err(err) => Unwind::error(&format!("Could not execute: {:?} ({})", args[0], err)),
    }
}

fn system_current_directory(_receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    classes::filepath::make_current_directory_filepath(env)
}

fn system_exit(_receiver: &Object, _args: &[Object], _env: &Env) -> Eval {
    std::process::exit(0)
}

fn system_exit_arg(_receiver: &Object, args: &[Object], _env: &Env) -> Eval {
    std::process::exit(args[0].integer() as i32)
}

fn system_files(_receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    classes::filepath::make_root_filepath(env)
}

fn system_getenv(_receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let name = args[0].as_str()?;
    match std::env::var(name) {
        Ok(s) => Ok(env.foo.into_string(s)),
        Err(std::env::VarError::NotPresent) => Ok(env.foo.make_boolean(false)),
        Err(_) => Unwind::error(&format!("Value of {} is not value UTF-8", name)),
    }
}

fn system_input(_receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_input("stdin", Box::new(std::io::stdin())))
}

fn system_is_unix(_receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_boolean(cfg!(target_family = "unix")))
}

fn system_is_windows(_receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_boolean(cfg!(target_family = "windows")))
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
