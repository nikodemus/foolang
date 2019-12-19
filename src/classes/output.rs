use crate::eval::Env;
use crate::objects::{Eval, Object, Vtable};

pub fn vtable() -> Vtable {
    let mut vt = Vtable::new("Output");
    vt.def("flush", output_flush);
    vt.def("newline", output_newline);
    vt.def("print:", output_print);
    vt.def("println:", output_println);
    vt.def("toString", output_to_string);
    vt
}

fn output_flush(receiver: &Object, _args: &[Object], _env: &Env) -> Eval {
    receiver.output().flush();
    Ok(receiver.clone())
}

fn output_newline(receiver: &Object, _args: &[Object], _env: &Env) -> Eval {
    receiver.output().write("\n");
    Ok(receiver.clone())
}

fn output_print(receiver: &Object, args: &[Object], _env: &Env) -> Eval {
    // FIXME: Type-error if not string
    receiver.output().write(args[0].string_as_str());
    Ok(receiver.clone())
}

fn output_println(receiver: &Object, args: &[Object], _env: &Env) -> Eval {
    let output = &receiver.output();
    // FIXME: Type-error if not string
    output.write(args[0].string_as_str());
    output.write("\n");
    Ok(receiver.clone())
}

fn output_to_string(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.into_string(format!("{}", receiver)))
}
