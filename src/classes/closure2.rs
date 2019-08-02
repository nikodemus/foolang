use crate::eval;
use crate::objects2::{Eval, Foolang, Object, Vtable};

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

fn closure_apply(receiver: &Object, args: &[&Object], foo: &Foolang) -> Eval {
    eval::apply(None, receiver.closure_ref(), args, foo)
}

fn closure_while_true(receiver: &Object, _args: &[&Object], foo: &Foolang) -> Eval {
    let t = foo.make_boolean(true);
    loop {
        let r = eval::apply(None, receiver.closure_ref(), &[], foo)?;
        if t != r {
            return Ok(r);
        }
    }
}

fn closure_while_true_closure(receiver: &Object, args: &[&Object], foo: &Foolang) -> Eval {
    let t = foo.make_boolean(true);
    // FIXME: Should initialize to nil
    let mut r = foo.make_boolean(false);
    loop {
        if eval::apply(None, args[0].closure_ref(), &[], foo)? == t {
            r = eval::apply(None, receiver.closure_ref(), &[], foo)?;
        } else {
            return Ok(r);
        }
    }
}

fn closure_while_false(receiver: &Object, args: &[&Object], foo: &Foolang) -> Eval {
    let f = foo.make_boolean(false);
    loop {
        let r = eval::apply(None, receiver.closure_ref(), &[], foo)?;
        if r != f {
            return Ok(r);
        }
    }
}

fn closure_while_false_closure(receiver: &Object, args: &[&Object], foo: &Foolang) -> Eval {
    let f = foo.make_boolean(false);
    // FIXME: Should initialize to nil
    let mut r = foo.make_boolean(false);
    loop {
        if eval::apply(None, args[0].closure_ref(), &[], foo)? == f {
            r = eval::apply(None, receiver.closure_ref(), &[], foo)?;
        } else {
            return Ok(r);
        }
    }
}
