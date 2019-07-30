use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::eval;
use crate::eval::Frame;
use crate::parse::{ClassDefinition, Expr, Var};
use crate::tokenstream::{Span, SyntaxError};

use crate::classes;

#[derive(Debug, PartialEq)]
pub enum Unwind {
    Exception(SyntaxError),
    ReturnFrom(Frame, Object),
}

impl Unwind {
    pub fn exception<T>(error: SyntaxError) -> Result<T, Unwind> {
        Err(Unwind::Exception(error))
    }
    pub fn return_from<T>(frame: Frame, value: Object) -> Result<T, Unwind> {
        Err(Unwind::ReturnFrom(frame, value))
    }
    pub fn add_context(self, source: &str) -> Unwind {
        match self {
            Unwind::Exception(error) => Unwind::Exception(error.add_context(source)),
            _ => self,
        }
    }
}

pub type Eval = Result<Object, Unwind>;

pub trait Source {
    fn source(self, span: &Span) -> Self;
}

impl Source for Eval {
    fn source(mut self, span: &Span) -> Self {
        if let Err(Unwind::Exception(e)) = &mut self {
            e.span = span.clone();
        }
        self
    }
}

type MethodFunction = fn(&Object, &[&Object], &Builtins) -> Eval;

pub enum Method {
    Primitive(MethodFunction),
    Interpreter(Closure),
    Reader(usize),
}

#[derive(Debug, PartialEq)]
pub struct Slot {
    pub index: usize,
    pub vtable: Option<Rc<Vtable>>,
}

pub struct Vtable {
    pub name: String,
    pub methods: HashMap<String, Method>,
    pub slots: HashMap<String, Slot>,
}

impl Vtable {
    pub fn new(class: &str) -> Vtable {
        Vtable {
            name: class.to_string(),
            methods: HashMap::new(),
            slots: HashMap::new(),
        }
    }

    pub fn def(&mut self, name: &str, method: MethodFunction) {
        self.methods.insert(name.to_string(), Method::Primitive(method));
    }

    pub fn add_method(&mut self, selector: &str, method: Closure) {
        self.methods.insert(selector.to_string(), Method::Interpreter(method));
    }

    pub fn add_reader(&mut self, selector: &str, index: usize) {
        self.methods.insert(selector.to_string(), Method::Reader(index));
    }

    pub fn add_slot(&mut self, name: &str, index: usize, vtable: Option<Rc<Vtable>>) {
        self.slots.insert(
            name.to_string(),
            Slot {
                index,
                vtable,
            },
        );
    }

    pub fn selectors(&self) -> Vec<String> {
        let mut selectors = vec![];
        for key in self.methods.keys() {
            selectors.push(key.clone());
        }
        selectors
    }

    pub fn get(&self, name: &str) -> Option<&Method> {
        self.methods.get(name)
    }
}

impl fmt::Debug for Vtable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Vtable[{}]", self.name)
    }
}

impl PartialEq for Vtable {
    fn eq(&self, other: &Self) -> bool {
        self as *const _ == other as *const _
    }
}

