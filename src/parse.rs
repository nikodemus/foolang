use std::collections::HashMap;
use std::str::FromStr;

pub use crate::tokenstream::SyntaxError;
use crate::tokenstream::{Span, Token, TokenStream};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Literal {
    Integer(i64),
    Float(f64),
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Constant(Span, Literal),
    LocalVariable(Span, String),
}

type PrefixParser = fn(&mut Parser) -> Result<Expr, SyntaxError>;
type PrefixTable = HashMap<Token, PrefixParser>;

type SuffixParser = fn(&mut Parser, Expr) -> Result<Expr, SyntaxError>;
type SuffixTable = HashMap<Token, SuffixParser>;

type PrecedenceFunction = fn(&Parser, Span) -> Result<usize, SyntaxError>;
type PrecedenceTable = HashMap<Token, PrecedenceFunction>;

struct Parser<'a> {
    tokenstream: TokenStream<'a>,
    prefix_table: PrefixTable,
    suffix_table: SuffixTable,
    precedence_table: PrecedenceTable,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Parser<'a> {
        Parser {
            prefix_table: make_prefix_table(),
            suffix_table: make_suffix_table(),
            precedence_table: make_precedence_table(),
            tokenstream: TokenStream::new(source),
        }
    }

    pub fn parse(&mut self) -> Result<Expr, SyntaxError> {
        self.parse_expr(0)
    }

    fn parse_expr(&mut self, precedence: usize) -> Result<Expr, SyntaxError> {
        let mut expr = self.parse_prefix()?;
        while precedence < self.precedence()? {
            expr = self.parse_suffix(expr)?;
        }
        Ok(expr)
    }

    fn parse_prefix(&mut self) -> Result<Expr, SyntaxError> {
        let token = self.tokenstream.scan()?;
        match self.prefix_table.get(&token) {
            Some(parser) => parser(self),
            None => unimplemented!("Don't know how to parse {:?} in prefix position.", token),
        }
    }

    fn parse_suffix(&mut self, expr: Expr) -> Result<Expr, SyntaxError> {
        let token = self.tokenstream.scan()?;
        match self.suffix_table.get(&token) {
            Some(parser) => parser(self, expr),
            None => unimplemented!("Don't know how to parse {:?} in suffix position.", token),
        }
    }

    fn precedence(&mut self) -> Result<usize, SyntaxError> {
        let (token, span) = self.tokenstream.lookahead()?;
        match self.precedence_table.get(&token) {
            Some(func) => func(self, span),
            None => unimplemented!("No precedence defined for {:?}", token),
        }
    }

    pub fn span(&self) -> Span {
        self.tokenstream.span()
    }

    pub fn slice(&self) -> &str {
        self.tokenstream.slice()
    }

    pub fn slice_at(&self, span: Span) -> &str {
        self.tokenstream.slice_at(span)
    }

    pub fn tokenstring(&self) -> String {
        self.tokenstream.tokenstring()
    }

    pub fn error<T>(&self, problem: &'static str) -> Result<T, SyntaxError> {
        self.tokenstream.error(problem)
    }

    pub fn error_at<T>(&self, span: Span, problem: &'static str) -> Result<T, SyntaxError> {
        self.tokenstream.error_at(span, problem)
    }
}

fn make_prefix_table() -> PrefixTable {
    let mut table: PrefixTable = HashMap::new();
    table.insert(Token::LocalId, parse_local_variable);
    table.insert(Token::Number, parse_number);
    table
}

fn make_suffix_table() -> SuffixTable {
    let mut table: SuffixTable = HashMap::new();
    table
}

fn make_precedence_table() -> PrecedenceTable {
    let mut table: PrecedenceTable = HashMap::new();
    table.insert(Token::Eof, precedence_0);
    table.insert(Token::Sigil, precedence_sigil);
    table
}

fn precedence_0(_: &Parser, _: Span) -> Result<usize, SyntaxError> {
    Ok(0)
}

fn precedence_sigil(parser: &Parser, span: Span) -> Result<usize, SyntaxError> {
    match parser.slice_at(span.clone()) {
        _ => parser.error_at(span, "Unknown sigil"),
    }
}

fn parse_local_variable(parser: &mut Parser) -> Result<Expr, SyntaxError> {
    Ok(Expr::LocalVariable(parser.span(), parser.tokenstring()))
}

fn parse_number(parser: &mut Parser) -> Result<Expr, SyntaxError> {
    let slice = parser.slice();
    // Hexadecimal case
    if slice.len() > 2 && ("0x" == &slice[0..2] || "0X" == &slice[0..2]) {
        let integer = match i64::from_str_radix(&slice[2..], 16) {
            Ok(i) => i,
            Err(_) => return parser.error("Malformed hexadecimal number"),
        };
        Ok(Expr::Constant(parser.span(), Literal::Integer(integer)))
    }
    // Binary case
    else if slice.len() > 2 && ("0b" == &slice[0..2] || "0B" == &slice[0..2]) {
        let integer = match i64::from_str_radix(&slice[2..], 2) {
            Ok(i) => i,
            Err(_) => return parser.error("Malformed binary number"),
        };
        Ok(Expr::Constant(parser.span(), Literal::Integer(integer)))
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
                        Ok(f) => return Ok(Expr::Constant(parser.span(), Literal::Float(f))),
                        Err(_) => return parser.error("Malformed number"),
                    }
                }
            }
        }
        Ok(Expr::Constant(parser.span(), Literal::Integer(decimal)))
    }
}

pub fn parse_str(source: &str) -> Result<Expr, SyntaxError> {
    Parser::new(source).parse()
}

fn int(span: Span, value: i64) -> Expr {
    Expr::Constant(span, Literal::Integer(value))
}

fn float(span: Span, value: f64) -> Expr {
    Expr::Constant(span, Literal::Float(value))
}

fn var(span: Span, name: &str) -> Expr {
    Expr::LocalVariable(span, name.to_string())
}

#[test]
fn parse_decimal() {
    assert_eq!(parse_str("123"), Ok(int(0..3, 123)));
}

#[test]
fn parse_hexadecimal() {
    assert_eq!(parse_str("0xFF"), Ok(int(0..4, 0xFF)));
}

#[test]
fn parse_binary() {
    assert_eq!(parse_str("0b101"), Ok(int(0..5, 0b101)));
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
