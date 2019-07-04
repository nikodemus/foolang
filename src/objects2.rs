use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::parse::SyntaxError;

use crate::classes;

// This will change into ControlFlow (Exception, ReturnTo)
pub type Value = Result<Object, SyntaxError>;

type MethodFunction = fn(&Object, &[&Object], &Builtins) -> Value;

#[derive(Clone)]
pub struct Vtable {
    class: String,
    methods: HashMap<String, MethodFunction>,
}

impl Vtable {
    pub fn new(class: &str) -> Vtable {
        Vtable {
            class: class.to_string(),
            methods: HashMap::new(),
        }
    }

    pub fn def(&mut self, name: &str, method: MethodFunction) {
        self.methods.insert(name.to_string(), method);
    }

    pub fn get(&self, name: &str) -> Option<&MethodFunction> {
        self.methods.get(name)
    }
}

impl fmt::Debug for Vtable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Vtable[{}]", self.class)
    }
}

impl PartialEq for Vtable {
    fn eq(&self, other: &Self) -> bool {
        self as *const _ == other as *const _
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Object {
    pub vtable: Rc<Vtable>,
    pub datum: Datum,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Datum {
    Integer(i64),
    Float(f64),
}

pub struct Builtins {
    float_vtable: Rc<Vtable>,
    integer_vtable: Rc<Vtable>,
}

impl Builtins {
    pub fn new() -> Builtins {
        Builtins {
            float_vtable: Rc::new(classes::float2::vtable()),
            integer_vtable: Rc::new(classes::integer2::vtable()),
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
}

impl Object {
    pub fn float(&self) -> f64 {
        match self.datum {
            Datum::Float(f) => f,
            _ => panic!("BUG: {} is not a Float", self),
        }
    }

    pub fn integer(&self) -> i64 {
        match self.datum {
            Datum::Integer(i) => i,
            _ => panic!("BUG: {} is not an Integer", self),
        }
    }

    pub fn send(&self, message: &str, args: &[&Object], builtins: &Builtins) -> Value {
        println!("debug: {} {} {:?}", self, message, args);
        match self.vtable.get(message) {
            Some(method) => method(self, args, builtins),
            None => unimplemented!("Object::send() message not understood"),
        }
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.datum {
            Datum::Integer(x) => write!(f, "{}", x),
            Datum::Float(x) => {
                if x - x.floor() == 0.0 {
                    write!(f, "{}.0", x)
                } else {
                    write!(f, "{}", x)
                }
            }
        }
    }
}
