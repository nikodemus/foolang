use std::cell::{Ref, RefCell, RefMut};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use crate::eval::Env;
use crate::objects::{Datum, Eval, Object, Vtable};
use crate::unwind::Unwind;

pub struct ByteArray {
    data: RefCell<Vec<u8>>,
}

impl ByteArray {
    pub fn borrow(&self) -> Ref<Vec<u8>> {
        self.data.borrow()
    }
    pub fn borrow_mut(&self) -> RefMut<Vec<u8>> {
        self.data.borrow_mut()
    }
}

impl PartialEq for ByteArray {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Eq for ByteArray {}

impl Hash for ByteArray {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(self, state);
    }
}

impl fmt::Debug for ByteArray {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let data = self.data.borrow();
        let mut buf = String::from("[");
        if !data.is_empty() {
            buf.push_str(&format!("{}", &data[0]));
            if data.len() > 1 {
                for elt in &data[1..] {
                    buf.push_str(&format!(", {}", elt));
                }
            }
        }
        buf.push_str("] bytes");
        write!(f, "{}", buf)
    }
}

pub fn class_vtable() -> Vtable {
    let vt = Vtable::for_class("ByteArray");
    vt.add_primitive_method_or_panic("new:", class_byte_array_new);
    vt
}

pub fn instance_vtable() -> Vtable {
    let vt = Vtable::for_instance("ByteArray");
    vt.add_primitive_method_or_panic("at:", byte_array_at);
    vt.add_primitive_method_or_panic("put:at:", byte_array_put_at);
    vt.add_primitive_method_or_panic("size", byte_array_size);
    vt
}

pub fn as_byte_array<'a>(obj: &'a Object, ctx: &str) -> Result<&'a ByteArray, Unwind> {
    match &obj.datum {
        Datum::ByteArray(ref byte_array) => Ok(byte_array),
        _ => Unwind::error(&format!("{:?} is not a ByteArray in {}", obj, ctx)),
    }
}

fn class_byte_array_new(_receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let arg = args[0].as_i64("size in ByteArray##new:")?;
    if arg < 0 {
        return Unwind::error(&format!("Negative argument in ByteArray#new: {}", arg));
    } else {
        let mut vec = Vec::with_capacity(arg as usize);
        vec.resize(arg as usize, 0 as u8);
        Ok(Object {
            vtable: Rc::clone(&env.foo.byte_array_vtable),
            datum: Datum::ByteArray(Rc::new(ByteArray {
                data: RefCell::new(vec),
            })),
        })
    }
}

fn byte_array_at(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let pos = &args[0].as_i64("position in ByteArray#at:")? - 1;
    let vec = receiver.as_byte_array("ByteArray#at:")?.borrow();
    if pos < 0 || vec.len() <= pos as usize {
        return Unwind::error(&format!("Index out of range in ByteArray#at: {}", &args[0]));
    };
    Ok(env.foo.make_integer(vec[pos as usize] as i64))
}

fn byte_array_put_at(receiver: &Object, args: &[Object], _env: &Env) -> Eval {
    let val = args[0].as_u8("value in ByteArray#put:at:")?;
    let pos = args[1].as_i64("position in ByteArray#put:at:")? - 1;
    let mut vec = as_byte_array(receiver, "receiver of ByteArray#put:at:")?.borrow_mut();
    if pos < 0 || vec.len() <= pos as usize {
        return Unwind::error(&format!("Index out of range in ByteArray#put:at: {}", &args[1]));
    }
    vec[pos as usize] = val;
    Ok(receiver.clone())
}

fn byte_array_size(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    let vec = receiver.as_byte_array("ByteArray#size")?.borrow();
    Ok(env.foo.make_integer(vec.len() as i64))
}
