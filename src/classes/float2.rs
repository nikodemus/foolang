use crate::objects2::{Eval, Foolang, Object, Vtable};

pub fn vtable() -> Vtable {
    let mut vt = Vtable::new("Float");
    // FUNDAMENTAL
    vt.def("asFloat", float_as_float);
    vt.def("asInteger", float_as_integer);
    vt.def("addInteger:", float_add_integer);
    vt.def("divInteger:", float_div_integer);
    vt.def("mulInteger:", float_mul_integer);
    vt.def("subInteger:", float_sub_integer);
    vt.def("prefix-", float_neg);
    // DERIVED
    vt.def("+", float_add);
    vt.def("/", float_div);
    vt.def("*", float_mul);
    vt.def("-", float_sub);
    vt.def("addFloat:", float_add_float);
    vt.def("divFloat:", float_div_float);
    vt.def("mulFloat:", float_mul_float);
    vt.def("subFloat:", float_sub_float);
    vt
}

// FUNDAMENTAL METHODS

fn float_as_float(receiver: &Object, _args: &[Object], _foo: &Foolang) -> Eval {
    Ok(receiver.clone())
}

fn float_as_integer(receiver: &Object, _args: &[Object], foo: &Foolang) -> Eval {
    Ok(foo.make_integer(receiver.float().round() as i64))
}

fn float_add_float(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let res = args[0].float() + receiver.float();
    Ok(foo.make_float(res))
}

fn float_div_float(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let res = args[0].float() / receiver.float();
    Ok(foo.make_float(res))
}

fn float_mul_float(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let res = args[0].float() * receiver.float();
    Ok(foo.make_float(res))
}

fn float_sub_float(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let res = args[0].float() - receiver.float();
    Ok(foo.make_float(res))
}

fn float_neg(receiver: &Object, _args: &[Object], foo: &Foolang) -> Eval {
    Ok(foo.make_float(-receiver.float()))
}

// DERIVED METHODS

fn float_add(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    args[0].send("addFloat:", &[receiver.clone()], foo)
}

fn float_div(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    args[0].send("divFloat:", &[receiver.clone()], foo)
}

fn float_mul(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    args[0].send("mulFloat:", &[receiver.clone()], foo)
}

fn float_sub(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    args[0].send("subFloat:", &[receiver.clone()], foo)
}

fn float_add_integer(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let float = args[0].send("asFloat", &[], foo)?;
    receiver.send("addFloat:", &[float], foo)
}

fn float_div_integer(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let float = args[0].send("asFloat", &[], foo)?;
    receiver.send("divFloat:", &[float], foo)
}

fn float_mul_integer(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let float = args[0].send("asFloat", &[], foo)?;
    receiver.send("mulFloat:", &[float], foo)
}

fn float_sub_integer(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let float = args[0].send("asFloat", &[], foo)?;
    receiver.send("subFloat:", &[float], foo)
}
