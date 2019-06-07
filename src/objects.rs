use crate::ast;
use crate::evaluator::Lexenv;
use std::fmt;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(PartialEq, Clone, Debug)]
pub struct ClassId(pub usize);

// NOTE: ALPHABETIC ORDER!
// Matches the order of builtin classes in evaluator.rs
pub const CLASS_ARRAY: ClassId = ClassId(0);
pub const CLASS_CHARACTER: ClassId = ClassId(1);
pub const CLASS_CLASS: ClassId = ClassId(2);
pub const CLASS_CLOSURE: ClassId = ClassId(3);
pub const CLASS_FLOAT: ClassId = ClassId(4);
pub const CLASS_INTEGER: ClassId = ClassId(5);
pub const CLASS_STDIN: ClassId = ClassId(6);
pub const CLASS_STDOUT: ClassId = ClassId(7);
pub const CLASS_STRING: ClassId = ClassId(8);
pub const CLASS_SYMBOL: ClassId = ClassId(9);

#[derive(Debug, PartialEq, Clone)]
pub struct Object {
    pub class: ClassId,
    pub datum: Datum,
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.datum {
            Datum::Integer(i) => write!(f, "{}", i),
            Datum::Float(x) => write!(f, "{}", x),
            Datum::Character(c) => write!(f, "${}", &c),
            Datum::String(s) => write!(f, r#"'{}'"#, &s),
            Datum::Symbol(s) => write!(f, "#{}", &s),
            Datum::Array(vec) => {
                write!(f, "#")?;
                let mut sep = "[";
                for elt in vec.iter() {
                    write!(f, "{}{}", sep, elt)?;
                    sep = " ";
                }
                write!(f, "]")
            }
            Datum::Class(class) => write!(f, "#<class {}>", class.name),
            Datum::Instance(_slot) => write!(f, "#<obj>"),
            Datum::Closure(_closure) => write!(f, "#<closure>"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ClassObject {
    pub id: ClassId,
    pub name: String,
}

#[derive(Debug)]
pub struct SlotObject {
    pub slots: Mutex<Vec<Object>>,
}

impl PartialEq for SlotObject {
    fn eq(&self, other: &Self) -> bool {
        self as *const _ == other as *const _
    }
}

#[derive(Debug)]
pub struct ClosureObject {
    pub block: ast::Block,
    pub env: Lexenv,
}

impl PartialEq for ClosureObject {
    fn eq(&self, other: &Self) -> bool {
        self as *const _ == other as *const _
    }
}

// FIXME: Should have the contained objects holding the
// Arc so things which are known to receive them could
// receive owned.
#[derive(Debug, PartialEq, Clone)]
pub enum Datum {
    Integer(i64),
    Float(f64),
    Character(Arc<String>),
    String(Arc<String>),
    Symbol(Arc<String>),
    Array(Arc<Vec<Object>>),
    Class(Arc<ClassObject>),
    Instance(Arc<SlotObject>),
    Closure(Arc<ClosureObject>),
}

impl Object {
    pub fn slot(&self, idx: usize) -> Object {
        if let Datum::Instance(obj) = &self.datum {
            obj.slots.lock().unwrap()[idx].clone()
        } else {
            panic!("Cannot access slot of a non-slot object.");
        }
    }
    pub fn set_slot(&self, idx: usize, val: Object) {
        if let Datum::Instance(obj) = &self.datum {
            obj.slots.lock().unwrap()[idx] = val;
        } else {
            panic!("Cannot access slot of a non-slot object.");
        }
    }
    pub fn into_closure(block: ast::Block, env: &Lexenv) -> Object {
        Object {
            class: CLASS_CLOSURE,
            datum: Datum::Closure(Arc::new(ClosureObject {
                block,
                env: env.to_owned(),
            })),
        }
    }
    pub fn make_class(meta: ClassId, id: ClassId, name: &str) -> Object {
        Object {
            class: meta,
            datum: Datum::Class(Arc::new(ClassObject {
                id,
                name: String::from(name),
            })),
        }
    }
    pub fn make_instance(class: ClassId, slots: Vec<Object>) -> Object {
        Object {
            class,
            datum: Datum::Instance(Arc::new(SlotObject {
                slots: Mutex::new(slots),
            })),
        }
    }
    pub fn make_float(x: f64) -> Object {
        Object {
            class: CLASS_FLOAT,
            datum: Datum::Float(x),
        }
    }
    pub fn make_integer(x: i64) -> Object {
        Object {
            class: CLASS_INTEGER,
            datum: Datum::Integer(x),
        }
    }
    pub fn make_string(s: &str) -> Object {
        Object::into_string(String::from(s))
    }
    pub fn into_string(s: String) -> Object {
        Object {
            class: CLASS_STRING,
            datum: Datum::String(Arc::new(s)),
        }
    }
    pub fn make_symbol(s: &str) -> Object {
        Object::into_symbol(String::from(s))
    }
    pub fn into_symbol(s: String) -> Object {
        Object {
            class: CLASS_SYMBOL,
            datum: Datum::Symbol(Arc::new(s)),
        }
    }
    pub fn make_character(s: &str) -> Object {
        Object::into_character(String::from(s))
    }
    pub fn into_character(s: String) -> Object {
        assert!(s.len() == 1);
        Object {
            class: CLASS_CHARACTER,
            datum: Datum::Character(Arc::new(s)),
        }
    }
    pub fn make_array(data: &[Object]) -> Object {
        Object::into_array(data.to_vec())
    }
    pub fn into_array(data: Vec<Object>) -> Object {
        Object {
            class: CLASS_ARRAY,
            datum: Datum::Array(Arc::new(data)),
        }
    }
}
