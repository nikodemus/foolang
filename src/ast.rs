#[derive(Debug, PartialEq)]
pub struct Identifier(pub String);

impl Identifier {
    pub fn concat(mut self, other: Identifier) -> Identifier {
        self.0.push_str(other.0.as_str());
        self
    }
}

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
    Keyword(Identifier, Vec<Expr>),
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Constant(Literal),
    Variable(Identifier),
    Unary(Box<Expr>, Identifier),
    Binary(Box<Expr>, Identifier, Box<Expr>),
    Keyword(Box<Expr>, Identifier, Vec<Expr>),
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
    Keyword(Identifier, Vec<Identifier>),
}
