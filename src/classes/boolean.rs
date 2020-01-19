use crate::eval::Env;
use crate::objects::{Eval, Object, Vtable};

pub fn vtable() -> Vtable {
    let mut vt = Vtable::new("Boolean");
    vt.def("ifTrue:ifFalse:", boolean_if_true_if_false);
    vt
}

fn boolean_if_true_if_false(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    if receiver.boolean() {
        args[0].send("value", &[], env)
    } else {
        args[1].send("value", &[], env)
    }
}
