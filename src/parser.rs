use crate::ast;

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(pub syntax);

pub fn parse_expr(s: &str) -> ast::Expr {
    match syntax::ExpressionParser::new().parse(s) {
        Ok(e) => e,
        Err(e) => {
            panic!(format!("Could not parse expression: {}\nError: {}", s, e));
        }
    }
}

pub fn parse_method(s: &str) -> ast::Method {
    match syntax::MethodParser::new().parse(s) {
        Ok(e) => e,
        Err(e) => {
            panic!(format!("Could not parse method: {}\nError: {}", s, e));
        }
    }
}

pub fn parse_class(s: &str) -> ast::ClassDescription {
    match syntax::ClassParser::new().parse(s) {
        Ok(e) => e,
        Err(e) => {
            panic!(format!("Could not parse class: {}\nError: {}", s, e));
        }
    }
}

pub fn parse_instance_method(s: &str) -> ast::MethodDescription {
    match syntax::InstanceMethodParser::new().parse(s) {
        Ok(e) => e,
        Err(e) => {
            panic!(format!(
                "Could not parse instance method: {}\nError: {}",
                s, e
            ));
        }
    }
}

pub fn parse_class_method(s: &str) -> ast::MethodDescription {
    match syntax::ClassMethodParser::new().parse(s) {
        Ok(e) => e,
        Err(e) => {
            panic!(format!("Could not parse class method: {}\nError: {}", s, e));
        }
    }
}
