use std::borrow::ToOwned;
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::Into;
use std::str::FromStr;

use crate::objects2::Unwind;
use crate::tokenstream::{Span, Token, TokenStream};

// FIXME: Do these really need clone, if so, why?

#[derive(Debug, PartialEq, Clone)]
pub struct Assign {
    pub span: Span,
    pub name: String,
    pub value: Box<Expr>,
}

impl Assign {
    fn expr(span: Span, name: String, value: Expr) -> Expr {
        Expr::Assign(Assign {
            span,
            name,
            value: Box::new(value),
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ClassDefinition {
    pub span: Span,
    pub name: String,
    pub instance_variables: Vec<String>,
    pub methods: Vec<MethodDefinition>,
    default_constructor: Option<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MethodDefinition {
    pub span: Span,
    pub selector: String,
    pub parameters: Vec<String>,
    pub body: Box<Expr>,
}

impl ClassDefinition {
    fn new(span: Span, name: String, instance_variables: Vec<String>) -> ClassDefinition {
        ClassDefinition {
            span,
            name,
            instance_variables,
            methods: Vec::new(),
            default_constructor: None,
        }
    }

    fn expr(span: Span, name: String, instance_variables: Vec<String>) -> Expr {
        Expr::ClassDefinition(ClassDefinition::new(span, name, instance_variables))
    }

    fn add_method(&mut self, method: MethodDefinition) {
        self.methods.push(method);
    }

    pub fn constructor(&self) -> String {
        if self.instance_variables.is_empty() {
            match &self.default_constructor {
                Some(ctor) => ctor.to_string(),
                None => "new".to_string(),
            }
        } else {
            let mut selector = String::new();
            for var in &self.instance_variables {
                selector.push_str(var);
                selector.push_str(":");
            }
            selector
        }
    }
}

impl MethodDefinition {
    fn new(span: Span, selector: String, parameters: Vec<String>, body: Expr) -> MethodDefinition {
        MethodDefinition {
            span,
            selector,
            parameters,
            body: Box::new(body),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Global {
    pub span: Span,
    pub name: String,
}

impl Global {
    fn expr(span: Span, name: String) -> Expr {
        Expr::Global(Global {
            span,
            name,
        })
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Literal {
    Integer(i64),
    Float(f64),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Return {
    pub span: Span,
    pub value: Box<Expr>,
}

impl Return {
    fn expr(span: Span, value: Expr) -> Expr {
        Expr::Return(Return {
            span,
            value: Box::new(value),
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Var {
    pub span: Span,
    pub name: String,
}

impl Var {
    fn expr(span: Span, name: String) -> Expr {
        Expr::Var(Var {
            span,
            name,
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Assign(Assign),
    Bind(String, Box<Expr>, Box<Expr>),
    Block(Span, Vec<String>, Box<Expr>),
    ClassDefinition(ClassDefinition),
    Const(Span, Literal),
    Global(Global),
    Send(Span, String, Box<Expr>, Vec<Expr>),
    Seq(Vec<Expr>),
    Var(Var),
    Return(Return),
}

impl Expr {
    fn is_var(&self) -> bool {
        match self {
            Expr::Var(..) => true,
            _ => false,
        }
    }

    fn add_method(&mut self, method: MethodDefinition) {
        match self {
            Expr::ClassDefinition(class) => class.add_method(method),
            _ => panic!("BUG: trying to add a method to {:?}", self),
        }
    }

    fn name(&self) -> String {
        match self {
            Expr::Var(var) => var.name.to_owned(),
            _ => panic!("BUG: cannot extract name from {:?}", self),
        }
    }

    fn span(&self) -> Span {
        use Expr::*;
        let span = match self {
            Assign(assign) => &assign.span,
            Bind(_, _, body) => return body.span(),
            Block(span, ..) => span,
            ClassDefinition(definition) => &definition.span,
            Const(span, ..) => span,
            Global(global) => &global.span,
            Send(span, ..) => span,
            Return(ret) => &ret.span,
            Seq(exprs) => return exprs[exprs.len() - 1].span(),
            Var(var) => &var.span,
        };
        span.to_owned()
    }
}

type PrefixParser = fn(&Parser) -> Result<Expr, Unwind>;
type SuffixParser = fn(&Parser, Expr, PrecedenceFunction) -> Result<Expr, Unwind>;
type PrecedenceFunction = fn(&Parser, Span) -> Result<usize, Unwind>;

#[derive(Clone)]
enum Syntax {
    General(PrefixParser, SuffixParser, PrecedenceFunction),
    Operator(bool, bool, usize),
}

type TokenTable = HashMap<Token, Syntax>;
type NameTable = HashMap<String, Syntax>;

pub struct ParserState<'a> {
    tokenstream: TokenStream<'a>,
    lookahead: Option<(Token, Span)>,
    span: Span,
}

impl<'a> ParserState<'a> {
    fn scan(&mut self) -> Result<(Token, Span), Unwind> {
        let token = self.tokenstream.scan()?;
        Ok((token, self.tokenstream.span()))
    }

    fn next_token(&mut self) -> Result<Token, Unwind> {
        let (token, span) = match self.lookahead.take() {
            None => self.scan()?,
            Some(look) => look,
        };
        self.span = span;
        Ok(token)
    }

    fn lookahead(&mut self) -> Result<(Token, Span), Unwind> {
        if let Some(look) = &self.lookahead {
            return Ok(look.clone());
        }
        let look = self.scan()?;
        self.lookahead = Some(look.clone());
        Ok(look)
    }
}

pub struct Parser<'a> {
    source: &'a str,
    token_table: TokenTable,
    name_table: NameTable,
    state: RefCell<ParserState<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Parser<'a> {
        Parser {
            source,
            token_table: make_token_table(),
            name_table: make_name_table(),
            state: RefCell::new(ParserState {
                tokenstream: TokenStream::new(source),
                lookahead: None,
                span: 0..0,
            }),
        }
    }

    pub fn parse(&mut self) -> Result<Expr, Unwind> {
        self.parse_expr(0)
    }

    pub fn parse_expr(&self, precedence: usize) -> Result<Expr, Unwind> {
        let mut expr = self.parse_prefix()?;
        let debug = false;
        if debug {
            println!("prefix: {:?}", &expr);
        }
        while precedence < self.precedence()? {
            if debug {
                println!("  {} < {}", precedence, self.precedence()?);
            }
            expr = self.parse_suffix(expr)?;
            if debug {
                println!("  => {:?}", &expr);
            }
        }
        if debug {
            println!("  NOT {} < {}", precedence, self.precedence()?);
        }
        if debug {
            println!("  next: {:?}", self.lookahead()?);
        }
        return Ok(expr);
    }

    fn parse_prefix(&self) -> Result<Expr, Unwind> {
        let token = self.next_token()?;
        match self.token_table.get(&token) {
            Some(syntax) => self.parse_prefix_syntax(syntax),
            None => unimplemented!("Don't know how to parse {:?} in prefix position.", token),
        }
    }

    fn parse_suffix(&self, left: Expr) -> Result<Expr, Unwind> {
        let token = self.next_token()?;
        match self.token_table.get(&token) {
            Some(syntax) => self.parse_suffix_syntax(syntax, left),
            None => unimplemented!("Don't know how to parse {:?} in suffix position.", token),
        }
    }

    fn precedence(&self) -> Result<usize, Unwind> {
        let (token, span) = self.lookahead()?;
        match self.token_table.get(&token) {
            Some(syntax) => self.syntax_precedence(syntax, span),
            None => unimplemented!("No precedence defined for {:?}", token),
        }
    }

    fn parse_prefix_syntax(&self, syntax: &Syntax) -> Result<Expr, Unwind> {
        match syntax {
            Syntax::General(prefix, _, _) => prefix(self),
            Syntax::Operator(is_prefix, _, _) if *is_prefix => {
                let operator = self.tokenstring();
                Ok(Expr::Send(self.span(), operator, Box::new(self.parse_expr(0)?), vec![]))
            }
            _ => self.error("Expected value or prefix operator"),
        }
    }

    fn parse_suffix_syntax(&self, syntax: &Syntax, left: Expr) -> Result<Expr, Unwind> {
        match syntax {
            Syntax::General(_, suffix, precedence) => suffix(self, left, *precedence),
            Syntax::Operator(_, is_binary, precedence) if *is_binary => {
                let operator = self.tokenstring();
                Ok(Expr::Send(
                    self.span(),
                    operator,
                    Box::new(left),
                    vec![self.parse_expr(*precedence)?],
                ))
            }
            _ => self.error("I don't understand"),
        }
    }

    fn syntax_precedence(&self, syntax: &Syntax, span: Span) -> Result<usize, Unwind> {
        match syntax {
            Syntax::General(_, _, precedence) => precedence(self, span),
            Syntax::Operator(_, _, precedence) => Ok(*precedence),
        }
    }

    pub fn lookahead(&self) -> Result<(Token, Span), Unwind> {
        self.state.borrow_mut().lookahead()
    }

    pub fn next_token(&self) -> Result<Token, Unwind> {
        self.state.borrow_mut().next_token()
    }

    pub fn at_eof(&self) -> bool {
        if let Ok((Token::EOF, _)) = self.lookahead() {
            return true;
        } else {
            return false;
        }
    }

    pub fn span(&self) -> Span {
        self.state.borrow().span.clone()
    }

    pub fn slice(&self) -> &str {
        &self.source[self.span()]
    }

    pub fn slice_at(&self, span: Span) -> &str {
        &self.source[span]
    }

    pub fn tokenstring(&self) -> String {
        self.state.borrow().tokenstream.tokenstring()
    }

    pub fn error<T>(&self, problem: &'static str) -> Result<T, Unwind> {
        self.state.borrow().tokenstream.error(problem)
    }

    pub fn error_at<T>(&self, span: Span, problem: &'static str) -> Result<T, Unwind> {
        self.state.borrow().tokenstream.error_at(span, problem)
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

    Syntax::def(t, HEX_INTEGER, number_prefix, invalid_suffix, precedence_invalid);
    Syntax::def(t, BIN_INTEGER, number_prefix, invalid_suffix, precedence_invalid);
    Syntax::def(t, DEC_INTEGER, number_prefix, invalid_suffix, precedence_invalid);
    Syntax::def(t, SINGLE_FLOAT, number_prefix, invalid_suffix, precedence_invalid);
    Syntax::def(t, DOUBLE_FLOAT, number_prefix, invalid_suffix, precedence_invalid);
    Syntax::def(t, WORD, identifier_prefix, identifier_suffix, identifier_precedence);
    Syntax::def(t, SIGIL, operator_prefix, operator_suffix, operator_precedence);
    Syntax::def(t, KEYWORD, invalid_prefix, keyword_suffix, precedence_5);
    Syntax::def(t, NEWLINE, newline_prefix, newline_suffix, precedence_1);
    Syntax::def(t, EOF, invalid_prefix, invalid_suffix, precedence_0);

    table
}

// KLUDGE: couple of places which don't have convenient access to the table
// need this.
const SEQ_PRECEDENCE: usize = 1;

fn make_name_table() -> NameTable {
    let mut table: NameTable = HashMap::new();
    let t = &mut table;

    Syntax::def(t, "class", class_prefix, invalid_suffix, precedence_0);
    Syntax::def(t, "method", invalid_prefix, invalid_suffix, precedence_0);
    Syntax::def(t, "defaultConstructor", invalid_prefix, invalid_suffix, precedence_0);
    Syntax::def(t, "end", invalid_prefix, invalid_suffix, precedence_0);
    Syntax::def(t, "let", let_prefix, invalid_suffix, precedence_0);
    Syntax::def(t, "return", return_prefix, invalid_suffix, precedence_0);
    Syntax::def(t, ",", invalid_prefix, sequence_suffix, precedence_1);
    Syntax::def(t, "=", invalid_prefix, assign_suffix, precedence_2);

    Syntax::def(t, "{", block_prefix, invalid_suffix, precedence_0);
    Syntax::def(t, "}", invalid_prefix, invalid_suffix, precedence_0);

    Syntax::op(t, "*", false, true, 50);
    Syntax::op(t, "/", false, true, 40);
    Syntax::op(t, "+", false, true, 30);
    Syntax::op(t, "-", true, true, 30);

    table
}

fn precedence_invalid(_: &Parser, _: Span) -> Result<usize, Unwind> {
    // To guarantee it aways gets parsed.
    Ok(1001)
}

fn precedence_5(_: &Parser, _: Span) -> Result<usize, Unwind> {
    Ok(2)
}

fn precedence_2(_: &Parser, _: Span) -> Result<usize, Unwind> {
    Ok(2)
}

fn precedence_1(_: &Parser, _: Span) -> Result<usize, Unwind> {
    Ok(1)
}

fn precedence_0(_: &Parser, _: Span) -> Result<usize, Unwind> {
    Ok(0)
}

fn invalid_prefix(parser: &Parser) -> Result<Expr, Unwind> {
    parser.error("Not valid in value position")
}

fn invalid_suffix(parser: &Parser, _: Expr, _: PrecedenceFunction) -> Result<Expr, Unwind> {
    parser.error("Not valid in operator position")
}

fn identifier_precedence(parser: &Parser, span: Span) -> Result<usize, Unwind> {
    match parser.name_table.get(parser.slice_at(span.clone())) {
        Some(syntax) => parser.syntax_precedence(syntax, span),
        None => return Ok(1000), // unary messages
    }
}

fn identifier_prefix(parser: &Parser) -> Result<Expr, Unwind> {
    let name = parser.slice();
    match parser.name_table.get(name) {
        Some(syntax) => parser.parse_prefix_syntax(syntax),
        None => {
            let c = name.chars().next().expect("BUG: empty identifier");
            if c.is_uppercase() {
                // FIXME: not all languages have uppercase
                return Ok(Global::expr(parser.span(), parser.tokenstring()));
            }
            return Ok(Var::expr(parser.span(), parser.tokenstring()));
        }
    }
}

fn identifier_suffix(parser: &Parser, left: Expr, _: PrecedenceFunction) -> Result<Expr, Unwind> {
    let name = parser.slice();
    match parser.name_table.get(name) {
        Some(syntax) => parser.parse_suffix_syntax(syntax, left),
        None => {
            let c = name.chars().next().expect("BUG: empty identifier");
            if c.is_uppercase() {
                // FIXME: not all languages have uppercase
                return parser.error("Invalid message name (must be lowercase)");
            }
            // Unary message
            Ok(Expr::Send(parser.span(), parser.tokenstring(), Box::new(left), vec![]))
        }
    }
}

fn operator_precedence(parser: &Parser, span: Span) -> Result<usize, Unwind> {
    let slice = parser.slice_at(span.clone());
    match parser.name_table.get(slice) {
        Some(syntax) => parser.syntax_precedence(syntax, span),
        None => parser.error_at(span, "Unknown operator"),
    }
}

fn operator_prefix(parser: &Parser) -> Result<Expr, Unwind> {
    match parser.name_table.get(parser.slice()) {
        Some(syntax) => parser.parse_prefix_syntax(syntax),
        None => parser.error("Unknown operator"),
    }
}

fn operator_suffix(parser: &Parser, left: Expr, _: PrecedenceFunction) -> Result<Expr, Unwind> {
    match parser.name_table.get(parser.slice()) {
        Some(syntax) => parser.parse_suffix_syntax(syntax, left),
        None => parser.error("Unknown operator"),
    }
}

fn assign_suffix(
    parser: &Parser,
    left: Expr,
    precedence: PrecedenceFunction,
) -> Result<Expr, Unwind> {
    if !left.is_var() {
        return parser.error_at(left.span(), "Cannot assign to this");
    }
    let right = parser.parse_expr(precedence(parser, parser.span())?)?;
    // We use the name we're assigning to as the span.
    // FIXME: Maybe this is a sign that we should actually store a Var with it's own span
    // in the Assign, then assign could have the span for just the operator?
    Ok(Assign::expr(left.span(), left.name(), right))
}

fn keyword_suffix(
    parser: &Parser,
    left: Expr,
    precedence: PrecedenceFunction,
) -> Result<Expr, Unwind> {
    let precedence = precedence(parser, parser.span())?;
    let mut selector = parser.tokenstring();
    let mut args = vec![];
    let start = parser.span();
    loop {
        args.push(parser.parse_expr(precedence)?);
        let (token, _span) = parser.lookahead()?;
        if token == Token::KEYWORD {
            parser.next_token()?;
            selector.push_str(parser.slice());
        } else {
            break;
        }
    }
    // FIXME: Potential multiline span is probably going to cause
    // trouble in error reporting...
    Ok(Expr::Send(start.start..parser.span().end, selector, Box::new(left), args))
}

fn parse_method(parser: &Parser, class: &mut ClassDefinition) -> Result<(), Unwind> {
    assert_eq!(parser.slice(), "method");
    // FIXME: span is the span of the method-marker
    let span = parser.span();
    let mut selector = String::new();
    let mut parameters = Vec::new();
    loop {
        match parser.next_token()? {
            Token::WORD | Token::SIGIL => {
                assert!(selector.is_empty());
                assert!(parameters.is_empty());
                selector = parser.tokenstring();
                break;
            }
            Token::KEYWORD => {
                selector.push_str(parser.slice());
                if let Token::WORD = parser.next_token()? {
                    parameters.push(parser.tokenstring());
                } else {
                    return parser.error("Expected keyword selector parameter");
                }
            }
            _ => return parser.error("Expected method selector"),
        }
        match parser.lookahead()? {
            (Token::KEYWORD, _) => continue,
            _ => break,
        }
    }
    // FIXME: This is the place where I could inform parser about instance
    // variables.
    let body = parser.parse_expr(0)?;
    class.add_method(MethodDefinition::new(span, selector, parameters, body));
    Ok(())
}

fn sequence_suffix(
    parser: &Parser,
    left: Expr,
    precedence: PrecedenceFunction,
) -> Result<Expr, Unwind> {
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

fn block_prefix(parser: &Parser) -> Result<Expr, Unwind> {
    let start = parser.span();
    let (token, span) = parser.lookahead()?;
    let mut params = vec![];
    if token == Token::SIGIL && parser.slice_at(span) == "|" {
        parser.next_token()?;
        loop {
            let token = parser.next_token()?;
            if token == Token::WORD {
                params.push(parser.tokenstring());
                continue;
            }
            if token == Token::SIGIL && parser.slice() == "|" {
                break;
            }
            return parser.error("Not valid as block parameter");
        }
    }
    let body = parser.parse_expr(0)?;
    let end = parser.next_token()?;
    // FIXME: hardcoded {
    // Would be nice to be able to swap between [] and {} and
    // keep this function same,
    if end == Token::SIGIL && parser.slice() == "}" {
        Ok(Expr::Block(start.start..parser.span().end, params, Box::new(body)))
    } else {
        parser.error("Expected } as block terminator")
    }
}

fn class_prefix(parser: &Parser) -> Result<Expr, Unwind> {
    // FIXME: span is the span of the class, but maybe it would be better if these
    // had all their own spans.
    let span = parser.span();
    let class_name = match parser.next_token()? {
        Token::WORD => {
            if parser.slice().chars().next().expect("BUG: empty identifier").is_uppercase() {
                parser.tokenstring()
            } else {
                // FIXME: Not all languages use capital letters
                return parser.error("Class names must start with an uppercase letter");
            }
        }
        _ => return parser.error("Expected class name"),
    };
    match parser.next_token()? {
        Token::SIGIL if parser.slice() == "{" => {}
        _ => return parser.error("Expected { to open instance variable block"),
    }
    let mut instance_variables = Vec::new();
    loop {
        let token = parser.next_token()?;
        let tokenstring = parser.tokenstring();
        match token {
            Token::WORD => {
                instance_variables.push(tokenstring);
            }
            Token::SIGIL if parser.slice() == "}" => {
                break;
            }
            Token::SIGIL if parser.slice() == "," => {
                continue;
            }
            _ => return parser.error("Invalid instance variable specification"),
        }
    }
    let size = instance_variables.len();
    let mut class = ClassDefinition::new(span, class_name, instance_variables);
    loop {
        let next = parser.next_token()?;
        if next == Token::NEWLINE {
            continue;
        }
        if next == Token::WORD && parser.slice() == "end" {
            break;
        }
        if next == Token::WORD && parser.slice() == "method" {
            parse_method(parser, &mut class)?;
            continue;
        }
        if next == Token::WORD && parser.slice() == "defaultConstructor" {
            let ctor = parser.next_token()?;
            if ctor == Token::WORD {
                if size > 0 {
                    return parser
                        .error("Class has instance variables: no default constructor available");
                }
                if class.default_constructor.is_some() {
                    return parser.error("Multiple default constructors specified");
                }
                class.default_constructor = Some(parser.tokenstring());
            }
            continue;
        }
        return parser.error("Expected method or end");
    }
    Ok(Expr::ClassDefinition(class))
}

fn let_prefix(parser: &Parser) -> Result<Expr, Unwind> {
    if Token::WORD != parser.next_token()? {
        return parser.error("Expected variable name after let");
    }
    let name = parser.slice().to_string();
    let next = parser.next_token()?;
    if Token::SIGIL != next || "=" != parser.slice() {
        return parser.error("Expected = in let");
    }
    let value = parser.parse_expr(SEQ_PRECEDENCE)?;
    let next = parser.next_token()?;
    if Token::SIGIL != next {
        return parser.error("Expected separator after let");
    }
    let body = parser.parse_expr(0)?;
    Ok(Expr::Bind(name, Box::new(value), Box::new(body)))
}

fn newline_prefix(parser: &Parser) -> Result<Expr, Unwind> {
    parser.parse_expr(0)
}

fn newline_suffix(_: &Parser, left: Expr, _: PrecedenceFunction) -> Result<Expr, Unwind> {
    Ok(left)
}

fn number_prefix(parser: &Parser) -> Result<Expr, Unwind> {
    let slice = parser.slice();
    // Hexadecimal case
    if slice.len() > 2 && ("0x" == &slice[0..2] || "0X" == &slice[0..2]) {
        let integer = match i64::from_str_radix(&slice[2..], 16) {
            Ok(i) => i,
            Err(_) => return parser.error("Malformed hexadecimal number"),
        };
        Ok(Expr::Const(parser.span(), Literal::Integer(integer)))
    }
    // Binary case
    else if slice.len() > 2 && ("0b" == &slice[0..2] || "0B" == &slice[0..2]) {
        let integer = match i64::from_str_radix(&slice[2..], 2) {
            Ok(i) => i,
            Err(_) => return parser.error("Malformed binary number"),
        };
        Ok(Expr::Const(parser.span(), Literal::Integer(integer)))
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
                        Ok(f) => return Ok(Expr::Const(parser.span(), Literal::Float(f))),
                        Err(_) => return parser.error("Malformed number"),
                    }
                }
            }
        }
        Ok(Expr::Const(parser.span(), Literal::Integer(decimal)))
    }
}

fn return_prefix(parser: &Parser) -> Result<Expr, Unwind> {
    Ok(Return::expr(parser.span(), parser.parse_expr(SEQ_PRECEDENCE)?))
}

/// Tests and tools

pub fn parse_str(source: &str) -> Result<Expr, Unwind> {
    Parser::new(source).parse().map_err(|e| e.add_context(source))
}

fn int(span: Span, value: i64) -> Expr {
    Expr::Const(span, Literal::Integer(value))
}

fn float(span: Span, value: f64) -> Expr {
    Expr::Const(span, Literal::Float(value))
}

fn var(span: Span, name: &str) -> Expr {
    Var::expr(span, name.to_string())
}

fn unary(span: Span, name: &str, left: Expr) -> Expr {
    Expr::Send(span, name.to_string(), Box::new(left), vec![])
}

fn binary(span: Span, name: &str, left: Expr, right: Expr) -> Expr {
    Expr::Send(span, name.to_string(), Box::new(left), vec![right])
}

fn keyword(span: Span, name: &str, left: Expr, args: Vec<Expr>) -> Expr {
    Expr::Send(span, name.to_string(), Box::new(left), args)
}

fn bind(name: &str, value: Expr, body: Expr) -> Expr {
    Expr::Bind(name.to_string(), Box::new(value), Box::new(body))
}

fn block(span: Span, params: Vec<&str>, body: Expr) -> Expr {
    Expr::Block(span, params.into_iter().map(String::from).collect(), Box::new(body))
}

fn seq(exprs: Vec<Expr>) -> Expr {
    Expr::Seq(exprs)
}

fn class(span: Span, name: &str, instance_variables: Vec<&str>) -> Expr {
    ClassDefinition::expr(
        span,
        name.to_string(),
        instance_variables.into_iter().map(|n| n.to_string()).collect(),
    )
}

fn method(span: Span, selector: &str, parameters: Vec<&str>, body: Expr) -> MethodDefinition {
    MethodDefinition::new(
        span,
        selector.to_string(),
        parameters.iter().map(|n| n.to_string()).collect(),
        body,
    )
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

#[test]
fn parse_keyword() {
    assert_eq!(
        parse_str("foo x: 1 y: 2, bar"),
        Ok(seq(vec![
            keyword(4..13, "x:y:", var(0..3, "foo"), vec![int(7..8, 1), int(12..13, 2)]),
            var(15..18, "bar")
        ]))
    );
}

#[test]
fn parse_block_no_args() {
    assert_eq!(
        parse_str(" { foo bar } "),
        Ok(block(1..12, vec![], unary(7..10, "bar", var(3..6, "foo"))))
    );
}

#[test]
fn parse_block_args() {
    assert_eq!(
        parse_str(" { |x| foo bar: x } "),
        Ok(block(
            1..19,
            vec!["x"],
            keyword(11..17, "bar:", var(7..10, "foo"), vec![var(16..17, "x")])
        ))
    );
}

#[test]
fn parse_class() {
    assert_eq!(parse_str("class Point { x, y } end"), Ok(class(0..5, "Point", vec!["x", "y"])));
}

#[test]
fn parse_method1() {
    let mut class = class(0..5, "Foo", vec![]);
    class.add_method(method(13..19, "bar", vec![], int(24..26, 42)));
    assert_eq!(parse_str("class Foo {} method bar 42 end"), Ok(class));
}

#[test]
fn parse_method2() {
    let mut class = class(18..23, "Foo", vec![]);
    class.add_method(method(52..58, "foo", vec![], unary(92..95, "bar", var(87..91, "self"))));
    class.add_method(method(117..123, "bar", vec![], int(152..154, 42)));
    assert_eq!(
        parse_str(
            "
                 class Foo {}
                     method foo
                        self bar
                     method bar
                        42
                 end"
        ),
        Ok(class)
    );
}

#[test]
fn parse_return1() {
    assert_eq!(parse_str("return 12"), Ok(Return::expr(0..6, int(7..9, 12))));
}
