use crate::eval::Env;
use crate::objects::{Eval, Object, Vtable};
use crate::unwind::Unwind;

pub fn vtable() -> Vtable {
    let mut vt = Vtable::new("Closure");
    // FUNDAMENTAL
    vt.def("onError:", closure_on_error);
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

fn closure_apply(receiver: &Object, args: &[Object], _env: &Env) -> Eval {
    receiver.closure_ref().apply(None, args)
}

fn closure_on_error(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = receiver.closure_ref().apply(None, args);
    if let Err(Unwind::Exception(error, loc)) = res {
        args[0].send(
            "value:",
            &[env.foo.into_string(error.what()), env.foo.into_string(loc.context())],
            env,
        )
    } else {
        res
    }
}

fn closure_while_true(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    let t = env.foo.make_boolean(true);
    loop {
        let r = receiver.closure_ref().apply(None, &[])?;
        if t != r {
            return Ok(r);
        }
    }
}

fn closure_while_true_closure(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let t = env.foo.make_boolean(true);
    // FIXME: Should initialize to nil
    let mut r = env.foo.make_boolean(false);
    loop {
        if receiver.closure_ref().apply(None, &[])? == t {
            r = args[0].closure_ref().apply(None, &[])?
        } else {
            return Ok(r);
        }
    }
}

fn closure_while_false(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    let f = env.foo.make_boolean(false);
    loop {
        let r = receiver.closure_ref().apply(None, &[])?;
        if r != f {
            return Ok(r);
        }
    }
}

fn closure_while_false_closure(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let f = env.foo.make_boolean(false);
    // FIXME: Should initialize to nil
    let mut r = env.foo.make_boolean(false);
    loop {
        if receiver.closure_ref().apply(None, &[])? == f {
            r = args[0].closure_ref().apply(None, &[])?;
        } else {
            return Ok(r);
        }
    }
}
