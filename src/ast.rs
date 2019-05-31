#[derive(Debug, PartialEq)]
pub struct Identifier(pub String);

#[derive(Debug, PartialEq)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    Character(String),
    Symbol(String),
    String(String),
    Array(Vec<Literal>),
}

#[derive(Debug, PartialEq)]
pub enum Cascade {
    Unary(Identifier),
    Binary(Identifier, Expr),
    Keyword(Vec<Identifier>, Vec<Expr>),
}

#[derive(Debug, PartialEq)]
pub enum Expr {


Constant(Literal),
    Variable(Identifier),
    Unary(Box<Expr>, Identifier),
    Binary(Box<Expr>, Identifier, Box<Expr>),
    Keyword(Box<Expr>, Vec<Identifier>, Vec<Expr>),
    Assign(Identifier, Box<Expr>),
    Return(Box<Expr>),
    Block(Vec<Identifier>, Vec<Expr>),
    Cascade(Box<Expr>, Vec<Cascade>),
}

#[derive(Debug, PartialEq)]
pub struct Method {
    pub pattern: Pattern,
    pub temporaries: Vec<Identifier>,
    pub statements: Vec<Expr>,
}

#[derive(Debug, PartialEq)]
pub enum Pattern {
    Unary(Identifier),
    Binary(Identifier, Identifier),
    Keyword(Vec<Identifier>, Vec<Identifier>),
}
