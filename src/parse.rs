pub use crate::tokenstream::SyntaxError;
use crate::tokenstream::{Span, Token, TokenStream};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Literal {
    Decimal(i64),
    Hex(i64),
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Constant(Span, Literal),
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
            Token::Number(span) => self.parse_number(span),
            _ => unimplemented!("Don't know how to parse: {:?}", token),
        }
    }
    fn parse_number(&mut self, span: Span) -> Result<Expr, SyntaxError> {
        let slice = self.tokenstream.slice(span.clone());
        if slice.len() > 2 && ("0x" == &slice[0..2] || "0X" == &slice[0..2]) {
            let integer = match i64::from_str_radix(&slice[2..], 16) {
                Ok(i) => i,
                Err(_) => return Err(SyntaxError::new(span, "Malformed hexadecimal")),
            };
            Ok(Expr::Constant(span, Literal::Hex(integer)))
        } else {
            let integer = match i64::from_str_radix(slice, 10) {
                Ok(i) => i,
                Err(_) => return Err(SyntaxError::new(span, "Malformed integer")),
            };
            Ok(Expr::Constant(span, Literal::Decimal(integer)))
        }
    }
}

pub fn parse_str(source: &str) -> Result<Expr, SyntaxError> {
    Parser::new(source).parse()
}

fn decimal(span: Span, value: i64) -> Expr {
    Expr::Constant(span, Literal::Decimal(value))
}

fn hex(span: Span, value: i64) -> Expr {
    Expr::Constant(span, Literal::Hex(value))
}

#[test]
fn parse_decimal() {
    assert_eq!(parse_str("123"), Ok(decimal(0..3, 123)));
}

#[test]
fn parse_hex() {
    assert_eq!(parse_str("0xFF"), Ok(hex(0..4, 0xFF)));
}
