use crate::objects::{Eval, Foolang, Object, Vtable};

pub fn class_vtable() -> Vtable {
    let vt = Vtable::new("class Array");
    vt
}

pub fn instance_vtable() -> Vtable {
    let mut vt = Vtable::new("Array");
    vt.def("do:", array_do);
    vt.def("inject:into:", array_inject_into);
    vt.def("push:", array_push);
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

fn array_push(receiver: &Object, args: &[Object], _foo: &Foolang) -> Eval {
    let elt = args[0].clone();
    receiver.as_vec(move |mut vec| {
        vec.push(elt);
        Ok(())
    })?;
    Ok(receiver.clone())
}
