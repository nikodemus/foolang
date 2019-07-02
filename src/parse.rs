use std::borrow::ToOwned;
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
    Variable(Span, String),
    Send(Span, Box<Expr>, String, Vec<Expr>),
}

type PrefixParser = fn(&mut Parser) -> Result<Expr, SyntaxError>;
type SuffixParser = fn(&mut Parser, Expr) -> Result<Expr, SyntaxError>;
type PrecedenceFunction = fn(&Parser, Span) -> Result<usize, SyntaxError>;

struct Syntax {
    parse_prefix: PrefixParser,
    parse_suffix: SuffixParser,
    precedence: PrecedenceFunction,
}

struct Operator {
    prefix_selector: Option<String>,
    suffix_selector: Option<String>,
    precedence: usize,
}

type SyntaxTable = HashMap<Token, Syntax>;
type OperatorTable = HashMap<String, Operator>;

struct Parser<'a> {
    tokenstream: TokenStream<'a>,
    syntax_table: SyntaxTable,
    operator_table: OperatorTable,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Parser<'a> {
        Parser {
            syntax_table: make_syntax_table(),
            operator_table: make_operator_table(),
            tokenstream: TokenStream::new(source),
        }
    }

    pub fn parse(&mut self) -> Result<Expr, SyntaxError> {
        self.parse_expr(0)
    }

    pub fn parse_expr(&mut self, precedence: usize) -> Result<Expr, SyntaxError> {
        let mut expr = self.parse_prefix()?;
        while precedence < self.precedence()? {
            expr = self.parse_suffix(expr)?;
        }
        Ok(expr)
    }

    fn parse_prefix(&mut self) -> Result<Expr, SyntaxError> {
        let token = self.tokenstream.scan()?;
        match self.syntax_table.get(&token) {
            Some(syntax) => (syntax.parse_prefix)(self),
            None => unimplemented!("Don't know how to parse {:?} in prefix position.", token),
        }
    }

    fn parse_suffix(&mut self, expr: Expr) -> Result<Expr, SyntaxError> {
        let token = self.tokenstream.scan()?;
        match self.syntax_table.get(&token) {
            Some(syntax) => (syntax.parse_suffix)(self, expr),
            None => unimplemented!("Don't know how to parse {:?} in suffix position.", token),
        }
    }

    fn precedence(&mut self) -> Result<usize, SyntaxError> {
        let (token, span) = self.tokenstream.lookahead()?;
        match self.syntax_table.get(&token) {
            Some(syntax) => (syntax.precedence)(self, span),
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

impl Syntax {
    fn define(
        table: &mut SyntaxTable,
        key: Token,
        parse_prefix: PrefixParser,
        parse_suffix: SuffixParser,
        precedence: PrecedenceFunction,
    ) {
        table.insert(
            key,
            Syntax {
                parse_prefix,
                parse_suffix,
                precedence,
            },
        );
    }
}

fn make_syntax_table() -> SyntaxTable {
    let mut table: SyntaxTable = HashMap::new();
    let t = &mut table;
    let def = Syntax::define;
    use Token::*;

    def(t, Number, number_prefix, invalid_suffix, precedence_invalid);
    def(t, LocalId, identifier_prefix, identifier_suffix, precedence_1000);
    def(t, Operator, operator_prefix, operator_suffix, operator_precedence_10_to_100);
    def(t, Eof, invalid_prefix, invalid_suffix, precedence_0);

    table
}

impl Operator {
    fn define(
        table: &mut OperatorTable,
        key: &str,
        prefix_selector: &str,
        suffix_selector: &str,
        precedence: usize,
    ) {
        assert!(key.len() > 0);
        assert!(10 <= precedence);
        assert!(precedence <= 100);
        table.insert(
            key.to_string(),
            Operator {
                prefix_selector: if prefix_selector.is_empty() {
                    None
                } else {
                    Some(prefix_selector.to_string())
                },
                suffix_selector: if suffix_selector.is_empty() {
                    None
                } else {
                    Some(suffix_selector.to_string())
                },
                precedence,
            },
        );
    }
}

fn make_operator_table() -> OperatorTable {
    let mut table: OperatorTable = HashMap::new();
    let t = &mut table;
    let def = Operator::define;

    def(t, "*", "", "mul:", 50);
    def(t, "/", "", "div:", 40);
    def(t, "+", "", "add:", 30);
    def(t, "-", "neg", "sub:", 30);

    table
}

fn precedence_invalid(_: &Parser, _: Span) -> Result<usize, SyntaxError> {
    // To guarantee it aways gets parsed.
    Ok(1001)
}

fn precedence_1000(_: &Parser, _: Span) -> Result<usize, SyntaxError> {
    Ok(1000)
}

fn precedence_0(_: &Parser, _: Span) -> Result<usize, SyntaxError> {
    Ok(0)
}

fn invalid_prefix(parser: &mut Parser) -> Result<Expr, SyntaxError> {
    parser.error("Not valid in value position")
}

fn invalid_suffix(parser: &mut Parser, _: Expr) -> Result<Expr, SyntaxError> {
    parser.error("Not valid in operator position")
}

fn identifier_prefix(parser: &mut Parser) -> Result<Expr, SyntaxError> {
    Ok(Expr::Variable(parser.span(), parser.tokenstring()))
}

fn identifier_suffix(parser: &mut Parser, left: Expr) -> Result<Expr, SyntaxError> {
    Ok(Expr::Send(parser.span(), Box::new(left), parser.tokenstring(), vec![]))
}

fn number_prefix(parser: &mut Parser) -> Result<Expr, SyntaxError> {
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

fn operator_precedence_10_to_100(parser: &Parser, span: Span) -> Result<usize, SyntaxError> {
    let precedence = match parser.operator_table.get(parser.slice_at(span.clone())) {
        Some(operator) => operator.precedence,
        None => return parser.error_at(span, "Unknown operator"),
    };
    Ok(precedence)
}

fn operator_prefix(parser: &mut Parser) -> Result<Expr, SyntaxError> {
    let span = parser.span();
    let selector = match parser.operator_table.get(parser.slice()) {
        Some(operator) => match operator.prefix_selector {
            Some(ref selector) => selector.to_owned(),
            None => return parser.error("Not a prefix operator"),
        },
        None => return parser.error("Unknown operator"),
    };
    Ok(Expr::Send(span, Box::new(parser.parse()?), selector, vec![]))
}

fn operator_suffix(parser: &mut Parser, left: Expr) -> Result<Expr, SyntaxError> {
    let span = parser.span();
    let (precedence, selector) = match parser.operator_table.get(parser.slice()) {
        Some(operator) => match operator.suffix_selector {
            Some(ref selector) => (operator.precedence, selector.to_owned()),
            None => return parser.error("Not a binary operator"),
        },
        None => return parser.error("Unknown operator"),
    };
    Ok(Expr::Send(span, Box::new(left), selector, vec![parser.parse_expr(precedence)?]))
}

/// Tests and tools

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
    Expr::Variable(span, name.to_string())
}

fn binary(span: Span, name: &str, left: Expr, right: Expr) -> Expr {
    Expr::Send(span, Box::new(left), name.to_string(), vec![right])
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

#[test]
fn parse_var2() {
    assert_eq!(parse_str(" c"), Ok(var(1..2, "c")));
}

#[test]
fn parse_operators() {
    assert_eq!(
        parse_str("a + b * c"),
        Ok(binary(
            2..3,
            "add:",
            var(0..1, "a"),
            binary(6..7, "mul:", var(4..5, "b"), var(8..9, "c"))
        ))
    );
    assert_eq!(
        parse_str("a * b + c"),
        Ok(binary(
            6..7,
            "add:",
            binary(2..3, "mul:", var(0..1, "a"), var(4..5, "b")),
            var(8..9, "c")
        ))
    );
}
