use crate::eval::Env;
use crate::objects::{Eval, Object, Vtable};

pub fn vtable() -> Vtable {
    let vt = Vtable::for_instance("Input");
    vt.add_primitive_method_or_panic("readline", input_readline);
    vt
}

fn input_readline(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    match receiver.input().readline() {
        Some(line) => Ok(env.foo.into_string(line)),
        // FIXME: Nil would make more sense, or a specific EOF object
        None => Ok(env.foo.make_boolean(false)),
    }
}
