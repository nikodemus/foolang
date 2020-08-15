use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use crate::eval::Env;
use crate::objects::{Datum, Eval, Object, Vtable};
use crate::unwind::Unwind;

pub struct Record {
    data: RefCell<HashMap<String, Object>>,
}

impl Record {
    pub fn borrow(&self) -> Ref<HashMap<String, Object>> {
        self.data.borrow()
    }
    pub fn borrow_mut(&self) -> RefMut<HashMap<String, Object>> {
        self.data.borrow_mut()
    }
}

impl PartialEq for Record {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Eq for Record {}

impl Hash for Record {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(self, state);
    }
}

impl fmt::Debug for Record {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{")?;
        let mut first = true;
        for (k, v) in self.borrow().iter() {
            if first {
                first = false;
            } else {
                write!(f, ", ")?;
            }
            write!(f, "{}: {:?}", k, v)?;
        }
        write!(f, "}}")
    }
}

pub fn class_vtable() -> Vtable {
    let vt = Vtable::new("Record");
    vt.add_primitive_method_or_panic("perform:with:", class_record_perform_with);
    vt.add_primitive_method_or_panic("keysIn:", class_record_keys_in);
    vt.add_primitive_method_or_panic("at:in:", class_record_at_in);
    vt
}

pub fn instance_vtable() -> Vtable {
    let vt = Vtable::new("Record");
    vt.add_primitive_method_or_panic("perform:with:", record_perform_with);
    vt.add_primitive_method_or_panic("displayOn:", record_display_on);
    vt
}

pub fn as_record<'a>(obj: &'a Object, ctx: &str) -> Result<&'a Record, Unwind> {
    match &obj.datum {
        Datum::Record(ref record) => Ok(record),
        _ => Unwind::error(&format!("{:?} is not a Record in {}", obj, ctx)),
    }
}

fn class_record_perform_with(_receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let mut data = HashMap::new();
    let selector = args[0].string_as_str();
    let values = args[1].as_array("in Record##perform:with:")?.borrow();
    for (k, v) in selector.split(':').zip(&*values) {
        data.insert(k.to_string(), v.clone());
    }
    Ok(Object {
        vtable: env.foo.record_vtable.clone(),
        datum: Datum::Record(Rc::new(Record {
            data: RefCell::new(data),
        })),
    })
}

fn class_record_keys_in(_receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let r: &Record = args[0].as_record("in keysIn:")?;
    let data: Ref<HashMap<_, _>> = r.borrow();
    let mut keys = Vec::new();
    for (k, _) in data.iter() {
        keys.push(env.foo.make_string(k));
    }
    Ok(env.foo.into_array(keys, None))
}

fn class_record_at_in(_receiver: &Object, args: &[Object], _env: &Env) -> Eval {
    let k = args[0].as_str()?;
    let r: &Record = args[1].as_record("record in Record#at:in:")?;
    let data: Ref<HashMap<_, _>> = r.borrow();
    Ok(data[k].clone())
}

fn record_perform_with(receiver: &Object, args: &[Object], _env: &Env) -> Eval {
    let r: &Record = receiver.as_record("in Record#perform:with:")?;
    let data: Ref<HashMap<_, _>> = r.borrow();
    let selector = args[0].string_as_str();
    match data.get(selector) {
        Some(obj) => Ok(obj.clone()),
        None => Unwind::message_error(receiver, selector, &args[1..2]),
    }
}

fn record_display_on(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    args[0].send("print:", &[receiver.send("toString", &[], env)?], env)
}
