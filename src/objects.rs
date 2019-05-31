use crate::ast;
use std::sync::Arc;

#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    Integer(i64),
    Float(f64),
    Block(Arc<ast::Block>),
}
