use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use crate::eval::Env;
use crate::objects::{Datum, Eval, Foolang, Object, Vtable};
use crate::unwind::Unwind;

pub struct Dictionary {
    data: RefCell<HashMap<Object, Object>>,
}

impl Dictionary {
    pub fn borrow(&self) -> Ref<HashMap<Object, Object>> {
        self.data.borrow()
    }
    pub fn borrow_mut(&self) -> RefMut<HashMap<Object, Object>> {
        self.data.borrow_mut()
    }
}

impl PartialEq for Dictionary {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Eq for Dictionary {}

impl Hash for Dictionary {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(self, state);
    }
}

impl fmt::Debug for Dictionary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{")?;
        let mut first = true;
        for (k, v) in self.borrow().iter() {
            if first {
                first = false;
            } else {
                write!(f, ", ")?;
            }
            write!(f, "{:?} -> {:?}", k, v)?;
        }
        write!(f, "}}")
    }
}

pub fn class_vtable() -> Vtable {
    let vt = Vtable::new("class Dictionary");
    vt.add_primitive_method_or_panic("new", class_dictionary_new);
    vt
}

pub fn instance_vtable() -> Vtable {
    let vt = Vtable::new("Dictionary");
    vt.add_primitive_method_or_panic("at:", dictionary_at);
    vt.add_primitive_method_or_panic("put:at:", dictionary_put_at);
    vt
}

pub fn into_dictionary(foolang: &Foolang, data: HashMap<Object, Object>) -> Object {
    Object {
        vtable: Rc::clone(&foolang.dictionary_vtable),
        datum: Datum::Dictionary(Rc::new(Dictionary {
            data: RefCell::new(data),
        })),
    }
}

pub fn as_dictionary<'a>(obj: &'a Object, ctx: &str) -> Result<&'a Dictionary, Unwind> {
    match &obj.datum {
        Datum::Dictionary(ref dict) => Ok(dict),
        _ => Unwind::error(&format!("{:?} is not a Dictionary in {}", obj, ctx)),
    }
}

fn class_dictionary_new(_receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    let data = RefCell::new(HashMap::new());
    Ok(Object {
        vtable: env.foo.dictionary_vtable.clone(),
        datum: Datum::Dictionary(Rc::new(Dictionary {
            data,
        })),
    })
}

fn dictionary_at(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    match receiver.as_dictionary("in Dictionary#at:")?.borrow().get(&args[0]) {
        Some(obj) => Ok(obj.clone()),
        None => Ok(env.foo.make_boolean(false)),
    }
}

fn dictionary_put_at(receiver: &Object, args: &[Object], _env: &Env) -> Eval {
    receiver
        .as_dictionary("in Dictionary#put:at:")?
        .borrow_mut()
        .insert(args[1].clone(), args[0].clone());
    Ok(receiver.clone())
}
