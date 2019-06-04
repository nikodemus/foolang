use crate::ast;
use std::sync::Arc;

#[derive(PartialEq, Clone, Debug)]
pub struct ClassId(pub usize);

// NOTE: ALPHABETIC ORDER!
// Matches the order of builtin classes in evaluator.rs
pub const CLASS_ARRAY: ClassId = ClassId(0);
pub const CLASS_BLOCK: ClassId = ClassId(1);
pub const CLASS_CHARACTER: ClassId = ClassId(2);
pub const CLASS_CLASS: ClassId = ClassId(3);
pub const CLASS_FLOAT: ClassId = ClassId(4);
pub const CLASS_INTEGER: ClassId = ClassId(5);
pub const CLASS_STRING: ClassId = ClassId(6);
pub const CLASS_SYMBOL: ClassId = ClassId(7);

#[derive(Debug, PartialEq, Clone)]
pub struct Object {
    pub class: ClassId,
    pub datum: Datum,
}

#[derive(Debug, PartialEq)]
pub struct ClassObject {
    pub id: ClassId,
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub struct SlotObject {
    pub slots: Vec<Object>,
}

// XXX: Would be nice to be able to switch between this and union
// depending on a build option!
#[derive(Debug, PartialEq, Clone)]
pub enum Datum {
    Integer(i64),
    Float(f64),
    Character(Arc<String>),
    String(Arc<String>),
    Symbol(Arc<String>),
    Block(Arc<ast::Block>),
    Array(Arc<Vec<Object>>),
    Class(Arc<ClassObject>),
    Instance(Arc<SlotObject>),
}

impl Object {
    pub fn slot(&self, idx: usize) -> Object {
        if let Datum::Instance(obj) = &self.datum {
            obj.slots[idx].clone()
        } else {
            panic!("Cannot access slot of a non-slot object.");
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
            datum: Datum::Instance(Arc::new(SlotObject { slots })),
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
    pub fn into_block(x: ast::Block) -> Object {
        Object {
            class: CLASS_BLOCK,
            datum: Datum::Block(Arc::new(x)),
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
