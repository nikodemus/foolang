use crate::objects::{Eval, Foolang, Object, Vtable};
use crate::unwind::Unwind;

pub fn class_vtable() -> Vtable {
    let vt = Vtable::new("class Array");
    vt
}

pub fn instance_vtable() -> Vtable {
    let mut vt = Vtable::new("Array");
    vt.def("*", array_mul);
    vt.def("/", array_div);
    vt.def("+", array_add);
    vt.def("-", array_sub);
    vt.def("addArray:", array_add_array);
    vt.def("at:", array_at);
    vt.def("divByFloat:", array_div_by_float);
    vt.def("do:", array_do);
    vt.def("dot:", array_dot);
    vt.def("inject:into:", array_inject_into);
    vt.def("norm", array_norm);
    vt.def("mulFloat:", array_mul_float);
    vt.def("mulInteger:", array_mul_integer);
    vt.def("normalized", array_normalized);
    vt.def("push:", array_push);
    vt.def("put:at:", array_put_at);
    vt.def("scalarProjectionOn:", array_scalar_projection_on);
    vt.def("subArray:", array_sub_array);
    vt.def("sum", array_sum);
    vt.def("sum:", array_sum_arg);
    vt.def("toString", array_to_string);
    vt.def("vectorProjectionOn:", array_vector_projection_on);
    vt
}

fn array_at(receiver: &Object, args: &[Object], _foo: &Foolang) -> Eval {
    receiver.as_vec(move |vec| Ok(vec[(args[0].integer() - 1) as usize].clone()))
}

fn array_do(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let block = &args[0];
    receiver.as_vec(move |vec| {
        for elt in vec.iter() {
            block.send("value:", std::slice::from_ref(elt), foo)?;
        }
        Ok(())
    })?;
    Ok(receiver.clone())
}

fn array_dot(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    receiver.as_vec(|a| {
        args[0].as_vec(|b| {
            let n = a.len();
            if n != b.len() {
                return Unwind::error(
                    "Cannot compute dot product for arrays of differing lengths.",
                );
            }
            if n == 0 {
                return Ok(foo.make_integer(0));
            }
            let mut sum = a[0].send("*", std::slice::from_ref(&b[0]), foo)?;
            if n > 1 {
                for i in 1..n {
                    sum =
                        sum.send("+", &[a[i].send("*", std::slice::from_ref(&b[i]), foo)?], foo)?;
                }
            }
            Ok(sum)
        })
    })
}

fn array_inject_into(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let init = args[0].clone();
    let block = &args[1];
    receiver.as_vec(move |vec| {
        let mut inject = init;
        for elt in vec.iter() {
            inject = block.send("value:value:", &[inject, elt.clone()], foo)?;
        }
        Ok(inject)
    })
}

fn array_add_array(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let mut a = receiver.as_vec(|v| Ok(v.clone()))?;
    let n = a.len();
    args[0].as_vec(move |b| {
        if n != b.len() {
            Unwind::error("Cannot add arrays of differing lengths.")
        } else {
            for i in 0..n {
                a[i] = a[i].send("+", std::slice::from_ref(&b[i]), foo)?;
            }
            Ok(foo.into_array(a))
        }
    })
}

fn array_sub_array(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let mut a = receiver.as_vec(|v| Ok(v.clone()))?;
    let n = a.len();
    args[0].as_vec(move |b| {
        if n != b.len() {
            Unwind::error("Cannot substract arrays of differing lengths.")
        } else {
            for i in 0..n {
                a[i] = b[i].send("-", std::slice::from_ref(&a[i]), foo)?;
            }
            Ok(foo.into_array(a))
        }
    })
}

fn array_add(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    args[0].send("addArray:", std::slice::from_ref(receiver), foo)
}

fn array_sub(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    args[0].send("subArray:", std::slice::from_ref(receiver), foo)
}

fn array_mul(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    args[0].send("*", std::slice::from_ref(receiver), foo)
}

fn array_div(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    args[0].send("divArray:", std::slice::from_ref(receiver), foo)
}

fn array_div_by_float(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let mut v = receiver.as_vec(|v| Ok(v.clone()))?;
    for i in 0..v.len() {
        let f = v[i].send("asFloat", &[], foo)?;
        v[i] = args[0].send("divFloat:", &[f], foo)?;
    }
    Ok(foo.into_array(v))
}

fn array_norm(receiver: &Object, _args: &[Object], foo: &Foolang) -> Eval {
    receiver.as_vec(|v| {
        let mut abs = 0.0;
        for elt in v.iter() {
            let f = elt.send("asFloat", &[], foo)?.float();
            abs += f * f;
        }
        Ok(foo.make_float(abs.sqrt()))
    })
}

fn array_normalized(receiver: &Object, _args: &[Object], foo: &Foolang) -> Eval {
    let reciprocal = foo.make_float(1.0 / array_norm(receiver, &[], foo)?.float());
    array_mul(receiver, std::slice::from_ref(&reciprocal), foo)
}

fn array_mul_integer(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let mut v = receiver.as_vec(|v| Ok(v.clone()))?;
    for i in 0..v.len() {
        v[i] = v[i].send("mulInteger:", args, foo)?;
    }
    Ok(foo.into_array(v))
}

fn array_mul_float(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let mut v = receiver.as_vec(|v| Ok(v.clone()))?;
    for i in 0..v.len() {
        v[i] = v[i].send("mulFloat:", args, foo)?;
    }
    Ok(foo.into_array(v))
}

fn array_push(receiver: &Object, args: &[Object], _foo: &Foolang) -> Eval {
    let elt = args[0].clone();
    receiver.as_mut_vec(move |mut vec| {
        vec.push(elt);
        Ok(())
    })?;
    Ok(receiver.clone())
}

fn array_put_at(receiver: &Object, args: &[Object], _foo: &Foolang) -> Eval {
    let elt = args[0].clone();
    receiver.as_mut_vec(move |mut vec| {
        vec[(args[1].integer() - 1) as usize] = elt.clone();
        Ok(elt)
    })
}

fn array_scalar_projection_on(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let ab = array_dot(receiver, args, foo)?;
    let bn = array_norm(&args[0], &[], foo)?;
    ab.send("/", &[bn], foo)
}
fn array_sum(receiver: &Object, _args: &[Object], foo: &Foolang) -> Eval {
    let mut sum = foo.make_boolean(false);
    receiver.as_vec(|v| {
        if v.len() > 0 {
            sum = v[0].clone();
            if v.len() > 1 {
                for elt in v[1..].iter() {
                    sum = sum.send("+", std::slice::from_ref(elt), foo)?;
                }
            }
        }
        Ok(sum)
    })
}

fn array_sum_arg(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let mut sum = foo.make_boolean(false);
    let block = &args[0];
    receiver.as_vec(|v| {
        if v.len() > 0 {
            sum = block.send("value:", std::slice::from_ref(&v[0]), foo)?;
            if v.len() > 1 {
                for elt in v[1..].iter() {
                    let val = block.send("value:", std::slice::from_ref(elt), foo)?;
                    sum = sum.send("+", std::slice::from_ref(&val), foo)?;
                }
            }
        }
        Ok(sum)
    })
}

fn array_to_string(receiver: &Object, _args: &[Object], foo: &Foolang) -> Eval {
    Ok(foo.into_string(receiver.as_vec(|v| Ok(format!("{:?}", v)))?))
}

fn array_vector_projection_on(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let ab = array_dot(receiver, args, foo)?;
    let bb = array_dot(&args[0], args, foo)?;
    ab.send("/", &[bb], foo)?.send("*", args, foo)
}
