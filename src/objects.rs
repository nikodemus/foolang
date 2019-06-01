use crate::ast;
use std::sync::Arc;

#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    Integer(i64),
    Float(f64),
    Character(Arc<String>),
    String(Arc<String>),
    Symbol(Arc<String>),
    Block(Arc<ast::Block>),
}

impl Object {
    pub fn make_string(s: &str) -> Object {
        Object::String(Arc::new(String::from(s)))
    }
    pub fn make_symbol(s: &str) -> Object {
        Object::Symbol(Arc::new(String::from(s)))
    }
    pub fn make_char(s: &str) -> Object {
        let s = String::from(s);
        assert!(s.len() == 1);
        Object::Character(Arc::new(s))
    }
}
