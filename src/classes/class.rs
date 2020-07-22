use std::cmp::Eq;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use crate::eval::Env;
use crate::objects::{generic_ctor, Datum, Eval, Method, Object, Vtable};
use crate::unwind::Unwind;

pub struct Class {
    pub instance_vtable: Rc<Vtable>,
    pub interface: bool,
}

impl PartialEq for Class {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Eq for Class {}

impl Hash for Class {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(self, state);
    }
}

impl Class {
    pub fn new_class(name: &str) -> Object {
        Object {
            vtable: Rc::new(Vtable::new(&format!("class {}", name))),
            datum: Datum::Class(Rc::new(Class {
                instance_vtable: Rc::new(Vtable::new(name)),
                interface: false,
            })),
        }
    }
    pub fn new_interface(name: &str) -> Object {
        Object {
            vtable: Rc::new(Vtable::new(&format!("interface {}", name))),
            datum: Datum::Class(Rc::new(Class {
                instance_vtable: Rc::new(Vtable::new(name)),
                interface: true,
            })),
        }
    }
    pub fn object(class_vtable: &Rc<Vtable>, instance_vtable: &Rc<Vtable>) -> Object {
        let class = Object {
            vtable: Rc::clone(class_vtable),
            datum: Datum::Class(Rc::new(Class {
                instance_vtable: Rc::clone(instance_vtable),
                interface: false,
            })),
        };
        class
    }

    pub fn add_slot(&self, name: &str, index: usize, typed: Option<Object>) -> Result<(), Unwind> {
        if self.interface {
            return Unwind::error("BUG: Cannot add slot to an interface");
        }
        self.instance_vtable.add_slot(name, index, typed);
        if !name.starts_with("_") {
            self.instance_vtable.add_method(name, Method::reader(index))?;
        }
        Ok(())
    }
}

pub fn class_vtable() -> Vtable {
    let vt = Vtable::new("Class");
    vt.add_primitive_method_or_panic(
        "new:slots:interfaces:methods:",
        class_new_slots_interfaces_methods,
    );
    vt
}

fn class_new_slots_interfaces_methods(_receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let class_object = Class::new_class(args[0].as_str()?);
    let class = class_object.as_class_ref()?;
    let mut selector = String::new();
    for (i, slot) in args[1]
        .as_array("slots in Class#new:slots:interfaces:methods:")?
        .borrow()
        .iter()
        .enumerate()
    {
        let name = slot.as_str()?;
        selector.push_str(name);
        selector.push_str(":");
        class.add_slot(name, i, None)?;
    }
    if selector.is_empty() {
        selector.push_str("new");
    }
    class_object.add_primitive_class_method(&selector, generic_ctor)?;
    for interface in
        args[2].as_array("interfaces in Class:new:slots:interfaes:methods:")?.borrow().iter()
    {
        class_object.add_interface(env, interface.as_str()?)?;
    }
    for method in
        args[3].as_array("methods in Class:new:slots:interfaces:methods:")?.borrow().iter()
    {
        class
            .instance_vtable
            .add_method(method.send("selector", &[], env)?.as_str()?, Method::object(method))?
    }
    Ok(class_object)
}
