use crate::eval::Env;
use crate::objects::{Eval, Object, Vtable};
use crate::unwind::Unwind;

pub fn vtable() -> Vtable {
    let vt = Vtable::new("Closure");
    // FUNDAMENTAL
    vt.add_primitive_method_or_panic("apply:", closure_apply_array);
    vt.add_primitive_method_or_panic("onError:", closure_on_error);
    vt.add_primitive_method_or_panic("finally:", closure_finally);
    vt.add_primitive_method_or_panic("value", closure_apply_values);
    vt.add_primitive_method_or_panic("value:", closure_apply_values);
    vt.add_primitive_method_or_panic("value:value:", closure_apply_values);
    vt.add_primitive_method_or_panic("value:value:value:", closure_apply_values);
    vt.add_primitive_method_or_panic("whileTrue:", closure_while_true);
    vt
}

// FUNDAMENTAL METHODS

fn closure_apply_array(receiver: &Object, args: &[Object], _env: &Env) -> Eval {
    let array = args[0].as_array("Closure#apply:")?.borrow();
    receiver.closure_ref().apply(None, &array)
}

fn closure_apply_values(receiver: &Object, args: &[Object], _env: &Env) -> Eval {
    receiver.closure_ref().apply(None, args)
}

fn closure_finally(receiver: &Object, args: &[Object], _env: &Env) -> Eval {
    let res = receiver.closure_ref().apply(None, &[]);
    args[0].closure_ref().apply(None, &[])?;
    res
}

fn closure_on_error(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let res = receiver.closure_ref().apply(None, &[]);
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

fn closure_while_true(receiver: &Object, args: &[Object], env: &Env) -> Eval {
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
