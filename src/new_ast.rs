// temporary
use crate::pratt::Expr;

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Decimal(i64),
    Float(f64),
    Character(char),
    Selector(String),
    String(String),
    Array(Vec<Literal>),
    Record(Vec<String>, Vec<Literal>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Message {
    pub selector: String,
    pub arguments: Vec<Expr>,
}

impl Message {
    pub fn no_position(mut self) -> Self {
        self.arguments = self
            .arguments
            .into_iter()
            .map(|arg| arg.no_position())
            .collect();
        self
    }
}
