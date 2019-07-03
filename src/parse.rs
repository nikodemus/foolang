use std::borrow::ToOwned;
use std::collections::HashMap;
use std::convert::Into;
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
    Bind(String, Box<Expr>, Box<Expr>),
    Send(Span, String, Box<Expr>, Vec<Expr>),
    Seq(Vec<Expr>),
}

type PrefixParser = fn(&mut Parser) -> Result<Expr, SyntaxError>;
type SuffixParser = fn(&mut Parser, Expr, PrecedenceFunction) -> Result<Expr, SyntaxError>;
type PrecedenceFunction = fn(&Parser, Span) -> Result<usize, SyntaxError>;

#[derive(Clone)]
enum Syntax {
    General(PrefixParser, SuffixParser, PrecedenceFunction),
    Operator(bool, bool, usize),
}

type TokenTable = HashMap<Token, Syntax>;
type NameTable = HashMap<String, Syntax>;

struct Parser<'a> {
    tokenstream: TokenStream<'a>,
    token_table: TokenTable,
    name_table: NameTable,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Parser<'a> {
        Parser {
            token_table: make_token_table(),
            name_table: make_name_table(),
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
        match self.token_table.get(&token) {
            Some(syntax) => {
                let syntax = syntax.to_owned();
                self.parse_prefix_syntax(syntax)
            }
            None => unimplemented!("Don't know how to parse {:?} in prefix position.", token),
        }
    }

    fn parse_suffix(&mut self, left: Expr) -> Result<Expr, SyntaxError> {
        let token = self.tokenstream.scan()?;
        match self.token_table.get(&token) {
            Some(syntax) => {
                let syntax = syntax.to_owned();
                self.parse_suffix_syntax(syntax, left)
            }
            None => unimplemented!("Don't know how to parse {:?} in suffix position.", token),
        }
    }

    fn precedence(&mut self) -> Result<usize, SyntaxError> {
        let (token, span) = self.tokenstream.lookahead()?;
        match self.token_table.get(&token).to_owned() {
            Some(syntax) => {
                let syntax = syntax.to_owned();
                self.syntax_precedence(syntax, span)
            }
            None => unimplemented!("No precedence defined for {:?}", token),
        }
    }

    fn parse_prefix_syntax(&mut self, syntax: Syntax) -> Result<Expr, SyntaxError> {
        match syntax {
            Syntax::General(prefix, _, _) => prefix(self),
            Syntax::Operator(is_prefix, _, _) if is_prefix => {
                let operator = self.tokenstring();
                Ok(Expr::Send(self.span(), operator, Box::new(self.parse()?), vec![]))
            }
            _ => self.error("Expected value or prefix operator"),
        }
    }

    fn parse_suffix_syntax(&mut self, syntax: Syntax, left: Expr) -> Result<Expr, SyntaxError> {
        match syntax {
            Syntax::General(_, suffix, precedence) => suffix(self, left, precedence),
            Syntax::Operator(_, is_binary, precedence) if is_binary => {
                let operator = self.tokenstring();
                Ok(Expr::Send(
                    self.span(),
                    operator,
                    Box::new(left),
                    vec![self.parse_expr(precedence)?],
                ))
            }
            _ => self.error("I don't understand"),
        }
    }

    fn syntax_precedence(&self, syntax: Syntax, span: Span) -> Result<usize, SyntaxError> {
        match syntax {
            Syntax::General(_, _, precedence) => precedence(self, span),
            Syntax::Operator(_, _, precedence) => Ok(precedence),
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
    fn def<A, T>(
        table: &mut HashMap<T, Syntax>,
        key: A,
        prefix_parser: PrefixParser,
        suffix_parser: SuffixParser,
        precedence_function: PrecedenceFunction,
    ) where
        T: std::cmp::Eq,
        T: std::hash::Hash,
        A: Into<T>,
    {
        table
            .insert(key.into(), Syntax::General(prefix_parser, suffix_parser, precedence_function));
    }
    fn op(table: &mut NameTable, key: &str, is_prefix: bool, is_binary: bool, precedence: usize) {
        assert!(key.len() > 0);
        assert!(is_prefix || is_binary);
        assert!(10 <= precedence);
        assert!(precedence <= 100);
        table.insert(key.to_string(), Syntax::Operator(is_prefix, is_binary, precedence));
    }
}

fn make_token_table() -> TokenTable {
    let mut table: TokenTable = HashMap::new();
    let t = &mut table;
    use Token::*;

    Syntax::def(t, Number, number_prefix, invalid_suffix, precedence_invalid);
    Syntax::def(t, LocalId, identifier_prefix, identifier_suffix, identifier_precedence);
    Syntax::def(t, Operator, operator_prefix, operator_suffix, operator_precedence);
    Syntax::def(t, Eof, invalid_prefix, invalid_suffix, precedence_0);

    table
}

fn make_name_table() -> NameTable {
    let mut table: NameTable = HashMap::new();
    let t = &mut table;

    Syntax::def(t, "let", let_prefix, invalid_suffix, precedence_0);
    Syntax::def(t, ",", invalid_prefix, comma_suffix, precedence_1);

    Syntax::op(t, "*", false, true, 50);
    Syntax::op(t, "/", false, true, 40);
    Syntax::op(t, "+", false, true, 30);
    Syntax::op(t, "-", true, true, 30);

    table
}

fn precedence_invalid(_: &Parser, _: Span) -> Result<usize, SyntaxError> {
    // To guarantee it aways gets parsed.
    Ok(1001)
}

fn precedence_1(_: &Parser, _: Span) -> Result<usize, SyntaxError> {
    Ok(1)
}

fn precedence_0(_: &Parser, _: Span) -> Result<usize, SyntaxError> {
    Ok(0)
}

fn invalid_prefix(parser: &mut Parser) -> Result<Expr, SyntaxError> {
    parser.error("Not valid in value position")
}

fn invalid_suffix(
    parser: &mut Parser,
    _: Expr,
    _: PrecedenceFunction,
) -> Result<Expr, SyntaxError> {
    parser.error("Not valid in operator position")
}

fn identifier_precedence(parser: &Parser, span: Span) -> Result<usize, SyntaxError> {
    match parser.name_table.get(parser.slice_at(span.clone())) {
        Some(syntax) => parser.syntax_precedence(syntax.to_owned(), span),
        None => return Ok(1000), // unary messages
    }
}

fn identifier_prefix(parser: &mut Parser) -> Result<Expr, SyntaxError> {
    match parser.name_table.get(parser.slice()) {
        Some(syntax) => {
            let syntax = syntax.to_owned();
            parser.parse_prefix_syntax(syntax)
        }
        None => return Ok(Expr::Variable(parser.span(), parser.tokenstring())),
    }
}

fn identifier_suffix(
    parser: &mut Parser,
    left: Expr,
    _: PrecedenceFunction,
) -> Result<Expr, SyntaxError> {
    match parser.name_table.get(parser.slice()) {
        Some(syntax) => {
            let syntax = syntax.to_owned();
            parser.parse_suffix_syntax(syntax.to_owned(), left)
        }
        None => {
            // Unary message
            Ok(Expr::Send(parser.span(), parser.tokenstring(), Box::new(left), vec![]))
        }
    }
}

fn operator_precedence(parser: &Parser, span: Span) -> Result<usize, SyntaxError> {
    let slice = parser.slice_at(span.clone());
    match parser.name_table.get(slice) {
        Some(syntax) => {
            let syntax = syntax.to_owned();
            parser.syntax_precedence(syntax, span)
        }
        None => parser.error_at(span, "Unknown operator"),
    }
}

fn operator_prefix(parser: &mut Parser) -> Result<Expr, SyntaxError> {
    match parser.name_table.get(parser.slice()) {
        Some(syntax) => {
            let syntax = syntax.to_owned();
            parser.parse_prefix_syntax(syntax)
        }
        None => parser.error("Unknown operator"),
    }
}

fn operator_suffix(
    parser: &mut Parser,
    left: Expr,
    _: PrecedenceFunction,
) -> Result<Expr, SyntaxError> {
    match parser.name_table.get(parser.slice()) {
        Some(syntax) => {
            let syntax = syntax.to_owned();
            parser.parse_suffix_syntax(syntax, left)
        }
        None => parser.error("Unknown operator"),
    }
}

fn comma_suffix(
    parser: &mut Parser,
    left: Expr,
    precedence: PrecedenceFunction,
) -> Result<Expr, SyntaxError> {
    let mut exprs = if let Expr::Seq(left_exprs) = left {
        left_exprs
    } else {
        vec![left]
    };
    let right = parser.parse_expr(precedence(parser, parser.span())?)?;
    if let Expr::Seq(mut right_exprs) = right {
        exprs.append(&mut right_exprs);
    } else {
        exprs.push(right);
    }
    Ok(Expr::Seq(exprs))
}

fn let_prefix(parser: &mut Parser) -> Result<Expr, SyntaxError> {
    if Token::LocalId != parser.tokenstream.scan()? {
        return parser.error("Expected variable name after let");
    }
    let name = parser.slice().to_string();
    let next = parser.tokenstream.scan()?;
    if Token::Operator != next || "=" != parser.slice() {
        return parser.error("Expected = in let");
    }
    // FIXME: hardcoded!
    let value = parser.parse_expr(1)?;
    let next = parser.tokenstream.scan()?;
    if Token::Operator != next {
        return parser.error("Expected separator after let");
    }
    let body = parser.parse()?;
    Ok(Expr::Bind(name, Box::new(value), Box::new(body)))
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

/// Tests and tools

pub fn parse_str(source: &str) -> Result<Expr, SyntaxError> {
    Parser::new(source).parse().map_err(|e| e.add_context(source))
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

fn unary(span: Span, name: &str, left: Expr) -> Expr {
    Expr::Send(span, name.to_string(), Box::new(left), vec![])
}

fn binary(span: Span, name: &str, left: Expr, right: Expr) -> Expr {
    Expr::Send(span, name.to_string(), Box::new(left), vec![right])
}

fn bind(name: &str, value: Expr, body: Expr) -> Expr {
    Expr::Bind(name.to_string(), Box::new(value), Box::new(body))
}

fn seq(exprs: Vec<Expr>) -> Expr {
    Expr::Seq(exprs)
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
fn parse_operators1() {
    assert_eq!(
        parse_str("a + b * c"),
        Ok(binary(2..3, "+", var(0..1, "a"), binary(6..7, "*", var(4..5, "b"), var(8..9, "c"))))
    );
}

#[test]
fn parse_operators2() {
    assert_eq!(
        parse_str("a * b + c"),
        Ok(binary(6..7, "+", binary(2..3, "*", var(0..1, "a"), var(4..5, "b")), var(8..9, "c")))
    );
}

#[test]
fn parse_sequence() {
    assert_eq!(
        parse_str("foo bar, quux"),
        Ok(seq(vec![unary(4..7, "bar", var(0..3, "foo")), var(9..13, "quux")]))
    );
}

#[test]
fn parse_let() {
    assert_eq!(
        parse_str("let x = 21 + 21, x"),
        Ok(bind("x", binary(11..12, "+", int(8..10, 21), int(13..15, 21)), var(17..18, "x")))
    );
}
