use std::cell::{Ref, RefCell, RefMut};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use crate::eval::Env;
use crate::objects::{Datum, Eval, Foolang, Object, Vtable};
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
        })),
    }
}

pub fn class_vtable() -> Vtable {
    let vt = Vtable::new("Array");
    vt.add_primitive_method_or_panic("withCapacity:", class_array_with_capacity);
    vt
}

pub fn instance_vtable() -> Vtable {
    let vt = Vtable::new("Array");
    vt.add_primitive_method_or_panic("at:", array_at);
    vt.add_primitive_method_or_panic("pop", array_pop);
    vt.add_primitive_method_or_panic("push:", array_push);
    vt.add_primitive_method_or_panic("put:at:", array_put_at);
    vt.add_primitive_method_or_panic("size", array_size);
    vt
}

fn class_array_with_capacity(_receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let n = args[0].as_usize("capacity in Array##withCapacity:")?;
    let v = Vec::with_capacity(n);
    Ok(into_array(&env.foo, v))
}

fn array_at(receiver: &Object, args: &[Object], _env: &Env) -> Eval {
    receiver
        .as_vec(move |vec| Ok(vec[(args[0].as_usize("index in Array#at:")? - 1) as usize].clone()))
}

fn array_pop(receiver: &Object, _args: &[Object], _env: &Env) -> Eval {
    receiver.as_mut_vec(move |mut vec| match vec.pop() {
        Some(obj) => Ok(obj),
        None => Unwind::error(&format!("Could not pop from: {:?}", receiver)),
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
        vec[(args[1].as_usize("index in Array#put:at:")? - 1) as usize] = elt.clone();
        Ok(elt)
    })
}

fn array_size(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    receiver.as_vec(|vec| Ok(env.foo.make_integer(vec.len() as i64)))
}
