use crate::eval;
use crate::objects2::{Builtins, Eval, Object, Vtable};

pub fn vtable() -> Vtable {
    let mut vt = Vtable::new("Closure");
    // FUNDAMENTAL
    vt.def("value", closure_apply);
    vt.def("value:", closure_apply);
    vt.def("value:value:", closure_apply);
    vt.def("value:value:value:", closure_apply);
    vt
}

// FUNDAMENTAL METHODS

fn closure_apply(receiver: &Object, args: &[&Object], builtins: &Builtins) -> Eval {
    eval::apply(None, receiver.closure_ref(), args, builtins)
}
