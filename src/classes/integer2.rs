use crate::objects2::{Eval, Foolang, Object, Vtable};

pub fn vtable() -> Vtable {
    let mut vt = Vtable::new("Integer");
    // FUNDAMENTAL
    vt.def("asFloat", integer_as_float);
    vt.def("asInteger", integer_as_integer);
    vt.def("addInteger:", integer_add_integer);
    vt.def("divInteger:", integer_div_integer);
    vt.def("eqInteger:", integer_eq_integer);
    vt.def("gtInteger:", integer_gt_integer);
    vt.def("gteInteger:", integer_gte_integer);
    vt.def("ltInteger:", integer_lt_integer);
    vt.def("lteInteger:", integer_lte_integer);
    vt.def("mulInteger:", integer_mul_integer);
    vt.def("subInteger:", integer_sub_integer);
    vt.def("toString", integer_to_string);
    vt.def("prefix-", integer_neg);
    vt.def("to:", integer_to);
    vt.def("to:do:", integer_to_do);
    // INCIDENTAL
    vt.def("gcd:", integer_gcd);
    // DERIVED
    vt.def("+", integer_add);
    vt.def("/", integer_div);
    vt.def("==", integer_eq);
    vt.def(">", integer_gt);
    vt.def(">=", integer_gte);
    vt.def("<", integer_lt);
    vt.def("<=", integer_lte);
    vt.def("*", integer_mul);
    vt.def("-", integer_sub);
    vt.def("addFloat:", integer_add_float);
    vt.def("divFloat:", integer_div_float);
    vt.def("mulFloat:", integer_mul_float);
    vt.def("subFloat:", integer_sub_float);
    vt
}

// FUNDAMENTAL METHODS

fn integer_as_integer(receiver: &Object, _args: &[Object], _foo: &Foolang) -> Eval {
    Ok(receiver.to_owned())
}

fn integer_as_float(receiver: &Object, _args: &[Object], foo: &Foolang) -> Eval {
    Ok(foo.make_float(receiver.integer() as f64))
}

fn integer_add_integer(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let res = args[0].integer() + receiver.integer();
    Ok(foo.make_integer(res))
}

fn integer_div_integer(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let res = args[0].integer() / receiver.integer();
    Ok(foo.make_integer(res))
}

fn integer_eq_integer(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let res = receiver.integer() == args[0].integer();
    Ok(foo.make_boolean(res))
}

fn integer_gt_integer(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let res = receiver.integer() > args[0].integer();
    Ok(foo.make_boolean(res))
}

fn integer_gte_integer(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let res = receiver.integer() >= args[0].integer();
    Ok(foo.make_boolean(res))
}

fn integer_lt_integer(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let res = receiver.integer() < args[0].integer();
    Ok(foo.make_boolean(res))
}

fn integer_lte_integer(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let res = receiver.integer() <= args[0].integer();
    Ok(foo.make_boolean(res))
}

fn integer_mul_integer(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let res = args[0].integer() * receiver.integer();
    Ok(foo.make_integer(res))
}

fn integer_sub_integer(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let res = args[0].integer() - receiver.integer();
    Ok(foo.make_integer(res))
}

fn integer_gcd(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    // FIXME: Panics if argument is not an integer.
    let res = num::integer::gcd(receiver.integer(), args[0].integer());
    Ok(foo.make_integer(res))
}

fn integer_to_string(receiver: &Object, _args: &[Object], foo: &Foolang) -> Eval {
    Ok(foo.make_string(&receiver.integer().to_string()))
}

fn integer_to(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let start = receiver.integer();
    // FIXME: Panics if argument is not an integer
    let end = args[0].integer();
    Ok(foo.make_interval(start, end))
}

fn integer_to_do(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let start = receiver.integer();
    let end = args[0].integer();
    let block = args[1].clone();
    let mut i = start;
    let step = if start < end {
        1
    } else {
        -1
    };
    loop {
        block.send("value:", &[foo.make_integer(i)], foo)?;
        if i == end {
            break;
        }
        i += step;
    }
    Ok(receiver.clone())
}

fn integer_neg(receiver: &Object, _args: &[Object], foo: &Foolang) -> Eval {
    Ok(foo.make_integer(-receiver.integer()))
}

// DERIVED METHODS

fn integer_add(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    args[0].send("addInteger:", &[receiver.clone()], foo)
}

fn integer_div(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    args[0].send("divInteger:", &[receiver.clone()], foo)
}

fn integer_eq(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    args[0].send("eqInteger:", &[receiver.clone()], foo)
}

fn integer_gt(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    args[0].send("ltInteger:", &[receiver.clone()], foo)
}

fn integer_gte(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    args[0].send("lteInteger:", &[receiver.clone()], foo)
}

fn integer_lt(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    args[0].send("gtInteger:", &[receiver.clone()], foo)
}

fn integer_lte(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    args[0].send("gteInteger:", &[receiver.clone()], foo)
}

fn integer_mul(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    args[0].send("mulInteger:", &[receiver.clone()], foo)
}

fn integer_sub(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    args[0].send("subInteger:", &[receiver.clone()], foo)
}

fn integer_add_float(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    receiver.send("asFloat", &[], foo)?.send("addFloat:", args, foo)
}

fn integer_div_float(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    receiver.send("asFloat", &[], foo)?.send("divFloat:", args, foo)
}

fn integer_mul_float(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    receiver.send("asFloat", &[], foo)?.send("mulFloat:", args, foo)
}

fn integer_sub_float(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    receiver.send("asFloat", &[], foo)?.send("subFloat:", args, foo)
}
