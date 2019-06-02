use crate::ast;
use std::sync::Arc;

#[derive(PartialEq, Clone, Debug)]
pub struct ClassId(pub usize);

// NOTE: ALPHABETIC ORDER!
// Matches the order of builtin classes in evaluator.rs
pub const CLASS_ARRAY: ClassId = ClassId(0);
pub const CLASS_BLOCK: ClassId = ClassId(1);
pub const CLASS_CHARACTER: ClassId = ClassId(2);
pub const CLASS_FLOAT: ClassId = ClassId(3);
pub const CLASS_INTEGER: ClassId = ClassId(4);
pub const CLASS_STRING: ClassId = ClassId(5);
pub const CLASS_SYMBOL: ClassId = ClassId(6);

#[derive(Debug, PartialEq, Clone)]
pub struct Object {
    pub class: ClassId,
    pub datum: Datum,
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
}

impl Object {
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
