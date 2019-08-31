use std::slice;

use crate::objects::{Eval, Foolang, Object, Vtable};

pub fn vtable() -> Vtable {
    let mut vt = Vtable::new("Integer");
    // FUNDAMENTAL
    vt.def("asFloat", integer_as_float);
    vt.def("asInteger", integer_as_integer);
    vt.def("addInteger:", integer_add_integer);
    vt.def("divInteger:", integer_div_integer);
    vt.def("equalInteger:", integer_equal_integer);
    vt.def("greaterThanInteger:", integer_greater_than_integer);
    vt.def("greaterThanOrEqualInteger:", integer_greater_than_or_equal_integer);
    vt.def("lessThanInteger:", integer_less_than_integer);
    vt.def("lessThanOrEqualInteger:", integer_less_than_or_equal_integer);
    vt.def("mulInteger:", integer_mul_integer);
    vt.def("subInteger:", integer_sub_integer);
    vt.def("toString", integer_to_string);
    vt.def("prefix-", integer_neg);
    vt.def("to:", integer_to);
    vt.def("to:do:", integer_to_do);
    vt.def("times:", integer_times);
    // INCIDENTAL
    vt.def("gcd:", integer_gcd);
    // DERIVED
    vt.def("+", integer_add);
    vt.def("/", integer_div);
    vt.def("==", integer_equal);
    vt.def(">", integer_greater_than);
    vt.def(">=", integer_greater_than_or_equal);
    vt.def("<", integer_less_than);
    vt.def("<=", integer_less_than_or_equal);
    vt.def("*", integer_mul);
    vt.def("-", integer_sub);
    vt.def("addFloat:", integer_add_float);
    vt.def("divFloat:", integer_div_float);
    vt.def("mulFloat:", integer_mul_float);
    vt.def("subFloat:", integer_sub_float);
    vt.def("equalFloat:", integer_equal_float);
    vt.def("lessThanFloat:", integer_less_than_float);
    vt.def("lessThanOrEqualFloat:", integer_less_than_or_equal_float);
    vt.def("greaterThanFloat:", integer_greater_than_float);
    vt.def("greaterThanOrEqualFloat:", integer_greater_than_or_equal_float);
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

fn integer_equal_integer(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let res = receiver.integer() == args[0].integer();
    Ok(foo.make_boolean(res))
}

fn integer_greater_than_integer(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let res = receiver.integer() > args[0].integer();
    Ok(foo.make_boolean(res))
}

fn integer_greater_than_or_equal_integer(
    receiver: &Object,
    args: &[Object],
    foo: &Foolang,
) -> Eval {
    let res = receiver.integer() >= args[0].integer();
    Ok(foo.make_boolean(res))
}

fn integer_less_than_integer(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let res = receiver.integer() < args[0].integer();
    Ok(foo.make_boolean(res))
}

fn integer_less_than_or_equal_integer(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
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

fn integer_neg(receiver: &Object, _args: &[Object], foo: &Foolang) -> Eval {
    Ok(foo.make_integer(-receiver.integer()))
}

fn integer_times(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let mut start = receiver.integer();
    let block = args[0].clone();
    while start > 0 {
        block.send("value", &[], foo)?;
        start -= 1;
    }
    Ok(receiver.clone())
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

fn integer_to_string(receiver: &Object, _args: &[Object], foo: &Foolang) -> Eval {
    Ok(foo.make_string(&receiver.integer().to_string()))
}

// DERIVED METHODS

fn integer_add(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    args[0].send("addInteger:", slice::from_ref(receiver), foo)
}

fn integer_div(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    args[0].send("divInteger:", slice::from_ref(receiver), foo)
}

fn integer_equal(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    args[0].send("equalInteger:", slice::from_ref(receiver), foo)
}

fn integer_greater_than(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    args[0].send("lessThanInteger:", slice::from_ref(receiver), foo)
}

fn integer_greater_than_or_equal(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    args[0].send("lessThanOrEqualInteger:", slice::from_ref(receiver), foo)
}

fn integer_less_than(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    args[0].send("greaterThanInteger:", slice::from_ref(receiver), foo)
}

fn integer_less_than_or_equal(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    args[0].send("greaterThanOrEqualInteger:", slice::from_ref(receiver), foo)
}

fn integer_mul(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    args[0].send("mulInteger:", slice::from_ref(receiver), foo)
}

fn integer_sub(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    args[0].send("subInteger:", slice::from_ref(receiver), foo)
}

fn integer_add_float(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    integer_as_float(receiver, &[], foo)?.send("addFloat:", args, foo)
}

fn integer_div_float(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    integer_as_float(receiver, &[], foo)?.send("divFloat:", args, foo)
}

fn integer_mul_float(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    integer_as_float(receiver, &[], foo)?.send("mulFloat:", args, foo)
}

fn integer_sub_float(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    integer_as_float(receiver, &[], foo)?.send("subFloat:", args, foo)
}

fn integer_equal_float(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    integer_as_float(receiver, &[], foo)?.send("equalFloat:", args, foo)
}

fn integer_greater_than_float(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    integer_as_float(receiver, &[], foo)?.send("greaterThanFloat:", args, foo)
}

fn integer_greater_than_or_equal_float(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    integer_as_float(receiver, &[], foo)?.send("greaterThanOrEqualFloat:", args, foo)
}

fn integer_less_than_float(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    integer_as_float(receiver, &[], foo)?.send("lessThanFloat:", args, foo)
}

fn integer_less_than_or_equal_float(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    integer_as_float(receiver, &[], foo)?.send("lessThanOrEqualFloat:", args, foo)
}
