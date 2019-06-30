use std::str::FromStr;

pub use crate::tokenstream::SyntaxError;
use crate::tokenstream::{Span, Token, TokenStream};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Literal {
    Decimal(i64),
    Hexadecimal(i64),
    Binary(i64),
    Float(f64),
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Constant(Span, Literal),
    LocalVariable(Span, String),
}

struct Parser<'a> {
    tokenstream: TokenStream<'a>,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str) -> Parser<'a> {
        Parser {
            tokenstream: TokenStream::new(source),
        }
    }

    fn parse(&mut self) -> Result<Expr, SyntaxError> {
        let token = self.tokenstream.scan()?;
        match token {
            Token::Number => self.parse_number(),
            Token::LocalId => self.parse_local_variable(),
            _ => unimplemented!("Don't know how to parse: {:?}", token),
        }
    }

    fn span(&self) -> Span {
        self.tokenstream.span()
    }

    fn error(&self, problem: &'static str) -> Result<Expr, SyntaxError> {
        self.tokenstream.error(problem)
    }

    fn parse_local_variable(&mut self) -> Result<Expr, SyntaxError> {
        let slice = self.tokenstream.slice(self.span());
        Ok(Expr::LocalVariable(self.span(), slice.to_string()))
    }

    fn parse_number(&mut self) -> Result<Expr, SyntaxError> {
        let slice = self.tokenstream.slice(self.span());
        // Hexadecimal case
        if slice.len() > 2 && ("0x" == &slice[0..2] || "0X" == &slice[0..2]) {
            let integer = match i64::from_str_radix(&slice[2..], 16) {
                Ok(i) => i,
                Err(_) => return self.error("Malformed hexadecimal number"),
            };
            Ok(Expr::Constant(self.span(), Literal::Hexadecimal(integer)))
        }
        // Binary case
        else if slice.len() > 2 && ("0b" == &slice[0..2] || "0B" == &slice[0..2]) {
            let integer = match i64::from_str_radix(&slice[2..], 2) {
                Ok(i) => i,
                Err(_) => return self.error("Malformed binary number"),
            };
            Ok(Expr::Constant(self.span(), Literal::Binary(integer)))
        }
        // Decimal and float case
        else {
            let mut decimal: i64 = 0;
            for byte in slice.bytes() {
                if byte < 128 {
                    let c = byte as char;
                    if c == '_' {
                        continue;
                    }
                    if c.is_digit(10) {
                        decimal = decimal * 10 + c.to_digit(10).unwrap() as i64;
                    } else {
                        match f64::from_str(slice) {
                            Ok(f) => return Ok(Expr::Constant(self.span(), Literal::Float(f))),
                            Err(_) => return self.error("Malformed number"),
                        }
                    }
                }
            }
            Ok(Expr::Constant(self.span(), Literal::Decimal(decimal)))
        }
    }
}

pub fn parse_str(source: &str) -> Result<Expr, SyntaxError> {
    Parser::new(source).parse()
}

fn decimal(span: Span, value: i64) -> Expr {
    Expr::Constant(span, Literal::Decimal(value))
}

fn hexadecimal(span: Span, value: i64) -> Expr {
    Expr::Constant(span, Literal::Hexadecimal(value))
}

fn binary(span: Span, value: i64) -> Expr {
    Expr::Constant(span, Literal::Binary(value))
}

fn float(span: Span, value: f64) -> Expr {
    Expr::Constant(span, Literal::Float(value))
}

fn var(span: Span, name: &str) -> Expr {
    Expr::LocalVariable(span, name.to_string())
}

#[test]
fn parse_decimal() {
    assert_eq!(parse_str("123"), Ok(decimal(0..3, 123)));
}

#[test]
fn parse_hexadecimal() {
    assert_eq!(parse_str("0xFF"), Ok(hexadecimal(0..4, 0xFF)));
}

#[test]
fn parse_binary() {
    assert_eq!(parse_str("0b101"), Ok(binary(0..5, 0b101)));
}

#[test]
fn parse_float1() {
    assert_eq!(parse_str("1.123"), Ok(float(0..5, 1.123)));
}

#[test]
fn parse_float2() {
    assert_eq!(parse_str("1.1e6"), Ok(float(0..5, 1.1e6)));
}

#[test]
fn parse_float3() {
    assert_eq!(parse_str("2e6"), Ok(float(0..3, 2e6)));
}

#[test]
fn parse_var() {
    assert_eq!(parse_str("foo"), Ok(var(0..3, "foo")));
}
