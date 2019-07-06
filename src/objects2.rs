use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::eval::Frame;
use crate::parse::{Expr, SyntaxError};

use crate::classes;

// This will change into ControlFlow (Exception, ReturnTo)
pub type Value = Result<Object, SyntaxError>;

type MethodFunction = fn(&Object, &[&Object], &Builtins) -> Value;

#[derive(Clone)]
pub struct Vtable {
    pub name: String,
    pub methods: HashMap<String, MethodFunction>,
}

impl Vtable {
    pub fn new(class: &str) -> Vtable {
        Vtable {
            name: class.to_string(),
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
    vtable: Rc<Vtable>,
    datum: Datum,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Closure {
    pub env: Rc<Frame>,
    pub params: Vec<String>,
    pub body: Expr,
}

#[derive(PartialEq, Clone)]
pub enum Datum {
    Integer(i64),
    Float(f64),
    Class(Rc<Class>),
    Closure(Rc<Closure>),
}

#[derive(PartialEq)]
pub struct Class {
    pub instance_vtable: Rc<Vtable>,
    pub instance_variables: Vec<String>,
}

pub struct Builtins {
    closure_vtable: Rc<Vtable>,
    float_vtable: Rc<Vtable>,
    integer_vtable: Rc<Vtable>,
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
                    instance_variables: vec![],
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
                    instance_variables: vec![],
                })),
            },
        );

        Builtins {
            closure_vtable: Rc::new(classes::closure2::vtable()),
            float_vtable,
            integer_vtable,
            globals: RefCell::new(globals),
        }
    }

    pub fn make_class(&self, name: &str, instance_variables: &[String]) -> Object {
        let mut vtable_name = "class ".to_string();
        vtable_name.push_str(name);
        Object {
            vtable: Rc::new(Vtable::new(vtable_name.as_str())),
            datum: Datum::Class(Rc::new(Class {
                instance_vtable: Rc::new(Vtable::new(name)),
                instance_variables: instance_variables.iter().map(|x| x.to_owned()).collect(),
            })),
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

    pub fn make_closure(&self, frame: Rc<Frame>, params: Vec<String>, body: Expr) -> Object {
        Object {
            vtable: Rc::clone(&self.closure_vtable),
            datum: Datum::Closure(Rc::new(Closure {
                env: frame,
                params,
                body,
            })),
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

    pub fn closure(&self) -> Rc<Closure> {
        match &self.datum {
            Datum::Closure(c) => Rc::clone(c),
            _ => panic!("BUG: {} is not a Closure", self),
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
            Datum::Closure(x) => write!(f, "{:?}", *x),
            Datum::Class(_) => write!(f, "{}", self.vtable.name),
        }
    }
}
