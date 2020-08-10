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
        "new:slots:interfaces:directMethods:instanceMethods:",
        class_new_,
    );
    vt
}

pub fn interface_vtable() -> Vtable {
    let vt = Vtable::new("Interface");
    vt.add_primitive_method_or_panic(
        "new:interfaces:directMethods:instanceMethods:",
        interface_new_,
    );
    vt
}

fn class_new_(_receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let class_object = Class::new_class(args[0].as_str()?);
    let class = class_object.as_class_ref()?;
    let mut selector = String::new();
    for (i, slot) in args[1]
        .as_array("slots in Class#new:slots:interfaces:directMethods:InstanceMethods:")?
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
    for each_interface in args[2]
        .as_array("interfaces in Class:new:slots:interfaces:directMethods:instanceMethods:")?
        .borrow()
        .iter()
    {
        class_object.add_interface_object(each_interface)?;
    }
    for method in args[3]
        .as_array("directMethods in Class:new:slots:interfaces:directMethods:instanceMethods:")?
        .borrow()
        .iter()
    {
        // println!("Adding class method: {}", method);
        class_object
            .vtable
            .add_method(method.send("name", &[], env)?.as_str()?, Method::object(method))?
    }
    for method in args[4]
        .as_array("methods in Class:new:slots:interfaces:directMethods:instanceMethods:")?
        .borrow()
        .iter()
    {
        //println!("Adding instance method: {} to {}",
        //         method.send("selector", &[], env)?, &class.instance_vtable.name);
        class
            .instance_vtable
            .add_method(method.send("name", &[], env)?.as_str()?, Method::object(method))?
    }
    Ok(class_object)
}

fn interface_new_(_receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let interface_object = Class::new_interface(args[0].as_str()?);
    let interface = interface_object.as_class_ref()?;
    for each_interface in args[1]
        .as_array("interfaces in Interface:new:interfaces:directMethods:instnceMethods:")?
        .borrow()
        .iter()
    {
        interface_object.add_interface_object(each_interface)?;
    }
    for method in args[2]
        .as_array("directMethods in Interface:new:interfaces:directMethods:instanceMethods:")?
        .borrow()
        .iter()
    {
        // println!("Adding interface method: {}", method);
        interface_object
            .vtable
            .add_method(method.send("name", &[], env)?.as_str()?, Method::object(method))?
    }
    for method in args[3]
        .as_array("methods in Interface:new:interfaces:directMethods:instanceMethods:")?
        .borrow()
        .iter()
    {
        //println!("Adding instance method: {} to {}",
        //         method.send("selector", &[], env)?, &interface.instance_vtable.name);
        interface
            .instance_vtable
            .add_method(method.send("name", &[], env)?.as_str()?, Method::object(method))?
    }
    Ok(interface_object)
}
