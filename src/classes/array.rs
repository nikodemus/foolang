use std::hash::{Hash, Hasher};
use std::cell::{Ref, RefCell, RefMut};
use std::fmt;
use std::rc::Rc;

use crate::eval::Env;
use crate::objects::{Datum, Eval, Object, Vtable, Foolang};
use crate::unwind::Unwind;

pub struct Array {
    pub data: RefCell<Vec<Object>>,
}

impl Array {
    pub fn borrow(&self) -> Ref<Vec<Object>> {
        self.data.borrow()
    }
    pub fn borrow_mut(&self) -> RefMut<Vec<Object>> {
        self.data.borrow_mut()
    }
}

impl PartialEq for Array {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Eq for Array {}

impl Hash for Array {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(self, state);
    }
}

impl fmt::Debug for Array {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let data = self.data.borrow();
        let mut buf = String::from("[");
        if !data.is_empty() {
            buf.push_str(format!("{:?}", &data[0]).as_str());
            if data.len() > 1 {
                for elt in &data[1..] {
                    buf.push_str(format!(", {:?}", elt).as_str());
                }
            }
        }
        buf.push_str("]");
        write!(f, "{}", buf)
    }
}

impl fmt::Display for Array {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let data = self.data.borrow();
        let mut buf = String::from("[");
        if !data.is_empty() {
            buf.push_str(format!("{}", &data[0]).as_str());
            if data.len() > 1 {
                for elt in &data[1..] {
                    buf.push_str(format!(", {}", elt).as_str());
                }
            }
        }
        buf.push_str("]");
        write!(f, "{}", buf)
    }
}

pub fn as_array<'a>(obj: &'a Object, ctx: &str) -> Result<&'a Array, Unwind> {
    match &obj.datum {
        Datum::Array(ref array) => Ok(array),
        _ => Unwind::error(&format!("{:?} is not a Array in {}", obj, ctx)),
    }
}

pub fn into_array(foolang: &Foolang, data: Vec<Object>) -> Object {
    Object {
        vtable: Rc::clone(&foolang.array_vtable),
        datum: Datum::Array(Rc::new(Array {
            data: RefCell::new(data),
        }))
    }
}

pub fn class_vtable() -> Vtable {
    let vt = Vtable::new("class Array");
    vt
}

pub fn instance_vtable() -> Vtable {
    let mut vt = Vtable::new("Array");
    vt.def("addArray:", array_add_array);
    vt.def("at:", array_at);
    vt.def("dot:", array_dot);
    vt.def("inject:into:", array_inject_into);
    vt.def("push:", array_push);
    vt.def("put:at:", array_put_at);
    vt.def("subArray:", array_sub_array);
    vt.def("toString", array_to_string);
    vt
}

fn array_at(receiver: &Object, args: &[Object], _env: &Env) -> Eval {
    receiver.as_vec(move |vec| Ok(vec[(args[0].integer() - 1) as usize].clone()))
}

fn array_dot(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    receiver.as_vec(|a| {
        args[0].as_vec(|b| {
            let n = a.len();
            if n != b.len() {
                return Unwind::error(
                    "Cannot compute dot product for arrays of differing lengths.",
                );
            }
            if n == 0 {
                return Ok(env.foo.make_integer(0));
            }
            let mut sum = a[0].send("*", std::slice::from_ref(&b[0]), env)?;
            if n > 1 {
                for i in 1..n {
                    sum =
                        sum.send("+", &[a[i].send("*", std::slice::from_ref(&b[i]), env)?], env)?;
                }
            }
            Ok(sum)
        })
    })
}

fn array_inject_into(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let init = args[0].clone();
    let block = &args[1];
    receiver.as_vec(move |vec| {
        let mut inject = init;
        for elt in vec.iter() {
            inject = block.send("value:value:", &[inject, elt.clone()], env)?;
        }
        Ok(inject)
    })
}

fn array_add_array(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let mut a = receiver.as_vec(|v| Ok(v.clone()))?;
    let n = a.len();
    args[0].as_vec(move |b| {
        if n != b.len() {
            Unwind::error("Cannot add arrays of differing lengths.")
        } else {
            for i in 0..n {
                a[i] = a[i].send("+", std::slice::from_ref(&b[i]), env)?;
            }
            Ok(env.foo.into_array(a))
        }
    })
}

fn array_sub_array(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let mut a = receiver.as_vec(|v| Ok(v.clone()))?;
    let n = a.len();
    args[0].as_vec(move |b| {
        if n != b.len() {
            Unwind::error("Cannot substract arrays of differing lengths.")
        } else {
            for i in 0..n {
                a[i] = b[i].send("-", std::slice::from_ref(&a[i]), env)?;
            }
            Ok(env.foo.into_array(a))
        }
    })
}

fn array_push(receiver: &Object, args: &[Object], _env: &Env) -> Eval {
    let elt = args[0].clone();
    receiver.as_mut_vec(move |mut vec| {
        vec.push(elt);
        Ok(())
    })?;
    Ok(receiver.clone())
}

fn array_put_at(receiver: &Object, args: &[Object], _env: &Env) -> Eval {
    let elt = args[0].clone();
    receiver.as_mut_vec(move |mut vec| {
        vec[(args[1].integer() - 1) as usize] = elt.clone();
        Ok(elt)
    })
}

fn array_to_string(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.into_string(receiver.as_vec(|v| Ok(format!("{:?}", v)))?))
}
