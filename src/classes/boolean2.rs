use crate::objects2::{Eval, Foolang, Object, Vtable};

pub fn vtable() -> Vtable {
    let mut vt = Vtable::new("Boolean");
    vt.def("and:", boolean_and);
    vt.def("ifFalse:", boolean_if_false);
    vt.def("ifTrue:", boolean_if_true);
    vt.def("ifTrue:ifFalse:", boolean_if_true_if_false);
    vt.def("not", boolean_not);
    vt.def("or:", boolean_or);
    vt.def("toString", boolean_to_string);
    vt
}

fn boolean_and(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    if receiver.boolean() && args[0].boolean() {
        Ok(foo.make_boolean(true))
    } else {
        Ok(foo.make_boolean(false))
    }
}

fn boolean_if_true(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    if receiver.boolean() {
        args[0].send("value", &[], foo)
    } else {
        Ok(receiver.clone())
    }
}

fn boolean_if_false(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    if receiver.boolean() {
        Ok(receiver.clone())
    } else {
        args[0].send("value", &[], foo)
    }
}

fn boolean_if_true_if_false(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    if receiver.boolean() {
        args[0].send("value", &[], foo)
    } else {
        args[1].send("value", &[], foo)
    }
}

fn boolean_not(receiver: &Object, _args: &[Object], foo: &Foolang) -> Eval {
    Ok(foo.make_boolean(!receiver.boolean()))
}

fn boolean_or(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    if receiver.boolean() || args[0].boolean() {
        Ok(foo.make_boolean(true))
    } else {
        Ok(foo.make_boolean(false))
    }
}

fn boolean_to_string(receiver: &Object, _args: &[Object], foo: &Foolang) -> Eval {
    if receiver.boolean() {
        Ok(foo.make_string("True"))
    } else {
        Ok(foo.make_string("False"))
    }
}
