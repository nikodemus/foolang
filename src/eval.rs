use crate::objects::Object;
use crate::parse::{parse_str, Expr, Literal, SyntaxError};
struct Env {}

impl Env {
    pub fn new() -> Env {
        Env {}
    }
    pub fn eval(&mut self, expr: Expr) -> Result<Object, SyntaxError> {
        match expr {
            Expr::Constant(_, literal) => self.eval_literal(literal),
        }
    }
    pub fn eval_literal(&self, literal: Literal) -> Result<Object, SyntaxError> {
        match literal {
            Literal::Decimal(value) => Ok(Object::make_integer(value)),
            Literal::Hexadecimal(value) => Ok(Object::make_integer(value)),
            Literal::Binary(value) => Ok(Object::make_integer(value)),
        }
    }
}

fn eval_str(source: &str) -> Result<Object, SyntaxError> {
    let expr = parse_str(source).map_err(|e| e.add_context(source))?;
    Env::new().eval(expr).map_err(|e| e.add_context(source))
}

fn integer(value: i64) -> Object {
    Object::make_integer(value)
}

#[test]
fn eval_decimal() {
    assert_eq!(eval_str("123"), Ok(integer(123)));
}

#[test]
fn eval_bad_decimal() {
    assert_eq!(
        eval_str("1x3"),
        Err(SyntaxError {
            span: 0..3,
            problem: "Malformed integer",
            context: concat!("001 1x3\n", "    ^^^ Malformed integer\n").to_string()
        })
    );
}

#[test]
fn eval_hex() {
    assert_eq!(eval_str("0xFFFF"), Ok(integer(0xFFFF)));
}

#[test]
fn eval_bad_hex() {
    assert_eq!(
        eval_str("0x1x3"),
        Err(SyntaxError {
            span: 0..5,
            problem: "Malformed hexadecimal number",
            context: concat!("001 0x1x3\n", "    ^^^^^ Malformed hexadecimal number\n").to_string()
        })
    );
}

#[test]
fn eval_binary() {
    assert_eq!(eval_str("0b101"), Ok(integer(0b101)));
}

#[test]
fn eval_bad_binary() {
    assert_eq!(
        eval_str("0b123"),
        Err(SyntaxError {
            span: 0..5,
            problem: "Malformed binary number",
            context: concat!("001 0b123\n", "    ^^^^^ Malformed binary number\n").to_string()
        })
    );
}
