use crate::objects::{Eval, Foolang, Object, Vtable};
use crate::unwind::Unwind;

pub fn class_vtable() -> Vtable {
    let vt = Vtable::new("class Array");
    vt
}

pub fn instance_vtable() -> Vtable {
    let mut vt = Vtable::new("Array");
    vt.def("*", array_mul);
    vt.def("+", array_add);
    vt.def("addArray:", array_add_array);
    vt.def("do:", array_do);
    vt.def("inject:into:", array_inject_into);
    vt.def("push:", array_push);
    vt.def("toString", array_to_string);
    vt.def("mulInteger:", array_mul_integer);
    vt.def("mulFloat:", array_mul_float);
    vt
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

fn array_add(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    args[0].send("addArray:", std::slice::from_ref(receiver), foo)
}

fn array_mul(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    args[0].send("*", std::slice::from_ref(receiver), foo)
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
    receiver.as_vec(move |mut vec| {
        vec.push(elt);
        Ok(())
    })?;
    Ok(receiver.clone())
}

fn array_to_string(receiver: &Object, _args: &[Object], foo: &Foolang) -> Eval {
    Ok(foo.into_string(receiver.as_vec(|v| Ok(format!("{:?}", v)))?))
}
