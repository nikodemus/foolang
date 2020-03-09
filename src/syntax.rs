use crate::def::Def;
use crate::expr::{Expr};

#[derive(Debug, PartialEq)]
pub enum Syntax {
    Def(Def),
    Expr(Expr),
}

#[cfg(test)]
impl Syntax {
    pub fn expr(self) -> Expr {
        match self {
            Syntax::Expr(e) => e,
            Syntax::Def(_) => panic!("Expr expected, got Def."),
        }
    }
    pub fn def(self) -> Def {
        match self {
            Syntax::Expr(_) => panic!("Def expected, got Expr."),
            Syntax::Def(d) => d
        }
    }
}