#[derive(PartialEq, Clone)]
pub struct Object {
    pub vtable: Rc<Vtable>,
    pub datum: Datum,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Arg {
    pub span: Span,
    pub name: String,
    pub vtable: Option<Rc<Vtable>>,
}

impl Arg {
    pub fn new(span: Span, name: String, vtable: Option<Rc<Vtable>>) -> Arg {
        Arg {
            span,
            name,
            vtable,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Closure {
    env: Option<Frame>,
    pub params: Vec<Arg>,
    pub body: Expr,
}

impl Closure {
    pub fn env(&self) -> Option<Frame> {
        self.env.clone()
    }
}

#[derive(PartialEq)]
pub struct Class {
    pub instance_vtable: Rc<Vtable>,
}

#[derive(PartialEq)]
pub struct Instance {
    pub instance_variables: RefCell<Vec<Object>>,
}

#[derive(PartialEq, Clone)]
pub enum Datum {
    Integer(i64),
    Float(f64),
    Class(Rc<Class>),
    Closure(Rc<Closure>),
    Instance(Rc<Instance>),
    String(Rc<String>),
}

pub struct Builtins {
    closure_vtable: Rc<Vtable>,
    float_vtable: Rc<Vtable>,
    integer_vtable: Rc<Vtable>,
    string_vtable: Rc<Vtable>,
    pub globals: RefCell<HashMap<String, Object>>,
}

impl Builtins {
    pub fn new() -> Builtins {
        let mut globals = HashMap::new();

        let integer_vtable = Rc::new(classes::integer2::vtable());
        globals.insert(
            "Integer".to_string(),
            Object {
                vtable: Rc::new(Vtable::new("class Integer")),
                datum: Datum::Class(Rc::new(Class {
                    instance_vtable: Rc::clone(&integer_vtable),
                })),
            },
        );

        let float_vtable = Rc::new(classes::float2::vtable());
        globals.insert(
            "Float".to_string(),
            Object {
                vtable: Rc::new(Vtable::new("class Float")),
                datum: Datum::Class(Rc::new(Class {
                    instance_vtable: Rc::clone(&float_vtable),
                })),
            },
        );

        let string_vtable = Rc::new(classes::string2::instance_vtable());
        globals.insert(
            "String".to_string(),
            Object {
                vtable: Rc::new(classes::string2::class_vtable()),
                datum: Datum::Class(Rc::new(Class {
                    instance_vtable: Rc::clone(&string_vtable),
                })),
            },
        );

        Builtins {
            closure_vtable: Rc::new(classes::closure2::vtable()),
            float_vtable,
            integer_vtable,
            string_vtable,
            globals: RefCell::new(globals),
        }
    }

    pub fn make_class(&self, classdef: &ClassDefinition) -> Eval {
        let mut vtable_name = "class ".to_string();
        vtable_name.push_str(&classdef.name);
        let mut class_vtable = Vtable::new(vtable_name.as_str());
        class_vtable.def(&classdef.constructor(), generic_ctor);
        let mut instance_vtable = Vtable::new(&classdef.name);
        let mut index = 0;
        for var in &classdef.instance_variables {
            index += 1;
            let vtable = match &var.typename {
                None => None,
                Some(typename) => {
                    let slotclass = self.find_class(typename, var.span.clone())?.class();
                    Some(slotclass.instance_vtable.clone())
                }
            };
            instance_vtable.add_slot(&var.name, index - 1, vtable);
            if &var.name[0..1] == "_" {
                continue;
            }
            instance_vtable.add_reader(&var.name, index - 1);
        }
        for method in &classdef.methods {
            instance_vtable.add_method(
                &method.selector,
                self.make_method_function(&method.parameters, &method.body),
            );
        }
        Ok(Object {
            vtable: Rc::new(class_vtable),
            datum: Datum::Class(Rc::new(Class {
                instance_vtable: Rc::new(instance_vtable),
            })),
        })
    }

    pub fn find_class(&self, name: &str, span: Span) -> Eval {
        match self.globals.borrow().get(name) {
            None => return Unwind::exception(SyntaxError::new(span, "Unknown class")),
            Some(global) => match global.datum {
                Datum::Class(_) => Ok(global.to_owned()),
                _ => Unwind::exception(SyntaxError::new(span, "Not a class name")),
            },
        }
    }

    pub fn make_float(&self, x: f64) -> Object {
        Object {
            vtable: Rc::clone(&self.float_vtable),
            datum: Datum::Float(x),
        }
    }

    pub fn make_integer(&self, x: i64) -> Object {
        Object {
            vtable: Rc::clone(&self.integer_vtable),
            datum: Datum::Integer(x),
        }
    }

    pub fn make_closure(&self, frame: Frame, params: Vec<Arg>, body: Expr) -> Object {
        Object {
            vtable: Rc::clone(&self.closure_vtable),
            datum: Datum::Closure(Rc::new(Closure {
                env: Some(frame),
                params,
                body,
            })),
        }
    }

    pub fn make_string(&self, string: &str) -> Object {
        self.into_string(string.to_string())
    }

    pub fn into_string(&self, string: String) -> Object {
        Object {
            vtable: Rc::clone(&self.string_vtable),
            datum: Datum::String(Rc::new(string)),
        }
    }

    pub fn make_method_function(&self, params: &[Var], body: &Expr) -> Closure {
        let mut args = vec![];
        for param in params {
            let vtable = match &param.typename {
                None => None,
                // FIXME: unwrap
                Some(tn) => Some(
                    self.find_class(tn, param.span.clone())
                        .unwrap()
                        .class()
                        .instance_vtable
                        .clone(),
                ),
            };
            args.push(Arg::new(param.span.clone(), param.name.clone(), vtable));
        }
        Closure {
            env: None,
            params: args,
            body: body.to_owned(),
        }
    }
}

impl Object {
    pub fn class(&self) -> Rc<Class> {
        match &self.datum {
            Datum::Class(class) => Rc::clone(class),
            _ => panic!("BUG: {} is not a Class", self),
        }
    }

    pub fn closure_ref(&self) -> &Closure {
        match &self.datum {
            Datum::Closure(c) => c.borrow(),
            _ => panic!("BUG: {} is not a Closure", self),
        }
    }

    pub fn float(&self) -> f64 {
        match self.datum {
            Datum::Float(f) => f,
            _ => panic!("BUG: {} is not a Float", self),
        }
    }

    pub fn instance(&self) -> Rc<Instance> {
        match &self.datum {
            Datum::Instance(instance) => Rc::clone(instance),
            _ => panic!("BUG: {} is not an instance", self),
        }
    }

    pub fn integer(&self) -> i64 {
        match self.datum {
            Datum::Integer(i) => i,
            _ => panic!("BUG: {} is not an Integer", self),
        }
    }

    pub fn string(&self) -> Rc<String> {
        match &self.datum {
            Datum::String(s) => Rc::clone(s),
            _ => panic!("BUG: {} is not a String", self),
        }
    }

    pub fn string_as_str(&self) -> &str {
        match &self.datum {
            Datum::String(s) => s.as_str(),
            _ => panic!("BUG: {} is not a String", self),
        }
    }

    pub fn send(&self, message: &str, args: &[&Object], builtins: &Builtins) -> Eval {
        // println!("debug: {} {} {:?}", self, message, args);
        match self.vtable.get(message) {
            Some(Method::Primitive(method)) => method(self, args, builtins),
            Some(Method::Interpreter(closure)) => eval::apply(Some(self), closure, args, builtins),
            Some(Method::Reader(index)) => read_instance_variable(self, *index),
            None => {
                // println!("debug: available methods: {:?}", &self.vtable.selectors());
                unimplemented!("{} doesNotUnderstand {} {:?}", self, message, args);
            }
        }
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.datum {
            Datum::Integer(x) => write!(f, "{}", x),
            Datum::Float(x) => {
                if x - x.floor() == 0.0 {
                    write!(f, "{}.0", x)
                } else {
                    write!(f, "{}", x)
                }
            }
            Datum::Closure(x) => write!(f, "$<closure {:?}>", x.params),
            Datum::Class(_) => write!(f, "$<{}>", self.vtable.name),
            Datum::Instance(_) => write!(f, "$<instance {}>", self.vtable.name),
            Datum::String(s) => write!(f, "{}", s),
        }
    }
}

impl fmt::Debug for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.datum {
            Datum::Integer(x) => write!(f, "{}", x),
            Datum::Float(x) => {
                if x - x.floor() == 0.0 {
                    write!(f, "{}.0", x)
                } else {
                    write!(f, "{}", x)
                }
            }
            Datum::Closure(x) => write!(f, "Closure({:?})", x.env),
            Datum::Class(_) => write!(f, "{}", self.vtable.name),
            Datum::Instance(_) => write!(f, "{}", self.vtable.name),
            // FIXME: Escape double-quotes
            Datum::String(s) => write!(f, "\"{}\"", s),
        }
    }
}

fn generic_ctor(receiver: &Object, args: &[&Object], _builtins: &Builtins) -> Eval {
    let class = receiver.class();
    Ok(Object {
        vtable: Rc::clone(&class.instance_vtable),
        datum: Datum::Instance(Rc::new(Instance {
            instance_variables: RefCell::new(args.iter().map(|x| (*x).to_owned()).collect()),
        })),
    })
}

pub fn read_instance_variable(receiver: &Object, index: usize) -> Eval {
    let instance = receiver.instance();
    let value = instance.instance_variables.borrow()[index].clone();
    Ok(value)
}

pub fn write_instance_variable(receiver: &Object, slot: &Slot, value: Object) -> Eval {
    if let Some(vtable) = &slot.vtable {
        if &value.vtable != vtable {
            // FIXME: None as span
            return Unwind::exception(SyntaxError::new(0..0, "TypeError"));
        }
    }
    let instance = receiver.instance();
    instance.instance_variables.borrow_mut()[slot.index] = value.clone();
    Ok(value)
}
