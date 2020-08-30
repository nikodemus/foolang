use std::cell::RefCell;
use std::cmp::Eq;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use crate::eval::Env;
use crate::objects::{Datum, Eval, Instance, Method, Object, Slot, Vtable};
use crate::unwind::Unwind;

pub struct Class {
    pub class_vtable: Rc<Vtable>,
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
        let class_vtable = Rc::new(Vtable::for_class(&format!("class {}", name)));
        let instance_vtable = Rc::new(Vtable::for_instance(name));
        let class = Object {
            vtable: class_vtable.clone(),
            datum: Datum::Class(Rc::new(Class {
                class_vtable,
                instance_vtable: instance_vtable.clone(),
                interface: false,
            })),
        };
        instance_vtable.class.borrow_mut().replace(class.clone());
        class
    }
    pub fn new_interface(name: &str) -> Object {
        let class_vtable = Rc::new(Vtable::for_class(&format!("interface {}", name)));
        Object {
            vtable: class_vtable.clone(),
            datum: Datum::Class(Rc::new(Class {
                class_vtable,
                instance_vtable: Rc::new(Vtable::for_instance(name)),
                interface: true,
            })),
        }
    }
    pub fn object(class_vtable: &Rc<Vtable>, instance_vtable: &Rc<Vtable>) -> Object {
        let class = Object {
            vtable: Rc::clone(class_vtable),
            datum: Datum::Class(Rc::new(Class {
                class_vtable: class_vtable.clone(),
                instance_vtable: Rc::clone(instance_vtable),
                interface: false,
            })),
        };
        instance_vtable.class.borrow_mut().replace(class.clone());
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

    pub fn find_slot(&self, name: &str) -> Option<Slot> {
        self.instance_vtable.slots().iter().find(|s| &s.name == name).cloned()
    }
}

pub fn class_vtable() -> Vtable {
    let vt = Vtable::raw("Class");
    vt.add_primitive_method_or_panic(
        "new:slots:interfaces:directMethods:instanceMethods:",
        class_new_,
    );
    vt.add_primitive_method_or_panic("classOf", generic_class_class);
    vt.add_primitive_method_or_panic("includes:", class_includes_);
    vt.add_primitive_method_or_panic("typecheck:", class_typecheck_);
    vt.add_primitive_method_or_panic("name", generic_class_name);
    vt
}

pub fn interface_vtable() -> Vtable {
    let vt = Vtable::raw("Interface");
    vt.add_primitive_method_or_panic(
        "new:interfaces:directMethods:instanceMethods:",
        interface_new_,
    );
    vt.add_primitive_method_or_panic("classOf", generic_class_class);
    vt.add_primitive_method_or_panic("includes:", interface_includes_);
    vt.add_primitive_method_or_panic("typecheck:", interface_typecheck_);
    vt.add_primitive_method_or_panic("name", generic_class_name);
    vt
}

// Is the argument a class?
fn is_class(argument: &Object) -> bool {
    match &argument.datum {
        Datum::Class(class) => !class.interface,
        _ => false,
    }
}

// Is the argument an interface?
fn is_interface(argument: &Object) -> bool {
    match &argument.datum {
        Datum::Class(class) => class.interface,
        _ => false,
    }
}

fn class_includes_(_receiver: &Object, args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_boolean(is_class(&args[0])))
}

fn class_typecheck_(_receiver: &Object, args: &[Object], _env: &Env) -> Eval {
    let arg = &args[0];
    if is_class(arg) {
        Ok(arg.clone())
    } else {
        Unwind::type_error(arg.clone(), String::from("Class"))
    }
}

fn interface_includes_(_receiver: &Object, args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_boolean(is_interface(&args[0])))
}

fn interface_typecheck_(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let arg = &args[0];
    if is_interface(arg) {
        Ok(arg.clone())
    } else {
        // panic!("boom");
        env.find_global_or_unwind("TypeError")?.send(
            "raise:expected:",
            &[args[0].clone(), receiver.clone()],
            env,
        )
    }
}

// FIXME: Doesn't match the MOP plan
pub fn generic_class_class(_receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    env.find_global_or_unwind("Class")
}

pub fn generic_class_name(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_string(&receiver.as_class_ref()?.instance_vtable.name))
}

pub fn generic_instance_class(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    match *receiver.vtable.class.borrow() {
        Some(ref class) => Ok(class.clone()),
        None => Ok(env.foo.make_boolean(false)),
    }
}

pub fn generic_class_typecheck_(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let class = receiver.as_class_ref()?;
    let arg = &args[0];
    if arg.is_type(&class.instance_vtable) {
        Ok(arg.clone())
    } else {
        // panic!("boom");
        env.find_global_or_unwind("TypeError")?.send(
            "raise:expected:",
            &[arg.clone(), receiver.clone()],
            env,
        )
    }
}

pub fn generic_class_includes_(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let class = receiver.as_class_ref()?;
    Ok(env.foo.make_boolean(args[0].is_type(&class.instance_vtable)))
}

pub fn generic_class_add_direct_method_(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let method = &args[0];
    let class = receiver.as_class_ref()?;
    class
        .class_vtable
        .add_method(method.send("name", &[], env)?.as_str()?, Method::object(method))?;
    Ok(receiver.clone())
}

pub fn generic_class_add_instance_method_(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let method = &args[0];
    let class = receiver.as_class_ref()?;
    class
        .instance_vtable
        .add_method(method.send("name", &[], env)?.as_str()?, Method::object(method))?;
    Ok(receiver.clone())
}

pub fn generic_class_add_interface_(receiver: &Object, args: &[Object], _env: &Env) -> Eval {
    receiver.add_interface_object(&args[0])?;
    Ok(receiver.clone())
}

pub fn generic_class_new_(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let class = receiver.as_class_ref()?;
    let mut instance_variables = Vec::with_capacity(args.len());
    for (slot, arg) in class.instance_vtable.slots.borrow().iter().zip(args) {
        let mut val = arg.clone();
        if let Some(slot_type) = &slot.typed {
            // println!("{}.{} :: {} = {}",
            //          &class.instance_vtable.name,
            //          &slot.name, &slot_type, &val);
            val = slot_type.send("typecheck:", &[val], env)?
        }
        instance_variables.push(val);
    }
    Ok(Object {
        vtable: Rc::clone(&class.instance_vtable),
        datum: Datum::Instance(Rc::new(Instance {
            instance_variables: RefCell::new(instance_variables),
        })),
    })
}

// FIXME: duplicates logic in Foolang::make_class()
fn class_new_(_receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let class_object = Class::new_class(args[0].as_str()?);
    let class = class_object.as_class_ref()?;
    let mut ctor = String::new();
    for (i, slot) in args[1]
        .as_array("slots in Class#new:slots:interfaces:directMethods:InstanceMethods:")?
        .borrow()
        .iter()
        .enumerate()
    {
        let name = slot.send("name", &[], env)?;
        let type_obj = slot.send("type", &[], env)?;
        ctor.push_str(name.as_str()?);
        ctor.push_str(":");
        class.add_slot(name.as_str()?, i, Some(type_obj))?;
    }
    if ctor.is_empty() {
        ctor.push_str("new");
    }
    class.class_vtable.add_ctor(&ctor);
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
