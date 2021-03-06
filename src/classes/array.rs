use std::cell::{Ref, RefCell, RefMut};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use crate::eval::Env;
use crate::objects::{Datum, Eval, Foolang, Object, Vtable};
use crate::unwind::Unwind;

pub struct Array {
    pub etype: Option<Object>,
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

pub fn into_array(foolang: &Foolang, data: Vec<Object>, etype: Option<Object>) -> Object {
    Object {
        vtable: Rc::clone(&foolang.array_vtable),
        datum: Datum::Array(Rc::new(Array {
            etype,
            data: RefCell::new(data),
        })),
    }
}

fn array_of(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let etype = args[0].clone();
    let data = receiver.as_array("Array#of:")?.borrow();
    for elt in data.iter() {
        etype.send("typecheck:", std::slice::from_ref(elt), env)?;
    }
    Ok(Object {
        vtable: receiver.vtable.clone(),
        datum: Datum::Array(Rc::new(Array {
            etype: Some(etype),
            data: RefCell::new(data.clone()),
        })),
    })
}

pub fn class_vtable() -> Vtable {
    let vt = Vtable::for_class("Array");
    vt.add_primitive_method_or_panic("of:new:value:", class_array_of_new_value);
    vt.add_primitive_method_or_panic("toString", class_array_to_string);
    vt
}

pub fn instance_vtable() -> Vtable {
    let vt = Vtable::for_instance("Array");
    vt.add_primitive_method_or_panic("of:", array_of);
    vt.add_primitive_method_or_panic("at:", array_at);
    vt.add_primitive_method_or_panic("put:at:", array_put_at);
    vt.add_primitive_method_or_panic("size", array_size);
    vt.add_primitive_method_or_panic("toString", array_to_string);
    vt.add_primitive_method_or_panic("arrayElementType", array_element_type);
    vt
}

fn class_array_of_new_value(_receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let n = args[1].as_usize("size in Array##new:value:")?;
    let mut v = Vec::with_capacity(n);
    let etype = args[0].clone();
    for _ in 0..n {
        let elt = args[2].clone();
        etype.send("typecheck:", std::slice::from_ref(&elt), env)?;
        v.push(elt);
    }
    Ok(into_array(&env.foo, v, Some(etype)))
}

fn class_array_to_string(_receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_string("Array"))
}

fn array_to_string(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_string(&format!("{}", receiver.as_array("Array#toString")?)))
}

fn array_element_type(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    if let Some(etype) = &receiver.as_array("Array#elementType")?.etype {
        Ok(etype.clone())
    } else {
        env.find_global_or_unwind("Any")
    }
}

fn array_at(receiver: &Object, args: &[Object], _env: &Env) -> Eval {
    let array = receiver.as_array("Array#at:")?;
    let data = array.data.borrow();
    let index_arg = args[0].as_index("Array#put:at:")?;
    let index = (index_arg - 1) as usize;
    if data.len() == 0 {
        return Unwind::error(&format!("Array#at: -- cannot access elements of an empty array!"));
    }
    if index_arg < 1 || data.len() <= index {
        return Unwind::error(&format!(
            "Array#at: -- index out of bounds: {}, should be 1-{}",
            index_arg,
            data.len()
        ));
    }
    Ok(data[index].clone())
}

fn array_put_at(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let elt = args[0].clone();
    let array = receiver.as_array("Array#put:at:")?;
    if let Some(etype) = &array.etype {
        etype.send("typecheck:", std::slice::from_ref(&elt), env)?;
    }
    let mut data = array.data.borrow_mut();
    let index_arg = args[1].as_index("Array#put:at:")?;
    let index = (index_arg - 1) as usize;
    if index_arg < 1 || data.len() <= index {
        return Unwind::error(&format!(
            "Array#put:at: -- index for {} out of bounds: {}, should be 1-{}",
            elt,
            index_arg,
            data.len()
        ));
    }
    data[index] = elt.clone();
    Ok(elt)
}

fn array_size(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    receiver.as_vec(|vec| Ok(env.foo.make_integer(vec.len() as i64)))
}
