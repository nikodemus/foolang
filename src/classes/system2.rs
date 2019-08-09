use crate::objects2::{Eval, Foolang, Object, Vtable};

pub fn vtable() -> Vtable {
    let mut vt = Vtable::new("System");
    vt.def("abort", system_abort);
    vt.def("exit", system_exit);
    vt.def("exit:", system_exit_arg);
    vt.def("input", system_input);
    vt.def("output", system_output);
    vt
}

fn system_abort(_receiver: &Object, _args: &[Object], _foo: &Foolang) -> Eval {
    std::process::abort()
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
