use crate::objects2::{Builtins, Eval, Object, Vtable};

pub fn vtable() -> Vtable {
    let mut vt = Vtable::new("Integer");
    // FUNDAMENTAL
    vt.def("asFloat", integer_as_float);
    vt.def("asInteger", integer_as_integer);
    vt.def("addInteger:", integer_add_integer);
    vt.def("divInteger:", integer_div_integer);
    vt.def("mulInteger:", integer_mul_integer);
    vt.def("subInteger:", integer_sub_integer);
    vt.def("toString", integer_to_string);
    // INCIDENTAL
    vt.def("gcd:", integer_gcd);
    // DERIVED
    vt.def("+", integer_add);
    vt.def("/", integer_div);
    vt.def("*", integer_mul);
    vt.def("-", integer_sub);
    vt.def("addFloat:", integer_add_float);
    vt.def("divFloat:", integer_div_float);
    vt.def("mulFloat:", integer_mul_float);
    vt.def("subFloat:", integer_sub_float);
    vt
}

// FUNDAMENTAL METHODS

fn integer_as_integer(receiver: &Object, _args: &[&Object], _builtins: &Builtins) -> Eval {
    Ok(receiver.to_owned())
}

fn integer_as_float(receiver: &Object, _args: &[&Object], builtins: &Builtins) -> Eval {
    Ok(builtins.make_float(receiver.integer() as f64))
}

fn integer_add_integer(receiver: &Object, args: &[&Object], builtins: &Builtins) -> Eval {
    let res = args[0].integer() + receiver.integer();
    Ok(builtins.make_integer(res))
}

fn integer_div_integer(receiver: &Object, args: &[&Object], builtins: &Builtins) -> Eval {
    let res = args[0].integer() / receiver.integer();
    Ok(builtins.make_integer(res))
}

fn integer_mul_integer(receiver: &Object, args: &[&Object], builtins: &Builtins) -> Eval {
    let res = args[0].integer() * receiver.integer();
    Ok(builtins.make_integer(res))
}

fn integer_sub_integer(receiver: &Object, args: &[&Object], builtins: &Builtins) -> Eval {
    let res = args[0].integer() - receiver.integer();
    Ok(builtins.make_integer(res))
}

fn integer_gcd(receiver: &Object, args: &[&Object], builtins: &Builtins) -> Eval {
    // FIXME: Panics if argument is not an integer.
    let res = num::integer::gcd(receiver.integer(), args[0].integer());
    Ok(builtins.make_integer(res))
}

fn integer_to_string(receiver: &Object, _args: &[&Object], builtins: &Builtins) -> Eval {
    Ok(builtins.make_string(&receiver.integer().to_string()))
}

// DERIVED METHODS

fn integer_add(receiver: &Object, args: &[&Object], builtins: &Builtins) -> Eval {
    args[0].send("addInteger:", &[receiver], builtins)
}

fn integer_div(receiver: &Object, args: &[&Object], builtins: &Builtins) -> Eval {
    args[0].send("divInteger:", &[receiver], builtins)
}

fn integer_mul(receiver: &Object, args: &[&Object], builtins: &Builtins) -> Eval {
    args[0].send("mulInteger:", &[receiver], builtins)
}

fn integer_sub(receiver: &Object, args: &[&Object], builtins: &Builtins) -> Eval {
    args[0].send("subInteger:", &[receiver], builtins)
}

fn integer_add_float(receiver: &Object, args: &[&Object], builtins: &Builtins) -> Eval {
    receiver.send("asFloat", &[], builtins)?.send("addFloat:", args, builtins)
}

fn integer_div_float(receiver: &Object, args: &[&Object], builtins: &Builtins) -> Eval {
    receiver.send("asFloat", &[], builtins)?.send("divFloat:", args, builtins)
}

fn integer_mul_float(receiver: &Object, args: &[&Object], builtins: &Builtins) -> Eval {
    receiver.send("asFloat", &[], builtins)?.send("mulFloat:", args, builtins)
}

fn integer_sub_float(receiver: &Object, args: &[&Object], builtins: &Builtins) -> Eval {
    receiver.send("asFloat", &[], builtins)?.send("subFloat:", args, builtins)
}
