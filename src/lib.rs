//use std::fmt::Debug;

#[derive(Debug)]
pub struct Ident(pub String);

impl Ident {
    pub fn append(mut self, s: &str) -> Ident {
        self.0.push_str(s);
        self
    }
}

#[derive(Debug)]
pub enum Expr {
    Int(i64),
    Float(f64),
    Variable(Ident),
    Return(Box<Expr>),
    Block(Vec<Ident>, Vec<Expr>),
    Assign(Ident, Box<Expr>),
    Unary(Box<Expr>, Ident),
    Binary(Box<Expr>, Ident, Box<Expr>),
    Keyword(Box<Expr>, Ident, Box<Expr>),
}

pub fn prepend<T>(e: T, mut es: Vec<T>) -> Vec<T> {
    es.insert(0, e);
    es
}
