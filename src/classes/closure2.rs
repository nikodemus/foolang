use crate::eval;
use crate::objects2::{Builtins, Eval, Object, Vtable};

pub fn vtable() -> Vtable {
    let mut vt = Vtable::new("Closure");
    // FUNDAMENTAL
    vt.def("value", closure_apply);
    vt.def("value:", closure_apply);
    vt.def("value:value:", closure_apply);
    vt.def("value:value:value:", closure_apply);
    vt.def("whileTrue:", closure_while_true_closure);
    vt.def("whileFalse:", closure_while_false_closure);
    vt.def("whileTrue", closure_while_true);
    vt.def("whileFalse", closure_while_false);
    vt
}

// FUNDAMENTAL METHODS

fn closure_apply(receiver: &Object, args: &[&Object], builtins: &Builtins) -> Eval {
    eval::apply(None, receiver.closure_ref(), args, builtins)
}

fn closure_while_true(receiver: &Object, _args: &[&Object], builtins: &Builtins) -> Eval {
    let t = builtins.make_boolean(true);
    loop {
        let r = eval::apply(None, receiver.closure_ref(), &[], builtins)?;
        if t != r {
            return Ok(r);
        }
    }
}

fn closure_while_true_closure(receiver: &Object, args: &[&Object], builtins: &Builtins) -> Eval {
    let t = builtins.make_boolean(true);
    // FIXME: Should initialize to nil
    let mut r = builtins.make_boolean(false);
    loop {
        if eval::apply(None, args[0].closure_ref(), &[], builtins)? == t {
            r = eval::apply(None, receiver.closure_ref(), &[], builtins)?;
        } else {
            return Ok(r);
        }
    }
}

fn closure_while_false(receiver: &Object, args: &[&Object], builtins: &Builtins) -> Eval {
    let f = builtins.make_boolean(false);
    loop {
        let r = eval::apply(None, receiver.closure_ref(), &[], builtins)?;
        if r != f {
            return Ok(r);
        }
    }
}

fn closure_while_false_closure(receiver: &Object, args: &[&Object], builtins: &Builtins) -> Eval {
    let f = builtins.make_boolean(false);
    // FIXME: Should initialize to nil
    let mut r = builtins.make_boolean(false);
    loop {
        if eval::apply(None, args[0].closure_ref(), &[], builtins)? == f {
            r = eval::apply(None, receiver.closure_ref(), &[], builtins)?;
        } else {
            return Ok(r);
        }
    }
}
