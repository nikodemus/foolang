use crate::ast::{ClassDescription, Expr, Method};

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(pub syntax);

pub fn parse_expr(s: &str) -> Expr {
    match syntax::ExpressionParser::new().parse(s) {
        Ok(e) => e,
        Err(e) => {
            panic!(format!("Could not parse expression: {}\nError: {}", s, e));
        }
    }
}

pub fn parse_method(s: &str) -> Method {
    match syntax::MethodParser::new().parse(s) {
        Ok(e) => e,
        Err(e) => {
            panic!(format!("Could not parse method: {}\nError: {}", s, e));
        }
    }
}

pub fn parse_class(s: &str) -> ClassDescription {
    match syntax::ClassParser::new().parse(s) {
        Ok(e) => e,
        Err(e) => {
            panic!(format!("Could not parse class: {}\nError: {}", s, e));
        }
    }
}
