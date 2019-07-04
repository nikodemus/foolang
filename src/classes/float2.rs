use crate::objects2::{Builtins, Object, Value, Vtable};

use std::borrow::ToOwned;

pub fn vtable() -> Vtable {
    let mut vt = Vtable::new("Float");
    // FUNDAMENTAL
    vt.def("asFloat", float_as_float);
    vt.def("asInteger", float_as_integer);
    vt.def("addInteger:", float_add_integer);
    vt.def("divInteger:", float_div_integer);
    vt.def("mulInteger:", float_mul_integer);
    vt.def("subInteger:", float_sub_integer);
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

fn float_as_float(receiver: &Object, _args: &[&Object], _builtins: &Builtins) -> Value {
    Ok(receiver.to_owned())
}

fn float_as_integer(receiver: &Object, _args: &[&Object], builtins: &Builtins) -> Value {
    Ok(builtins.make_integer(receiver.float().round() as i64))
}

fn float_add_float(receiver: &Object, args: &[&Object], builtins: &Builtins) -> Value {
    let res = args[0].float() + receiver.float();
    Ok(builtins.make_float(res))
}

fn float_div_float(receiver: &Object, args: &[&Object], builtins: &Builtins) -> Value {
    let res = args[0].float() / receiver.float();
    Ok(builtins.make_float(res))
}

fn float_mul_float(receiver: &Object, args: &[&Object], builtins: &Builtins) -> Value {
    let res = args[0].float() * receiver.float();
    Ok(builtins.make_float(res))
}

fn float_sub_float(receiver: &Object, args: &[&Object], builtins: &Builtins) -> Value {
    let res = args[0].float() - receiver.float();
    Ok(builtins.make_float(res))
}

// DERIVED METHODS

fn float_add(receiver: &Object, args: &[&Object], builtins: &Builtins) -> Value {
    args[0].send("addFloat:", &[receiver], builtins)
}

fn float_div(receiver: &Object, args: &[&Object], builtins: &Builtins) -> Value {
    args[0].send("divFloat:", &[receiver], builtins)
}

fn float_mul(receiver: &Object, args: &[&Object], builtins: &Builtins) -> Value {
    args[0].send("mulFloat:", &[receiver], builtins)
}

fn float_sub(receiver: &Object, args: &[&Object], builtins: &Builtins) -> Value {
    args[0].send("subFloat:", &[receiver], builtins)
}

fn float_add_integer(receiver: &Object, args: &[&Object], builtins: &Builtins) -> Value {
    let float = args[0].send("asFloat", &[], builtins)?;
    receiver.send("addFloat:", &[&float], builtins)
}

fn float_div_integer(receiver: &Object, args: &[&Object], builtins: &Builtins) -> Value {
    let float = args[0].send("asFloat", &[], builtins)?;
    receiver.send("divFloat:", &[&float], builtins)
}

fn float_mul_integer(receiver: &Object, args: &[&Object], builtins: &Builtins) -> Value {
    let float = args[0].send("asFloat", &[], builtins)?;
    receiver.send("mulFloat:", &[&float], builtins)
}

fn float_sub_integer(receiver: &Object, args: &[&Object], builtins: &Builtins) -> Value {
    let float = args[0].send("asFloat", &[], builtins)?;
    receiver.send("subFloat:", &[&float], builtins)
}
