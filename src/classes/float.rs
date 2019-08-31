use std::slice;

use crate::objects::{Eval, Foolang, Object, Vtable};

pub fn vtable() -> Vtable {
    let mut vt = Vtable::new("Float");
    // FUNDAMENTAL
    vt.def("addFloat:", float_add_float);
    vt.def("asFloat", float_as_float);
    vt.def("divFloat:", float_div_float);
    vt.def("equalFloat:", float_equal_float);
    vt.def("greaterThanFloat:", float_greater_than_float);
    vt.def("greaterThanOrEqualFloat:", float_greater_than_or_equal_float);
    vt.def("lessThanFloat:", float_less_than_float);
    vt.def("lessThanOrEqualFloat:", float_less_than_or_equal_float);
    vt.def("mulFloat:", float_mul_float);
    vt.def("prefix-", float_neg);
    vt.def("subFloat:", float_sub_float);
    vt.def("toString", float_to_string);
    // DERIVED
    vt.def("<", float_less_than);
    vt.def(">", float_greater_than);
    vt.def("==", float_equal);
    vt.def("<=", float_less_than_or_equal);
    vt.def(">=", float_greater_than_or_equal);
    vt.def("+", float_add);
    vt.def("/", float_div);
    vt.def("*", float_mul);
    vt.def("-", float_sub);
    vt.def("addInteger:", float_add_integer);
    vt.def("asInteger", float_as_integer);
    vt.def("atLeast:atMost:", float_at_least_at_most);
    vt.def("divInteger:", float_div_integer);
    vt.def("equalInteger:", float_equal_integer);
    vt.def("greaterThanInteger:", float_greater_than_integer);
    vt.def("greaterThanOrEqualInteger:", float_greater_than_or_equal_integer);
    vt.def("lessThanInteger:", float_less_than_integer);
    vt.def("lessThanOrEqualInteger:", float_less_than_or_equal_integer);
    vt.def("mulInteger:", float_mul_integer);
    vt.def("subInteger:", float_sub_integer);
    vt.def("divArray:", float_div_array);
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

fn float_div_array(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    args[0].send("divByFloat:", std::slice::from_ref(receiver), foo)
}

fn float_mul_float(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let res = args[0].float() * receiver.float();
    Ok(foo.make_float(res))
}

fn float_sub_float(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let res = args[0].float() - receiver.float();
    Ok(foo.make_float(res))
}

fn float_equal_float(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let res = receiver.float() == args[0].float();
    Ok(foo.make_boolean(res))
}

fn float_greater_than_float(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let res = receiver.float() > args[0].float();
    Ok(foo.make_boolean(res))
}

fn float_greater_than_or_equal_float(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let res = receiver.float() >= args[0].float();
    Ok(foo.make_boolean(res))
}

fn float_less_than_float(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let res = receiver.float() < args[0].float();
    Ok(foo.make_boolean(res))
}

fn float_less_than_or_equal_float(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let res = receiver.float() <= args[0].float();
    Ok(foo.make_boolean(res))
}

fn float_neg(receiver: &Object, _args: &[Object], foo: &Foolang) -> Eval {
    Ok(foo.make_float(-receiver.float()))
}

fn float_to_string(receiver: &Object, _args: &[Object], foo: &Foolang) -> Eval {
    Ok(foo.make_string(&receiver.float().to_string()))
}

// DERIVED METHODS

fn float_add(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    args[0].send("addFloat:", slice::from_ref(receiver), foo)
}

fn float_div(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    args[0].send("divFloat:", slice::from_ref(receiver), foo)
}

fn float_mul(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    args[0].send("mulFloat:", slice::from_ref(receiver), foo)
}

fn float_sub(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    args[0].send("subFloat:", slice::from_ref(receiver), foo)
}

fn float_add_integer(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let float = args[0].send("asFloat", &[], foo)?;
    float_add_float(receiver, &[float], foo)
}

fn float_div_integer(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let float = args[0].send("asFloat", &[], foo)?;
    float_div_float(receiver, &[float], foo)
}

fn float_mul_integer(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let float = args[0].send("asFloat", &[], foo)?;
    float_mul_float(receiver, &[float], foo)
}

fn float_sub_integer(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let float = args[0].send("asFloat", &[], foo)?;
    float_sub_float(receiver, &[float], foo)
}

fn float_less_than(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    args[0].send("greaterThanFloat:", std::slice::from_ref(receiver), foo)
}

fn float_less_than_or_equal(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    args[0].send("greaterThanOrEqualFloat:", std::slice::from_ref(receiver), foo)
}

fn float_greater_than(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    args[0].send("lessThanFloat:", std::slice::from_ref(receiver), foo)
}

fn float_greater_than_or_equal(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    args[0].send("lessThanOrEqualFloat:", std::slice::from_ref(receiver), foo)
}

fn float_equal(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    args[0].send("equalFloat:", std::slice::from_ref(receiver), foo)
}

fn float_equal_integer(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let float = args[0].send("asFloat", &[], foo)?;
    float_equal_float(receiver, &[float], foo)
}

fn float_greater_than_integer(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let float = args[0].send("asFloat", &[], foo)?;
    float_greater_than_float(receiver, &[float], foo)
}

fn float_greater_than_or_equal_integer(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let float = args[0].send("asFloat", &[], foo)?;
    float_greater_than_or_equal_float(receiver, &[float], foo)
}

fn float_less_than_integer(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let float = args[0].send("asFloat", &[], foo)?;
    float_less_than_float(receiver, &[float], foo)
}

fn float_less_than_or_equal_integer(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let float = args[0].send("asFloat", &[], foo)?;
    float_less_than_or_equal_float(receiver, &[float], foo)
}

fn float_at_least_at_most(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let min_value = args[0].send("asFloat", &[], foo)?;
    let max_value = args[1].send("asFloat", &[], foo)?;
    let value = receiver.float();
    if value < min_value.float() {
        return Ok(min_value);
    }
    if value > max_value.float() {
        return Ok(max_value);
    }
    Ok(receiver.clone())
}
