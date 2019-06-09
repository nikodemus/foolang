#[derive(Debug, PartialEq, Clone)]
pub struct Identifier(pub String);

impl Identifier {
    pub fn concat(mut self, other: Identifier) -> Identifier {
        self.0.push_str(other.0.as_str());
        self
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub parameters: Vec<Identifier>,
    pub temporaries: Vec<Identifier>,
    pub statements: Vec<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    Character(String),
    Symbol(String),
    String(String),
    Array(Vec<Literal>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Cascade {
    Message(Identifier, Vec<Expr>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Constant(Literal),
    Variable(Identifier),
    Send(Box<Expr>, Identifier, Vec<Expr>),
    Block(Block),
    Assign(Identifier, Box<Expr>),
    Return(Box<Expr>),
    Cascade(Box<Expr>, Vec<Cascade>),
    ArrayCtor(Vec<Expr>),
}

pub enum ProgramElement {
    Expr(Expr),
    Definition(Definition),
}

pub type Program = Vec<Definition>;

#[derive(Debug, PartialEq, Clone)]
pub enum Definition {
    Class(ClassDescription),
    InstanceMethod(MethodDescription),
    ClassMethod(MethodDescription),
}

#[derive(Debug, PartialEq, Clone)]
pub struct ClassDescription {
    pub name: Identifier,
    pub slots: Vec<Identifier>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MethodDescription {
    pub class: Identifier,
    pub method: Method,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Method {
    pub selector: Identifier,
    pub parameters: Vec<Identifier>,
    pub temporaries: Vec<Identifier>,
    pub docstring: Option<String>,
    pub statements: Vec<Expr>,
}
