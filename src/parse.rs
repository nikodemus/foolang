use std::borrow::ToOwned;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::convert::Into;
use std::str::FromStr;

use crate::tokenstream::{Span, Token, TokenStream};
use crate::unwind::Unwind;

trait ShiftSpan {
    fn shift(&mut self, n: usize);
}

impl ShiftSpan for Span {
    fn shift(&mut self, n: usize) {
        self.start += n;
        self.end += n;
    }
}

// FIXME: Do these really need clone, if so, why?

#[derive(Debug, PartialEq, Clone)]
pub struct Array {
    pub span: Span,
    pub data: Vec<Expr>,
}

impl Array {
    pub fn expr(span: Span, data: Vec<Expr>) -> Expr {
        Expr::Array(Array {
            span,
            data,
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Assign {
    pub span: Span,
    pub name: String,
    pub value: Box<Expr>,
}

impl Assign {
    pub fn expr(span: Span, name: String, value: Expr) -> Expr {
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
    pub instance_variables: Vec<Var>,
    pub instance_methods: Vec<MethodDefinition>,
    pub class_methods: Vec<MethodDefinition>,
    default_constructor: Option<String>,
}

impl ClassDefinition {
    fn new(span: Span, name: String, instance_variables: Vec<Var>) -> ClassDefinition {
        ClassDefinition {
            span,
            name,
            instance_variables,
            instance_methods: Vec::new(),
            class_methods: Vec::new(),
            default_constructor: None,
        }
    }

    #[cfg(test)]
    pub fn expr(span: Span, name: String, instance_variables: Vec<Var>) -> Expr {
        Expr::ClassDefinition(ClassDefinition::new(span, name, instance_variables))
    }

    fn add_method(&mut self, kind: MethodKind, method: MethodDefinition) {
        match kind {
            MethodKind::Instance => self.instance_methods.push(method),
            MethodKind::Class => self.class_methods.push(method),
        };
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
                selector.push_str(&var.name);
                selector.push_str(":");
            }
            selector
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Import {
    pub span: Span,
    pub path: String,
    pub prefix: String,
    pub name: Option<String>,
    pub body: Option<Box<Expr>>,
}

impl Import {
    pub fn expr(
        span: Span,
        path: &str,
        prefix: &str,
        name: Option<&str>,
        body: Option<Expr>,
    ) -> Expr {
        Expr::Import(Import {
            span,
            path: path.to_string(),
            prefix: prefix.to_string(),
            name: name.map(|x| x.to_string()),
            body: body.map(|x| Box::new(x)),
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Message {
    pub span: Span,
    pub selector: String,
    pub args: Vec<Expr>,
}

impl Message {
    fn shift_span(&mut self, n: usize) {
        self.span.shift(n);
        for arg in &mut self.args {
            arg.shift_span(n);
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct MethodDefinition {
    pub span: Span,
    pub selector: String,
    pub parameters: Vec<Var>,
    pub body: Box<Expr>,
    pub return_type: Option<String>,
}

impl MethodDefinition {
    fn new(
        span: Span,
        selector: String,
        parameters: Vec<Var>,
        body: Expr,
        return_type: Option<String>,
    ) -> MethodDefinition {
        MethodDefinition {
            span,
            selector,
            parameters,
            body: Box::new(body),
            return_type,
        }
    }
    fn shift_span(&mut self, n: usize) {
        self.span.shift(n);
        for var in &mut self.parameters {
            var.span.shift(n);
        }
        self.body.shift_span(n);
    }
}

pub enum MethodKind {
    Class,
    Instance,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Return {
    pub span: Span,
    pub value: Box<Expr>,
}

impl Return {
    pub fn expr(span: Span, value: Expr) -> Expr {
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
    pub typename: Option<String>,
}

impl Var {
    fn untyped(span: Span, name: String) -> Var {
        Var {
            span,
            name,
            typename: None,
        }
    }
    fn typed(span: Span, name: String, typename: String) -> Var {
        Var {
            span,
            name,
            typename: Some(typename),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Global {
    pub span: Span,
    pub name: String,
}

impl Global {
    pub fn expr(span: Span, name: &str) -> Expr {
        Expr::Global(Global {
            span,
            name: name.to_string(),
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Array(Array),
    Assign(Assign),
    Bind(String, Option<String>, Box<Expr>, Option<Box<Expr>>),
    Block(Span, Vec<Var>, Box<Expr>, Option<String>),
    Cascade(Box<Expr>, Vec<Vec<Message>>),
    Chain(Box<Expr>, Vec<Message>),
    ClassDefinition(ClassDefinition),
    Const(Span, Literal),
    Eq(Span, Box<Expr>, Box<Expr>),
    Global(Global),
    Import(Import),
    Return(Return),
    Seq(Vec<Expr>),
    Typecheck(Span, Box<Expr>, String),
    Var(Var),
}

impl Expr {
    fn is_var(&self) -> bool {
        match self {
            Expr::Var(..) => true,
            _ => false,
        }
    }

    fn is_class_definition(&self) -> bool {
        match self {
            Expr::ClassDefinition(..) => true,
            _ => false,
        }
    }

    #[cfg(test)]
    pub fn add_method(&mut self, kind: MethodKind, method: MethodDefinition) {
        match self {
            Expr::ClassDefinition(class) => class.add_method(kind, method),
            _ => panic!("BUG: trying to add a method to {:?}", self),
        }
    }

    fn name(&self) -> String {
        match self {
            Expr::Var(var) => var.name.to_owned(),
            _ => panic!("BUG: cannot extract name from {:?}", self),
        }
    }

    fn to_cascade(self, in_cascade: bool) -> Expr {
        // If we're already in cascade then self is a Chain whose
        // receiver is a cascade and we splice the messages into the
        // cascade, which becomes our receiver.
        //
        // Otherwise left becomes the initial receiver of an initially
        // empty cascade.
        match self {
            Expr::Cascade(..) => self,
            Expr::Chain(receiver, messages) => {
                if let Expr::Cascade(cascade_receiver, mut chains) = *receiver {
                    chains.push(messages);
                    Expr::Cascade(cascade_receiver, chains)
                } else {
                    assert!(in_cascade);
                    Expr::Cascade(Box::new(Expr::Chain(receiver, messages)), vec![])
                }
            }
            _ => {
                assert!(in_cascade);
                Expr::Cascade(Box::new(self), vec![])
            }
        }
    }

    pub fn send(self, message: Message) -> Expr {
        let (receiver, messages) = match self {
            Expr::Chain(receiver, mut messages) => {
                messages.push(message);
                (receiver, messages)
            }
            _ => (Box::new(self), vec![message]),
        };
        Expr::Chain(receiver, messages)
    }

    pub fn span(&self) -> Span {
        use Expr::*;
        let span = match self {
            Array(array) => &array.span,
            Assign(assign) => &assign.span,
            Bind(_, _, expr, ..) => return expr.span(),
            Block(span, ..) => span,
            Cascade(left, ..) => return left.span(),
            ClassDefinition(definition) => &definition.span,
            Const(span, ..) => span,
            Eq(span, ..) => span,
            Global(global) => &global.span,
            Chain(left, _) => return left.span(),
            Import(import) => &import.span,
            Return(ret) => &ret.span,
            // FIXME: Questionable
            Seq(exprs) => return exprs[exprs.len() - 1].span(),
            Typecheck(span, ..) => span,
            Var(var) => &var.span,
        };
        span.to_owned()
    }

    pub fn shift_span(&mut self, n: usize) {
        use Expr::*;
        match self {
            Array(array) => {
                array.span.shift(n);
                for elt in &mut array.data {
                    elt.shift_span(n);
                }
            }
            Assign(assign) => {
                assign.span.shift(n);
                assign.value.shift_span(n);
            }
            Bind(_name, _type, value, body) => {
                value.shift_span(n);
                if let Some(expr) = body {
                    expr.shift_span(n);
                }
            }
            Block(span, vars, body, _rtype) => {
                span.shift(n);
                for var in vars {
                    var.span.shift(n);
                }
                body.shift_span(n);
            }
            Cascade(receiver, chains) => {
                receiver.shift_span(n);
                for chain in chains {
                    for message in chain {
                        message.shift_span(n);
                    }
                }
            }
            ClassDefinition(class) => {
                class.span.shift(n);
                for var in &mut class.instance_variables {
                    var.span.shift(n);
                }
                for m in &mut class.instance_methods {
                    m.shift_span(n);
                }
                for m in &mut class.class_methods {
                    m.shift_span(n);
                }
            }
            Const(span, _literal) => {
                span.shift(n);
            }
            Eq(span, left, right) => {
                span.shift(n);
                left.shift_span(n);
                right.shift_span(n);
            }
            Global(global) => {
                global.span.shift(n);
            }
            Chain(receiver, chain) => {
                receiver.shift_span(n);
                for message in chain {
                    message.shift_span(n);
                }
            }
            Import(import) => {
                import.span.shift(n);
                if let Some(ref mut body) = import.body {
                    body.shift_span(n);
                }
            }
            Return(ret) => {
                ret.span.shift(n);
                ret.value.shift_span(n);
            }
            Seq(exprs) => {
                for expr in exprs {
                    expr.shift_span(n);
                }
            }
            Typecheck(span, expr, _type) => {
                span.shift(n);
                expr.shift_span(n);
            }
            Var(var) => {
                var.span.shift(n);
            }
        };
    }
}

type PrefixParser = fn(&Parser) -> Result<Expr, Unwind>;
type SuffixParser = fn(&Parser, Expr, PrecedenceFunction) -> Result<Expr, Unwind>;
// FIXME: can I remove the span from here?
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
    lookahead: VecDeque<(Token, Span)>,
    span: Span,
}

impl<'a> ParserState<'a> {
    fn scan(&mut self) -> Result<(Token, Span), Unwind> {
        let token = self.tokenstream.scan()?;
        Ok((token, self.tokenstream.span()))
    }

    fn next_token(&mut self) -> Result<Token, Unwind> {
        let (token, span) = if self.lookahead.is_empty() {
            self.scan()?
        } else {
            self.lookahead.pop_front().unwrap()
        };
        self.span = span;
        Ok(token)
    }

    fn lookahead(&mut self) -> Result<(Token, Span), Unwind> {
        if self.lookahead.is_empty() {
            let look = self.scan()?;
            self.lookahead.push_back(look);
        }
        Ok(self.lookahead.front().unwrap().clone())
    }

    fn lookahead2(&mut self) -> Result<((Token, Span), (Token, Span)), Unwind> {
        while self.lookahead.len() < 2 {
            let look = self.scan()?;
            self.lookahead.push_back(look);
        }
        Ok((self.lookahead.get(0).unwrap().clone(), self.lookahead.get(1).unwrap().clone()))
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
                lookahead: VecDeque::new(),
                span: 0..0,
            }),
        }
    }

    pub fn parse(&mut self) -> Result<Expr, Unwind> {
        self.parse_expr(0)
    }

    pub fn parse_seq(&self) -> Result<Expr, Unwind> {
        let seq = self.parse_expr(1)?;
        // FIXME: Terrible KLUDGE.
        //
        // 1. Class does not consume it's end since we use it to sequence toplevel definitions.
        // 2. If class appears in a sequence context it will leave the end behind to be
        //    discovered in a _prefix_ context.
        //
        // So we clean it up.
        //
        let (token, span) = self.lookahead()?;
        if token == Token::WORD && self.slice_at(span) == "end" {
            let is_class_def = if let Expr::Seq(seq) = &seq {
                seq[seq.len() - 1].is_class_definition()
            } else {
                seq.is_class_definition()
            };
            if is_class_def {
                self.next_token()?;
            }
        }
        Ok(seq)
    }

    pub fn parse_body(&self) -> Result<Expr, Unwind> {
        self.parse_expr(1)
    }

    pub fn parse_expr(&self, precedence: usize) -> Result<Expr, Unwind> {
        self.parse_tail(self.parse_prefix()?, precedence)
    }

    pub fn parse_tail(&self, mut expr: Expr, precedence: usize) -> Result<Expr, Unwind> {
        while precedence < self.next_precedence()? {
            expr = self.parse_suffix(expr)?;
        }
        Ok(expr)
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

    fn next_precedence(&self) -> Result<usize, Unwind> {
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
                let span = self.span();
                Ok(self.parse_expr(PREFIX_PRECEDENCE)?.send(Message {
                    span,
                    selector: format!("prefix{}", operator),
                    args: vec![],
                }))
            }
            _ => self.error("Expected value or prefix operator"),
        }
    }

    fn parse_suffix_syntax(&self, syntax: &Syntax, left: Expr) -> Result<Expr, Unwind> {
        match syntax {
            Syntax::General(_, suffix, precedence) => suffix(self, left, *precedence),
            Syntax::Operator(_, is_binary, precedence) if *is_binary => {
                let operator = self.tokenstring();
                Ok(left.send(Message {
                    span: self.span(),
                    selector: operator,
                    args: vec![self.parse_expr(*precedence)?],
                }))
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

    pub fn lookahead2(&self) -> Result<((Token, Span), (Token, Span)), Unwind> {
        self.state.borrow_mut().lookahead2()
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

    pub fn eof_error<T>(&self, problem: &str) -> Result<T, Unwind> {
        self.state.borrow().tokenstream.eof_error(problem)
    }

    pub fn error<T>(&self, problem: &str) -> Result<T, Unwind> {
        self.state.borrow().tokenstream.error(problem)
    }

    pub fn error_at<T>(&self, span: Span, problem: &str) -> Result<T, Unwind> {
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

    // Literals should appear in prefix-positions only, hence precedence_invald
    Syntax::def(t, HEX_INTEGER, number_prefix, invalid_suffix, precedence_invalid);
    Syntax::def(t, BIN_INTEGER, number_prefix, invalid_suffix, precedence_invalid);
    Syntax::def(t, DEC_INTEGER, number_prefix, invalid_suffix, precedence_invalid);
    Syntax::def(t, SINGLE_FLOAT, number_prefix, invalid_suffix, precedence_invalid);
    Syntax::def(t, DOUBLE_FLOAT, number_prefix, invalid_suffix, precedence_invalid);
    Syntax::def(t, STRING, string_prefix, invalid_suffix, precedence_invalid);
    // Comments
    Syntax::def(t, COMMENT, ignore_prefix, ignore_suffix, precedence_1000);
    Syntax::def(t, BLOCK_COMMENT, ignore_prefix, ignore_suffix, precedence_1000);
    // Others
    Syntax::def(t, WORD, identifier_prefix, identifier_suffix, identifier_precedence);
    Syntax::def(t, SIGIL, operator_prefix, operator_suffix, operator_precedence);
    Syntax::def(t, KEYWORD, invalid_prefix, keyword_suffix, precedence_9);
    Syntax::def(t, EOF, eof_prefix, eof_suffix, precedence_0);

    table
}

// KLUDGE: couple of places which don't have convenient access to the table
// need this.
const SEQ_PRECEDENCE: usize = 2;
const PREFIX_PRECEDENCE: usize = 1000;

fn make_name_table() -> NameTable {
    let mut table: NameTable = HashMap::new();
    let t = &mut table;

    Syntax::def(t, "class", class_prefix, invalid_suffix, precedence_0);
    Syntax::def(t, "import", import_prefix, invalid_suffix, precedence_0);
    Syntax::def(t, ",", invalid_prefix, invalid_suffix, precedence_0);
    Syntax::def(t, "defaultConstructor", invalid_prefix, invalid_suffix, precedence_0);
    Syntax::def(t, "method", invalid_prefix, invalid_suffix, precedence_0);
    Syntax::def(t, "end", invalid_prefix, end_suffix, precedence_1);
    Syntax::def(t, ".", invalid_prefix, sequence_suffix, precedence_2);
    Syntax::def(t, "let", let_prefix, invalid_suffix, precedence_3);
    Syntax::def(t, "return", return_prefix, invalid_suffix, precedence_3);
    Syntax::def(t, ";", invalid_prefix, cascade_suffix, precedence_3);
    Syntax::def(t, "=", invalid_prefix, assign_suffix, precedence_4);
    Syntax::def(t, "is", invalid_prefix, is_suffix, precedence_10);
    Syntax::def(t, "::", invalid_prefix, typecheck_suffix, precedence_1000);

    // FIXME: Should opening group sigils use prefix precedence?
    Syntax::def(t, "[", array_prefix, invalid_suffix, precedence_3);
    Syntax::def(t, "]", invalid_prefix, invalid_suffix, precedence_0);
    Syntax::def(t, "(", paren_prefix, invalid_suffix, precedence_3);
    Syntax::def(t, ")", invalid_prefix, invalid_suffix, precedence_0);
    Syntax::def(t, "{", block_prefix, invalid_suffix, precedence_3);
    Syntax::def(t, "}", invalid_prefix, invalid_suffix, precedence_0);

    Syntax::def(t, "False", false_prefix, invalid_suffix, precedence_3);
    Syntax::def(t, "True", true_prefix, invalid_suffix, precedence_3);

    Syntax::op(t, "*", false, true, 50);
    Syntax::op(t, "/", false, true, 40);
    Syntax::op(t, "+", false, true, 30);
    Syntax::op(t, "-", true, true, 30);

    Syntax::op(t, "<", false, true, 10);
    Syntax::op(t, "<=", false, true, 10);
    Syntax::op(t, ">", false, true, 10);
    Syntax::op(t, ">=", false, true, 10);
    Syntax::op(t, "==", false, true, 10);

    table
}

fn precedence_invalid(_: &Parser, _: Span) -> Result<usize, Unwind> {
    // To guarantee it aways gets parsed.
    Ok(1001)
}

fn precedence_1000(_: &Parser, _: Span) -> Result<usize, Unwind> {
    Ok(1000)
}

fn precedence_10(_: &Parser, _: Span) -> Result<usize, Unwind> {
    Ok(10)
}

fn precedence_9(_: &Parser, _: Span) -> Result<usize, Unwind> {
    Ok(9)
}

fn precedence_4(_: &Parser, _: Span) -> Result<usize, Unwind> {
    Ok(4)
}

fn precedence_3(_: &Parser, _: Span) -> Result<usize, Unwind> {
    Ok(3)
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

fn array_prefix(parser: &Parser) -> Result<Expr, Unwind> {
    let start = parser.span().start;
    let (token, next) = parser.lookahead()?;
    let next_end = next.end;
    let (span, data) = if token == Token::SIGIL && parser.slice_at(next) == "]" {
        parser.next_token()?;
        (start..next_end, vec![])
    } else {
        let mut data = vec![];
        loop {
            data.push(parser.parse_expr(0)?);
            let token = parser.next_token()?;
            if token == Token::SIGIL && parser.slice() == "]" {
                break (start..parser.span().end, data);
            }
            if token == Token::SIGIL && parser.slice() == "," {
                continue;
            }
            return parser.error("Expected ] or ,");
        }
    };
    Ok(Array::expr(span, data))
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

fn cascade_suffix(
    parser: &Parser,
    left: Expr,
    precedence: PrecedenceFunction,
) -> Result<Expr, Unwind> {
    let receiver = left.to_cascade(true);
    Ok(parser.parse_tail(receiver, precedence(parser, parser.span())?)?.to_cascade(false))
}

fn eof_prefix(parser: &Parser) -> Result<Expr, Unwind> {
    parser.eof_error("Unexpected EOF in value position")
}

fn eof_suffix(parser: &Parser, _: Expr, _: PrecedenceFunction) -> Result<Expr, Unwind> {
    parser.eof_error("Unexpected EOF in suffix position")
}

fn false_prefix(parser: &Parser) -> Result<Expr, Unwind> {
    Ok(Expr::Const(parser.span(), Literal::Boolean(false)))
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
                return Ok(Global::expr(parser.span(), parser.slice()));
            }
            return Ok(Expr::Var(Var::untyped(parser.span(), parser.tokenstring())));
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
            Ok(left.send(Message {
                span: parser.span(),
                selector: parser.tokenstring(),
                args: vec![],
            }))
        }
    }
}

fn is_suffix(parser: &Parser, left: Expr, pre: PrecedenceFunction) -> Result<Expr, Unwind> {
    let span = parser.span();
    let right = parser.parse_expr(pre(parser, span.clone())?)?;
    Ok(Expr::Eq(span, Box::new(left), Box::new(right)))
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
        // Two-element lookahead.
        let (token, _) = parser.lookahead()?;
        if token == Token::KEYWORD {
            parser.next_token()?;
            selector.push_str(parser.slice());
        } else {
            break;
        }
    }
    // FIXME: Potential multiline span is probably going to cause
    // trouble in error reporting...
    Ok(left.send(Message {
        span: start.start..parser.span().end,
        selector,
        args,
    }))
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

fn paren_prefix(parser: &Parser) -> Result<Expr, Unwind> {
    let expr = parser.parse_seq()?;
    let token = parser.next_token()?;
    if token == Token::SIGIL && parser.slice() == ")" {
        Ok(expr)
    } else {
        // FIXME: EOF
        parser.error("Expected )")
    }
}

fn true_prefix(parser: &Parser) -> Result<Expr, Unwind> {
    Ok(Expr::Const(parser.span(), Literal::Boolean(true)))
}

fn typecheck_suffix(
    parser: &Parser,
    left: Expr,
    _precedence: PrecedenceFunction,
) -> Result<Expr, Unwind> {
    match parser.next_token()? {
        Token::WORD => Ok(Expr::Typecheck(parser.span(), Box::new(left), parser.tokenstring())),
        _ => parser.error("Invalid type designator"),
    }
}

fn sequence_suffix(
    parser: &Parser,
    left: Expr,
    precedence: PrecedenceFunction,
) -> Result<Expr, Unwind> {
    let (token, span) = parser.lookahead()?;
    let text = parser.slice_at(span);
    if (token == Token::WORD && (text == "method" || text == "end" || text == "class"))
        || token == Token::EOF
    {
        return Ok(left);
    }
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

fn end_suffix(parser: &Parser, left: Expr, precedence: PrecedenceFunction) -> Result<Expr, Unwind> {
    let mut exprs = if let Expr::Seq(left_exprs) = left {
        if left_exprs[left_exprs.len() - 1].is_class_definition() {
            left_exprs
        } else {
            return parser.error("Unexpected 'end': not after class definition.");
        }
    } else {
        if left.is_class_definition() {
            vec![left]
        } else {
            return parser.error("Unexpected 'end': not after class definition.");
        }
    };
    let (token, _) = parser.lookahead()?;
    if token == Token::EOF {
        if exprs.len() > 1 {
            return Ok(Expr::Seq(exprs));
        } else {
            return Ok(exprs.pop().unwrap());
        }
    }
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
                params.push(parse_var(parser)?);
                continue;
            }
            if token == Token::SIGIL && parser.slice() == "|" {
                break;
            }
            return parser.error("Not valid as block parameter");
        }
    }
    let (token, span2) = parser.lookahead()?;
    let rtype = if token == Token::SIGIL && parser.slice_at(span2) == "->" {
        parser.next_token()?;
        Some(parse_type_designator(parser)?)
    } else {
        None
    };
    let (token, span3) = parser.lookahead()?;
    let body = if token == Token::SIGIL && parser.slice_at(span3) == "}" {
        // FIXME: Bogus source location
        Expr::Const(0..0, Literal::Boolean(false))
    } else {
        parser.parse_seq()?
    };
    let end = parser.next_token()?;
    // FIXME: hardcoded {
    // Would be nice to be able to swap between [] and {} and
    // keep this function same,
    if end == Token::SIGIL && parser.slice() == "}" {
        Ok(Expr::Block(start.start..parser.span().end, params, Box::new(body), rtype))
    } else if end == Token::EOF {
        parser.eof_error("Unexpected EOF while pasing a block: expected } as block terminator")
    } else {
        parser.error("Expected } as block terminator")
    }
}

fn dotted_name_at(parser: &Parser, point: usize) -> Result<Option<Span>, Unwind> {
    let ((token1, span1), (token2, span2)) = parser.lookahead2()?;
    if !(token1 == Token::SIGIL && parser.slice_at(span1.clone()) == "." && span1.start == point) {
        return Ok(None);
    }
    if !((token2 == Token::WORD || token2 == Token::SIGIL) && span2.start == span1.end) {
        return Ok(None);
    }
    // Dot followed by a word or sigil, no whitespace -- ok!
    parser.next_token()?;
    parser.next_token()?;
    Ok(Some(point..span2.end))
}

fn import_prefix(parser: &Parser) -> Result<Expr, Unwind> {
    let start = parser.span().start;
    if Token::WORD == parser.next_token()? {
        let mut span = start..parser.span().end;
        let mut spec = parser.slice().to_string();
        loop {
            if let Some(more) = dotted_name_at(parser, span.end)? {
                spec.push_str(parser.slice_at(more.clone()));
                span = more;
            } else {
                break;
            }
        }
        let body = if Token::EOF == parser.lookahead()?.0 {
            parser.next_token()?;
            None
        } else {
            Some(Box::new(parser.parse_seq()?))
        };
        let mut path = String::new();
        let mut prefix = String::new();
        let mut name = None;
        let mut parts = spec.split(".").peekable();
        while let Some(part) = parts.next() {
            let uppercase = part.chars().next().unwrap().is_uppercase();
            if parts.peek().is_some() {
                if uppercase {
                    return Unwind::error_at(
                        start..parser.span().start,
                        "Illegal import: uppercase module name",
                    );
                }
                path.push_str(part);
                path.push_str(".");
            } else {
                if uppercase {
                    assert_eq!(Some('.'), path.pop());
                    name = Some(part.to_string());
                } else {
                    path.push_str(part);
                    prefix.push_str(part);
                };
            }
        }
        Ok(Expr::Import(Import {
            span,
            path,
            prefix,
            name,
            body,
        }))
    } else {
        return parser.error("Expected module name");
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
    loop {
        match parser.next_token()? {
            Token::SIGIL if parser.slice() == "{" => break,
            _ => return parser.error("Expected { to open instance variable block"),
        }
    }
    let mut instance_variables = Vec::new();
    loop {
        let token = parser.next_token()?;
        match token {
            Token::WORD => {
                instance_variables.push(parse_var(parser)?);
            }
            Token::SIGIL if parser.slice() == "}" => break,
            _ => return parser.error("Invalid instance variable specification"),
        }
    }
    let size = instance_variables.len();
    let mut class = ClassDefinition::new(span, class_name, instance_variables);
    loop {
        let (next, span2) = parser.lookahead()?;
        if next == Token::WORD && parser.slice_at(span2) == "end" {
            break;
        }
        parser.next_token()?;
        if next == Token::EOF {
            return parser.eof_error("Unexpected EOF while parsing class: expected method or end");
        }
        if next == Token::WORD && parser.slice() == "class" {
            if parser.next_token()? == Token::WORD && parser.slice() == "method" {
                parse_method(parser, &mut class, MethodKind::Class)?;
                continue;
            } else {
                return parser.error("Expected class method");
            }
        }
        if next == Token::WORD && parser.slice() == "method" {
            parse_method(parser, &mut class, MethodKind::Instance)?;
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

    let Var {
        name,
        typename,
        ..
    } = parse_var(parser)?;

    if !(parser.next_token()? == Token::SIGIL && parser.slice() == "=") {
        return parser.error("Expected = in let");
    }

    let value = parser.parse_expr(SEQ_PRECEDENCE)?;
    let mut eof = match parser.next_token()? {
        Token::SIGIL if parser.slice() == "." => false,
        Token::EOF => true,
        _ => {
            return parser.error("Expected separator after let");
        }
    };
    // For REPL niceness:
    //   > let x = 2
    //   2
    if Token::EOF == parser.lookahead()?.0 {
        parser.next_token()?;
        eof = true;
    }
    let body = if eof {
        None
    } else {
        Some(Box::new(parser.parse_seq()?))
    };
    Ok(Expr::Bind(name, typename, Box::new(value), body))
}

fn ignore_prefix(parser: &Parser) -> Result<Expr, Unwind> {
    parser.parse_expr(SEQ_PRECEDENCE)
}

fn ignore_suffix(_parser: &Parser, left: Expr, _pre: PrecedenceFunction) -> Result<Expr, Unwind> {
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

fn string_prefix(parser: &Parser) -> Result<Expr, Unwind> {
    let slice = parser.slice();

    // Strip quotes
    let mut n = 0;
    while n < slice.len() - n && &slice[n..n + 1] == "\"" {
        n += 1;
    }

    fn interpolate(parser: &Parser, span: Span, n: usize) -> Result<Expr, Unwind> {
        let data = parser.slice_at(span.clone());

        let p0 = match data.find('{') {
            None => {
                // No interpolation.
                return Ok(Expr::Const(
                    span.start - n..span.end + n,
                    Literal::String(data.to_string()),
                ));
            }
            Some(p) => p,
        };
        let p1 = match data[p0..].find('}') {
            None => return parser.error("Unterminated string interpolation."),
            Some(p) => p + p0,
        };

        // FIXME: parse errors from parse_str don't show the larger context, and have
        // incorrect line numbers
        let mut expr = parse_str(&data[p0 + 1..p1])?;
        expr.shift_span(span.start);
        let interp = expr.send(Message {
            span: p0 + 1..p1,
            selector: "toString".to_string(),
            args: vec![],
        });
        let left = if p0 > 0 {
            Expr::Const(span.start..p0, Literal::String(data[0..p0].to_string())).send(Message {
                span: p0 + 1..p1,
                selector: "append:".to_string(),
                args: vec![interp],
            })
        } else {
            interp
        };

        if p1 + 1 < span.end {
            let right = interpolate(parser, span.start + p1 + 1..span.end, n)?;
            Ok(left.send(Message {
                span: p0 + 1..p1,
                selector: "append:".to_string(),
                args: vec![right],
            }))
        } else {
            Ok(left)
        }
    }

    let span = parser.span();
    interpolate(parser, (span.start + n)..(span.end - n), n)
}

/// Utils

fn parse_type_designator(parser: &Parser) -> Result<String, Unwind> {
    if let Token::WORD = parser.next_token()? {
        Ok(parser.tokenstring())
    } else {
        parser.error("Invalid type designator")
    }
}

fn parse_var(parser: &Parser) -> Result<Var, Unwind> {
    let name = parser.tokenstring();
    let namespan = parser.span();
    let (token, span) = parser.lookahead()?;
    if token == Token::SIGIL && parser.slice_at(span) == "::" {
        parser.next_token()?;
        Ok(Var::typed(namespan, name, parse_type_designator(parser)?))
    } else {
        Ok(Var::untyped(namespan, name))
    }
}

fn parse_method(
    parser: &Parser,
    class: &mut ClassDefinition,
    kind: MethodKind,
) -> Result<(), Unwind> {
    assert_eq!(parser.slice(), "method");
    let span = parser.span();
    let mut selector = String::new();
    let mut parameters = Vec::new();
    loop {
        let token = parser.next_token()?;
        selector.push_str(parser.slice());
        match token {
            Token::WORD => {
                assert!(parameters.is_empty());
                break;
            }
            Token::SIGIL => {
                assert!(parameters.is_empty());
                if let Token::WORD = parser.next_token()? {
                    parameters.push(parse_var(parser)?);
                } else {
                    return parser.error("Expected binary selector parameter");
                }
                break;
            }
            Token::KEYWORD => {
                if let Token::WORD = parser.next_token()? {
                    parameters.push(parse_var(parser)?);
                } else {
                    return parser.error("Expected keyword selector parameter");
                }
            }
            _ => return parser.error("Expected method selector"),
        }
        if let (Token::KEYWORD, _) = parser.lookahead()? {
            continue;
        }
        break;
    }
    let (token, span2) = parser.lookahead()?;
    let rtype = if token == Token::SIGIL && parser.slice_at(span2) == "->" {
        parser.next_token()?;
        Some(parse_type_designator(parser)?)
    } else {
        None
    };
    // FIXME: This is the place where I could inform parser about instance
    // variables.
    let body = parser.parse_body()?;
    class.add_method(kind, MethodDefinition::new(span, selector, parameters, body, rtype));
    Ok(())
}

/// Tests and tools

pub fn parse_str(source: &str) -> Result<Expr, Unwind> {
    Parser::new(source).parse().map_err(|unwind| unwind.with_context(source))
}

#[cfg(test)]
pub mod utils {

    use crate::parse::*;

    pub fn block(span: Span, params: Vec<&str>, body: Expr) -> Expr {
        let mut p = span.start + 3;
        let mut blockparams = vec![];
        for param in params {
            let start = p;
            let end = start + param.len();
            p = end + 2;
            blockparams.push(Var::untyped(start..end, param.to_string()))
        }
        Expr::Block(span, blockparams, Box::new(body), None)
    }

    pub fn block_typed(span: Span, params: Vec<(&str, &str)>, body: Expr) -> Expr {
        let mut p = span.start + 3;
        let mut blockparams = vec![];
        for param in params {
            let start = p;
            let end = start + param.0.len();
            p = end + 4 + param.1.len();
            blockparams.push(Var::typed(start..end, param.0.to_string(), param.1.to_string()));
        }
        Expr::Block(span, blockparams, Box::new(body), None)
    }

    pub fn binary(span: Span, name: &str, left: Expr, right: Expr) -> Expr {
        left.send(Message {
            span,
            selector: name.to_string(),
            args: vec![right],
        })
    }

    pub fn bind(name: &str, value: Expr, body: Expr) -> Expr {
        Expr::Bind(name.to_string(), None, Box::new(value), Some(Box::new(body)))
    }

    pub fn bind_typed(name: &str, typename: &str, value: Expr, body: Expr) -> Expr {
        Expr::Bind(
            name.to_string(),
            Some(typename.to_string()),
            Box::new(value),
            Some(Box::new(body)),
        )
    }

    pub fn boolean(span: Span, value: bool) -> Expr {
        Expr::Const(span, Literal::Boolean(value))
    }

    pub fn class(span: Span, name: &str, instance_variables: Vec<&str>) -> Expr {
        let mut p = span.start + "class ".len() + name.len() + " { ".len();
        let mut vars = Vec::new();
        for v in instance_variables {
            vars.push(Var::untyped(p..p + v.len(), v.to_string()));
            p += v.len() + " ".len()
        }
        ClassDefinition::expr(span, name.to_string(), vars)
    }

    pub fn float(span: Span, value: f64) -> Expr {
        Expr::Const(span, Literal::Float(value))
    }

    pub fn int(span: Span, value: i64) -> Expr {
        Expr::Const(span, Literal::Integer(value))
    }

    pub fn keyword(span: Span, name: &str, left: Expr, args: Vec<Expr>) -> Expr {
        left.send(Message {
            span,
            selector: name.to_string(),
            args,
        })
    }
    pub fn method(
        span: Span,
        selector: &str,
        parameters: Vec<&str>,
        body: Expr,
    ) -> MethodDefinition {
        MethodDefinition::new(
            span,
            selector.to_string(),
            // FIXME: span
            parameters.iter().map(|name| Var::untyped(0..0, name.to_string())).collect(),
            body,
            None,
        )
    }

    pub fn seq(exprs: Vec<Expr>) -> Expr {
        Expr::Seq(exprs)
    }

    pub fn string(span: Span, value: &str) -> Expr {
        Expr::Const(span, Literal::String(value.to_string()))
    }

    pub fn typecheck(span: Span, expr: Expr, typename: &str) -> Expr {
        Expr::Typecheck(span, Box::new(expr), typename.to_string())
    }

    pub fn unary(span: Span, name: &str, left: Expr) -> Expr {
        left.send(Message {
            span,
            selector: name.to_string(),
            args: vec![],
        })
    }

    pub fn var(span: Span, name: &str) -> Expr {
        Expr::Var(Var::untyped(span, name.to_string()))
    }

}
