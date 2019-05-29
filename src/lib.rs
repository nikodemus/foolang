//use std::fmt::Debug;

#[derive(Debug)]
pub struct Ident(pub String);

impl Ident {
    pub fn append(mut self, s: &str) -> Ident {
        self.0.push_str(s);
        self
    }
}

pub fn format(e: Expr) -> String {
    format_with(e, 0, 0, String::new())
}

fn newline(&mut buf: String, indent: i32) {
    buf.push_str("\n");
    for _ in 0..(indent) {
        buf.push_str(" ");
    }
}

fn format_with(e: Expr, indent: i32, pos: i32, buf: String) -> String {
    match e {
       Int(i) => {
           let s = i.to_string();
           if (s.length() + pos > 80) {
               newline(&mut buf, indent + 2);
           }
           buf.push_str(s);
       }
       _ => {}
    }
    buf
}

#[derive(Debug)]
pub enum Expr {
    Int(i64),
    Float(f64),
    Symbol(Ident),
    Character(String),
    String(String),
    Array(Vec<Expr>),
    Variable(Ident),
    Return(Box<Expr>),
    Block(Vec<Ident>, Vec<Expr>),
    Assign(Ident, Box<Expr>),
    Unary(Box<Expr>, Ident),
    Binary(Box<Expr>, Ident, Box<Expr>),
    Keyword(Box<Expr>, Ident, Box<Expr>),
    Cascade(Box<Expr>, Vec<Cascade>),
}

#[derive(Debug)]
pub enum Cascade {
    Unary(Ident),
    Binary(Ident, Expr),
    Keyword(Ident, Expr, Option<Box<Cascade>>),
}

#[derive(Debug)]
pub enum Pattern {
    Unary(Ident),
    Binary(Ident, Ident),
    Keyword(Ident, Ident, Option<Box<Pattern>>),
}

#[derive(Debug)]
pub struct Method {
    pub pattern: Pattern,
    pub temporaries: Vec<Ident>,
    pub statements: Vec<Expr>,
}

pub fn prepend<T>(e: T, mut es: Vec<T>) -> Vec<T> {
    es.insert(0, e);
    es
}

pub fn chop(mut s: String) -> String {
    s.remove(0);
    s
}
