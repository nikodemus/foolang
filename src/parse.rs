use std::borrow::ToOwned;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::convert::Into;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::string::ToString;

use crate::tokenstream::{Span, Token, TokenStream};
use crate::unwind::{Error, Unwind};

trait TweakSpan {
    fn tweak(&mut self, shift: usize, extend: isize);
    fn shift(&mut self, shift: usize);
}

impl TweakSpan for Span {
    fn tweak(&mut self, shift: usize, extend: isize) {
        self.start += shift;
        self.end += shift;
        if extend < 0 {
            self.start -= (-extend) as usize;
        } else {
            self.end += extend as usize;
        }
    }
    fn shift(&mut self, shift: usize) {
        self.tweak(shift, 0);
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
    fn tweak_span(&mut self, shift: usize, extend: isize) {
        self.span.tweak(shift, extend);
        for elt in &mut self.data {
            elt.tweak_span(shift, extend);
        }
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
    fn tweak_span(&mut self, shift: usize, extend: isize) {
        self.span.tweak(shift, extend);
        self.value.tweak_span(shift, extend);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Bind {
    pub name: String,
    pub typename: Option<String>,
    pub value: Box<Expr>,
    pub body: Option<Box<Expr>>,
}

impl Bind {
    pub fn expr(
        name: String,
        typename: Option<String>,
        value: Box<Expr>,
        body: Option<Box<Expr>>,
    ) -> Expr {
        Expr::Bind(Bind {
            name,
            typename,
            value,
            body,
        })
    }
    fn tweak_span(&mut self, shift: usize, extend: isize) {
        self.value.tweak_span(shift, extend);
        if let Some(ref mut expr) = self.body {
            expr.tweak_span(shift, extend);
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub span: Span,
    pub params: Vec<Var>,
    pub body: Box<Expr>,
    pub rtype: Option<String>,
}

impl Block {
    pub fn expr(span: Span, params: Vec<Var>, body: Box<Expr>, rtype: Option<String>) -> Expr {
        Expr::Block(Block {
            span,
            params,
            body,
            rtype,
        })
    }
    fn tweak_span(&mut self, shift: usize, extend: isize) {
        self.span.tweak(shift, extend);
        for p in &mut self.params {
            p.span.tweak(shift, extend);
        }
        self.body.tweak_span(shift, extend);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Cascade {
    pub receiver: Box<Expr>,
    pub chains: Vec<Vec<Message>>,
}

impl Cascade {
    pub fn expr(receiver: Box<Expr>, chains: Vec<Vec<Message>>) -> Expr {
        Expr::Cascade(Cascade {
            receiver,
            chains,
        })
    }
    fn tweak_span(&mut self, shift: usize, extend: isize) {
        self.receiver.tweak_span(shift, extend);
        for chain in &mut self.chains {
            for message in chain {
                message.tweak_span(shift, extend);
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Chain {
    pub receiver: Box<Expr>,
    pub messages: Vec<Message>,
}

impl Chain {
    pub fn expr(receiver: Box<Expr>, messages: Vec<Message>) -> Expr {
        Expr::Chain(Chain {
            receiver,
            messages,
        })
    }
    fn tweak_span(&mut self, shift: usize, extend: isize) {
        self.receiver.tweak_span(shift, extend);
        for message in &mut self.messages {
            message.tweak_span(shift, extend);
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Const {
    pub span: Span,
    pub literal: Literal,
}

impl Const {
    pub fn expr(span: Span, literal: Literal) -> Expr {
        Expr::Const(Const {
            span,
            literal,
        })
    }
    fn tweak_span(&mut self, shift: usize, extend: isize) {
        self.span.tweak(shift, extend);
    }
}

// Span, Box<Expr>, Box<Expr>),

#[derive(Debug, PartialEq, Clone)]
pub struct Eq {
    pub span: Span,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

impl Eq {
    pub fn expr(span: Span, left: Box<Expr>, right: Box<Expr>) -> Expr {
        Expr::Eq(Eq {
            span,
            left,
            right,
        })
    }
    fn tweak_span(&mut self, shift: usize, extend: isize) {
        self.span.tweak(shift, extend);
        self.left.tweak_span(shift, extend);
        self.right.tweak_span(shift, extend);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Seq {
    pub exprs: Vec<Expr>,
}

impl Seq {
    fn expr(exprs: Vec<Expr>) -> Expr {
        Expr::Seq(Seq {
            exprs,
        })
    }
    fn tweak_span(&mut self, shift: usize, extend: isize) {
        for expr in &mut self.exprs {
            expr.tweak_span(shift, extend);
        }
    }
}

// Span, Box<Expr>, String),

#[derive(Debug, PartialEq, Clone)]
pub struct Typecheck {
    pub span: Span,
    pub expr: Box<Expr>,
    pub typename: String,
}

impl Typecheck {
    fn expr(span: Span, expr: Box<Expr>, typename: String) -> Expr {
        Expr::Typecheck(Typecheck {
            span,
            expr,
            typename,
        })
    }
    fn tweak_span(&mut self, shift: usize, extend: isize) {
        self.span.tweak(shift, extend);
        self.expr.tweak_span(shift, extend);
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
pub struct ClassExtension {
    pub span: Span,
    pub name: String,
    pub instance_methods: Vec<MethodDefinition>,
    pub class_methods: Vec<MethodDefinition>,
}

impl ClassExtension {
    pub fn new(span: Span, name: &str) -> Self {
        Self {
            span,
            name: name.to_string(),
            instance_methods: Vec::new(),
            class_methods: Vec::new(),
        }
    }

    pub fn add_method(&mut self, kind: MethodKind, method: MethodDefinition) {
        match kind {
            MethodKind::Instance => self.instance_methods.push(method),
            MethodKind::Class => self.class_methods.push(method),
        };
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Import {
    pub span: Span,
    pub path: PathBuf,
    pub prefix: String,
    pub name: Option<String>,
    pub body: Option<Box<Expr>>,
}

impl Import {
    pub fn expr<P: AsRef<Path>>(
        span: Span,
        path: P,
        prefix: &str,
        name: Option<&str>,
        body: Option<Expr>,
    ) -> Expr {
        Expr::Import(Import {
            span,
            path: path.as_ref().to_path_buf(),
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
    fn tweak_span(&mut self, shift: usize, ext: isize) {
        self.span.tweak(shift, ext);
        for arg in &mut self.args {
            arg.tweak_span(shift, ext);
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
    fn tweak_span(&mut self, shift: usize, extend: isize) {
        self.span.tweak(shift, extend);
        for var in &mut self.parameters {
            var.span.tweak(shift, extend);
        }
        self.body.tweak_span(shift, extend);
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
    Bind(Bind),
    Block(Block),
    Cascade(Cascade),
    Chain(Chain),
    ClassDefinition(ClassDefinition),
    ClassExtension(ClassExtension),
    Const(Const),
    Eq(Eq),
    Global(Global),
    Import(Import),
    Return(Return),
    Seq(Seq),
    Typecheck(Typecheck),
    Var(Var),
}

impl Expr {
    fn is_var(&self) -> bool {
        match self {
            Expr::Var(..) => true,
            _ => false,
        }
    }

    fn is_end_expr(&self) -> bool {
        match self {
            Expr::ClassDefinition(..) => true,
            Expr::ClassExtension(..) => true,
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
            Expr::Chain(chain) => {
                if let Expr::Cascade(mut cascade) = *chain.receiver {
                    cascade.chains.push(chain.messages);
                    Expr::Cascade(cascade)
                } else {
                    assert!(in_cascade);
                    Cascade::expr(Box::new(Expr::Chain(chain)), vec![])
                }
            }
            _ => {
                assert!(in_cascade);
                Cascade::expr(Box::new(self), vec![])
            }
        }
    }

    pub fn send(mut self, message: Message) -> Expr {
        match self {
            Expr::Chain(ref mut chain) => {
                chain.messages.push(message);
                self
            }
            _ => Chain::expr(Box::new(self), vec![message]),
        }
    }

    pub fn span(&self) -> Span {
        use Expr::*;
        let span = match self {
            Array(array) => &array.span,
            Assign(assign) => &assign.span,
            Bind(bind) => return bind.value.span(),
            Block(block) => &block.span,
            Cascade(cascade) => return cascade.receiver.span(),
            ClassDefinition(definition) => &definition.span,
            ClassExtension(extension) => &extension.span,
            Const(constant) => &constant.span,
            Eq(eq) => &eq.span,
            Global(global) => &global.span,
            Chain(chain) => return chain.receiver.span(),
            Import(import) => &import.span,
            Return(ret) => &ret.span,
            // FIXME: Questionable
            Seq(seq) => return seq.exprs[seq.exprs.len() - 1].span(),
            Typecheck(typecheck) => &typecheck.span,
            Var(var) => &var.span,
        };
        span.to_owned()
    }

    fn shift_span(&mut self, n: usize) {
        self.tweak_span(n, 0);
    }

    fn extend_span(&mut self, n: isize) {
        self.tweak_span(0, n);
    }

    fn tweak_span(&mut self, shift: usize, extend: isize) {
        use Expr::*;
        match self {
            Array(array) => array.tweak_span(shift, extend),
            Assign(assign) => assign.tweak_span(shift, extend),
            Bind(bind) => bind.tweak_span(shift, extend),
            Block(block) => block.tweak_span(shift, extend),
            Cascade(cascade) => cascade.tweak_span(shift, extend),
            Chain(chain) => chain.tweak_span(shift, extend),
            Const(constant) => constant.tweak_span(shift, extend),
            Eq(eq) => eq.tweak_span(shift, extend),
            Seq(seq) => seq.tweak_span(shift, extend),
            Typecheck(typecheck) => typecheck.tweak_span(shift, extend),
            ClassDefinition(class) => {
                class.span.tweak(shift, extend);
                for var in &mut class.instance_variables {
                    var.span.tweak(shift, extend);
                }
                for m in &mut class.instance_methods {
                    m.tweak_span(shift, extend);
                }
                for m in &mut class.class_methods {
                    m.tweak_span(shift, extend);
                }
            }
            ClassExtension(ext) => {
                ext.span.tweak(shift, extend);
                for m in &mut ext.instance_methods {
                    m.tweak_span(shift, extend);
                }
                for m in &mut ext.class_methods {
                    m.tweak_span(shift, extend);
                }
            }
            Global(global) => {
                global.span.tweak(shift, extend);
            }
            Import(import) => {
                import.span.tweak(shift, extend);
                if let Some(ref mut body) = import.body {
                    body.tweak_span(shift, extend);
                }
            }
            Return(ret) => {
                ret.span.tweak(shift, extend);
                ret.value.tweak_span(shift, extend);
            }
            Var(var) => {
                var.span.tweak(shift, extend);
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
    // Directory to use for relative imports. Normally the directory of
    // the source file, but different in REPL, etc.
    root: PathBuf,
}

impl<'a> Parser<'a> {
    pub fn new<P: AsRef<Path>>(source: &'a str, root: P) -> Parser<'a> {
        Parser {
            source,
            token_table: make_token_table(),
            name_table: make_name_table(),
            state: RefCell::new(ParserState {
                tokenstream: TokenStream::new(source),
                lookahead: VecDeque::new(),
                span: 0..0,
            }),
            root: root.as_ref().to_path_buf(),
        }
    }

    pub fn parse(&mut self) -> Result<Expr, Unwind> {
        let res = self.parse_expr(0)?;
        if Token::EOF == self.next_token()? {
            Ok(res)
        } else {
            Unwind::error_at(self.span(), "Incomplete parse")
        }
    }

    pub fn parse_interpolated_block(&self, span: Span) -> Result<(Expr, usize), Unwind> {
        let subparser = Parser::new(self.slice_at(span.clone()), &self.root);
        match subparser.parse_prefix() {
            Err(Unwind::Exception(Error::EofError(_), _)) => {
                Unwind::error_at(span, "Unterminated string interpolation.")
            }
            Err(unwind) => Err(unwind.shift_span(span.start)),
            Ok(Expr::Block(mut block)) => {
                block.span.shift(span.start);
                if !block.params.is_empty() {
                    return Unwind::error_at(block.span, "Interpolated block has variables.");
                }
                if block.rtype.is_some() {
                    return Unwind::error_at(block.span, "Interpolated block has a return type.");
                }
                let mut expr = *block.body;
                expr.shift_span(span.start);
                return Ok((expr, block.span.end));
            }
            Ok(other) => {
                let mut errspan = other.span();
                errspan.shift(span.start);
                Unwind::error_at(errspan, "Interpolation not a block.")
            }
        }
    }

    pub fn parse_seq(&self) -> Result<Expr, Unwind> {
        let seq = self.parse_expr(1)?;
        // FIXME: Terrible KLUDGE.
        //
        // 1. Expressions like 'class' and 'extend' do not consume
        //    their 'end' since we use it to sequence toplevel definitions.
        // 2. If they appear in a sequence context it will leave the
        //    end behind to be discovered in a _prefix_ context.
        //
        // So we clean it up.
        //
        let (token, span) = self.lookahead()?;
        if token == Token::WORD && self.slice_at(span) == "end" {
            let is_class_def = if let Expr::Seq(seq) = &seq {
                seq.exprs[seq.exprs.len() - 1].is_end_expr()
            } else {
                seq.is_end_expr()
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
            Syntax::Operator(_, _, _) => {
                let operator = self.tokenstring();
                let span = self.span();
                Ok(self.parse_expr(PREFIX_PRECEDENCE)?.send(Message {
                    span,
                    selector: format!("prefix{}", operator),
                    args: vec![],
                }))
            }
        }
    }

    fn parse_suffix_syntax(&self, syntax: &Syntax, left: Expr) -> Result<Expr, Unwind> {
        match syntax {
            Syntax::General(_, suffix, precedence) => suffix(self, left, *precedence),
            Syntax::Operator(_, _, precedence) => {
                let operator = self.tokenstring();
                Ok(left.send(Message {
                    span: self.span(),
                    selector: operator,
                    args: vec![self.parse_expr(*precedence)?],
                }))
            }
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

    fn dotted_name_at(&self, point: usize, star: bool) -> Result<Option<Span>, Unwind> {
        let ((token1, span1), (token2, span2)) = self.lookahead2()?;
        if span1.start != point {
            return Ok(None);
        }
        // Starts at point!

        if token1 != Token::SIGIL || self.slice_at(span1.clone()) != "." {
            return Ok(None);
        }
        // Starts with a dot!

        if span1.end != span2.start {
            return Ok(None);
        }
        // Next token follows immediately without intervening whitespace

        if star {
            if token2 != Token::SIGIL || self.slice_at(span2.clone()) != "*" {
                return Ok(None);
            }
        // Star wanted, next token is a star
        } else {
            if token2 != Token::WORD {
                return Ok(None);
            }
            // Star not wanted, Next token is a word
        }

        self.next_token()?;
        self.next_token()?;
        Ok(Some(point..span2.end))
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

const UNKNOWN_OPERATOR_SYNTAX: Syntax = Syntax::Operator(true, true, 10);

fn make_name_table() -> NameTable {
    let mut table: NameTable = HashMap::new();
    let t = &mut table;

    Syntax::def(t, "class", class_prefix, invalid_suffix, precedence_0);
    Syntax::def(t, "extend", extend_prefix, invalid_suffix, precedence_0);
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

    Syntax::op(t, "^", false, true, 100);

    Syntax::op(t, "*", false, true, 90);
    Syntax::op(t, "/", false, true, 90);
    Syntax::op(t, "%", false, true, 90);

    Syntax::op(t, "+", false, true, 80);
    Syntax::op(t, "-", true, true, 80);

    Syntax::op(t, "<<", false, true, 70);
    Syntax::op(t, ">>", false, true, 70);

    Syntax::op(t, "&", false, true, 60);
    Syntax::op(t, "|", false, true, 60);

    Syntax::op(t, "<", false, true, 50);
    Syntax::op(t, "<=", false, true, 50);
    Syntax::op(t, ">", false, true, 50);
    Syntax::op(t, ">=", false, true, 50);
    Syntax::op(t, "==", false, true, 50);
    Syntax::op(t, "!=", false, true, 50);

    Syntax::op(t, "&&", false, true, 40);
    Syntax::op(t, "||", false, true, 30);

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
    Ok(Const::expr(parser.span(), Literal::Boolean(false)))
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
    Ok(Eq::expr(span, Box::new(left), Box::new(right)))
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
    let syntax = parser.name_table.get(slice).unwrap_or(&UNKNOWN_OPERATOR_SYNTAX);
    parser.syntax_precedence(syntax, span)
}

fn operator_prefix(parser: &Parser) -> Result<Expr, Unwind> {
    let syntax = parser.name_table.get(parser.slice()).unwrap_or(&UNKNOWN_OPERATOR_SYNTAX);
    parser.parse_prefix_syntax(syntax)
}

fn operator_suffix(parser: &Parser, left: Expr, _: PrecedenceFunction) -> Result<Expr, Unwind> {
    let syntax = parser.name_table.get(parser.slice()).unwrap_or(&UNKNOWN_OPERATOR_SYNTAX);
    parser.parse_suffix_syntax(syntax, left)
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
    Ok(Const::expr(parser.span(), Literal::Boolean(true)))
}

fn typecheck_suffix(
    parser: &Parser,
    left: Expr,
    _precedence: PrecedenceFunction,
) -> Result<Expr, Unwind> {
    match parser.next_token()? {
        Token::WORD => Ok(Typecheck::expr(parser.span(), Box::new(left), parser.tokenstring())),
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
    let mut exprs = if let Expr::Seq(left_seq) = left {
        left_seq.exprs
    } else {
        vec![left]
    };
    let right = parser.parse_expr(precedence(parser, parser.span())?)?;
    if let Expr::Seq(mut right_seq) = right {
        exprs.append(&mut right_seq.exprs);
    } else {
        exprs.push(right);
    }
    Ok(Seq::expr(exprs))
}

fn end_suffix(parser: &Parser, left: Expr, precedence: PrecedenceFunction) -> Result<Expr, Unwind> {
    let mut exprs = if let Expr::Seq(left_seq) = left {
        if left_seq.exprs[left_seq.exprs.len() - 1].is_end_expr() {
            left_seq.exprs
        } else {
            return parser.error("Unexpected 'end': not after class definition.");
        }
    } else {
        if left.is_end_expr() {
            vec![left]
        } else {
            return parser.error("Unexpected 'end': not after class definition.");
        }
    };
    let (token, _) = parser.lookahead()?;
    if token == Token::EOF {
        if exprs.len() > 1 {
            return Ok(Seq::expr(exprs));
        } else {
            return Ok(exprs.pop().unwrap());
        }
    }
    let right = parser.parse_expr(precedence(parser, parser.span())?)?;
    if let Expr::Seq(mut right_seq) = right {
        exprs.append(&mut right_seq.exprs);
    } else {
        exprs.push(right);
    }
    Ok(Seq::expr(exprs))
}

fn parse_record(parser: &Parser) -> Result<Expr, Unwind> {
    let start = parser.span().start;
    let mut selector = String::new();
    let mut args = Vec::new();
    loop {
        match parser.next_token()? {
            Token::KEYWORD => {
                selector.push_str(parser.slice());
                args.push(parser.parse_expr(0)?);
                match parser.next_token()? {
                    Token::SIGIL if "," == parser.slice() => continue,
                    Token::SIGIL if "}" == parser.slice() => break,
                    _ => return parser.error("Malformed record")
                }
            }
            Token::SIGIL if "}" == parser.slice() => break,
            _ => return parser.error("Malformed record")
        }
    }
    let end = parser.span().end;
    // This kind of indicates I need a more felicitious representation
    // in order to be able to reliably print back things without converting
    // {x: 42} to Record x: 42 accidentally. (Or I need to not have this syntax).
    Ok(Expr::Var(Var::untyped(0..0, "Record".to_string())).send(
        Message {
            span: start..end,
            selector,
            args
        })
    )
}

fn parse_block(parser: &Parser) -> Result<Expr, Unwind> {
    let start = parser.span();
    assert_eq!("{", parser.slice());
    let mut params = vec![];
    let (token, span) = parser.lookahead()?;
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
        Const::expr(0..0, Literal::Boolean(false))
    } else {
        parser.parse_seq()?
    };
    let end = parser.next_token()?;
    // FIXME: hardcoded {
    // Would be nice to be able to swap between [] and {} and
    // keep this function same,
    if end == Token::SIGIL && parser.slice() == "}" {
        Ok(Block::expr(start.start..parser.span().end, params, Box::new(body), rtype))
    } else if end == Token::EOF {
        parser.eof_error("Unexpected EOF while pasing a block: expected } as block terminator")
    } else {
        parser.error("Expected } as block terminator")
    }
}

fn block_prefix(parser: &Parser) -> Result<Expr, Unwind> {
    //
    // { keyword: expr } --> Record
    //
    // { ... } -> Block
    //
    match parser.lookahead()? {
        (Token::KEYWORD, _) => return parse_record(parser),
        _ => return parse_block(parser)
    }
}

fn import_prefix(parser: &Parser) -> Result<Expr, Unwind> {
    let import_start = parser.span().start;
    let (token, name_span) = parser.lookahead()?;
    let mut spec = String::new();
    let mut relative = false;
    if let Some(dotted) = parser.dotted_name_at(name_span.start, false)? {
        spec.push_str(parser.slice_at(dotted.clone()));
        relative = true;
    } else if Token::WORD == token {
        parser.next_token()?;
        spec.push_str(parser.slice());
    }
    if spec.len() > 0 {
        // Deal with .*
        if let Some(star) = parser.dotted_name_at(parser.span().end, true)? {
            spec.push_str(parser.slice_at(star));
        }
        let name_end = parser.span().end;
        let body = if Token::EOF == parser.lookahead()?.0 {
            parser.next_token()?;
            None
        } else {
            Some(Box::new(parser.parse_seq()?))
        };
        let mut path = PathBuf::new();
        let mut prefix = String::new();
        let mut name = None;
        let mut parts = if relative {
            path.push(&parser.root);
            spec[1..].split(".").peekable()
        } else {
            spec.split(".").peekable()
        };
        while let Some(part) = parts.next() {
            assert!(!part.is_empty());
            let is_name = part == "*" || part.chars().next().unwrap().is_uppercase();
            if parts.peek().is_some() {
                if is_name {
                    return Unwind::error_at(
                        import_start..parser.span().start,
                        "Illegal import: invalid module name",
                    );
                }
                path.push(part);
            } else {
                if is_name {
                    name = Some(part.to_string());
                } else {
                    path.push(part);
                    prefix.push_str(part);
                };
            }
        }
        path.set_extension("foo");
        Ok(Expr::Import(Import {
            span: import_start..name_end,
            path,
            prefix,
            name,
            body,
        }))
    } else {
        return parser.error_at(name_span, "Expected module name");
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
                class.add_method(MethodKind::Class, parse_method(parser)?);
                continue;
            } else {
                return parser.error("Expected class method");
            }
        }
        if next == Token::WORD && parser.slice() == "method" {
            class.add_method(MethodKind::Instance, parse_method(parser)?);
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
        if next == Token::COMMENT || next == Token::BLOCK_COMMENT {
            continue;
        }
        return parser.error("Expected method or end");
    }
    Ok(Expr::ClassDefinition(class))
}

fn extend_prefix(parser: &Parser) -> Result<Expr, Unwind> {
    // FIXME: span is the span of the extension, but maybe it would be better if these
    // had all their own spans.
    //
    // FIXME: duplicated class_prefix pretty much.
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
    let mut class = ClassExtension::new(span, &class_name);
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
                class.add_method(MethodKind::Class, parse_method(parser)?);
                continue;
            } else {
                return parser.error("Expected class method");
            }
        }
        if next == Token::WORD && parser.slice() == "method" {
            class.add_method(MethodKind::Instance, parse_method(parser)?);
            continue;
        }
        return parser.error("Expected method or end");
    }
    Ok(Expr::ClassExtension(class))
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
    // FIXME: Should just return false here instead of playing with
    // an Option for body.
    let body = if eof {
        None
    } else {
        Some(Box::new(parser.parse_seq()?))
    };
    Ok(Bind::expr(name, typename, Box::new(value), body))
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
        Ok(Const::expr(parser.span(), Literal::Integer(integer)))
    }
    // Binary case
    else if slice.len() > 2 && ("0b" == &slice[0..2] || "0B" == &slice[0..2]) {
        let integer = match i64::from_str_radix(&slice[2..], 2) {
            Ok(i) => i,
            Err(_) => return parser.error("Malformed binary number"),
        };
        Ok(Const::expr(parser.span(), Literal::Integer(integer)))
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
                        Ok(f) => return Ok(Const::expr(parser.span(), Literal::Float(f))),
                        Err(_) => return parser.error("Malformed number"),
                    }
                }
            }
        }
        Ok(Const::expr(parser.span(), Literal::Integer(decimal)))
    }
}

fn return_prefix(parser: &Parser) -> Result<Expr, Unwind> {
    Ok(Return::expr(parser.span(), parser.parse_expr(SEQ_PRECEDENCE)?))
}

/// Takes care of \n, and such. Terminates on { or end of string.
fn scan_string_part(parser: &Parser, span: Span) -> Result<Expr, Unwind> {
    // println!("scan: '{}'", parser.slice_at(span.clone()));
    let mut chars = parser.slice_at(span.clone()).char_indices();
    let mut res = String::new();
    let start = span.start;
    loop {
        match chars.next() {
            None => return Ok(Const::expr(span, Literal::String(res))),
            Some((pos0, '\\')) => match chars.next() {
                None => {
                    return Unwind::error_at(
                        start + pos0..start + pos0 + 1,
                        "Literal string ends on escape.",
                    )
                }
                Some((_, '"')) => res.push_str("\""),
                Some((_, '\\')) => res.push_str("\\"),
                Some((_, 'n')) => res.push_str("\n"),
                Some((_, 't')) => res.push_str("\t"),
                Some((_, 'r')) => res.push_str("\r"),
                Some((_, '{')) => res.push_str("{"),
                Some((pos1, _)) => {
                    return Unwind::error_at(
                        start + pos0..start + pos1,
                        "Unknown escape sequence in literal string.",
                    )
                }
            },
            Some((pos, '{')) => return Ok(Const::expr(start..start + pos, Literal::String(res))),
            Some((_, c)) => res.push(c),
        }
    }
}

fn string_prefix(parser: &Parser) -> Result<Expr, Unwind> {
    let slice = parser.slice();
    let full = parser.span();
    let mut span = full.clone();

    // Strip quotes from ends of span
    let mut n = 0;
    while n < slice.len() - n && &slice[n..n + 1] == "\"" {
        n += 1;
    }
    span = (span.start + n)..(span.end - n);

    // Collect literal and interpolated parts
    let mut parts = Vec::new();

    loop {
        let literal = scan_string_part(parser, span.clone())?;
        span = literal.span().end..span.end;
        parts.push(literal);
        if span.start < span.end {
            let (interp, end) = parser.parse_interpolated_block(span.clone())?;
            span = end..span.end;
            parts.push(interp);
        } else {
            break;
        }
    }

    // Extend first and last part spans to cover the quotes, leaving
    // the parts reversed for what comes nest.
    parts[0].extend_span(-(n as isize));
    parts.reverse();
    parts[0].extend_span(n as isize);

    // Append them all togather.
    let mut expr = match parts.pop() {
        None => return Ok(Const::expr(span, Literal::String("".to_string()))),
        Some(part) => part,
    };
    while let Some(right) = parts.pop() {
        let rspan = right.span();
        expr = expr.send(Message {
            span: rspan.clone(),
            selector: "append:".to_string(),
            args: vec![right.send(Message {
                span: rspan,
                selector: "toString".to_string(),
                args: vec![],
            })],
        })
    }

    Ok(expr)
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

fn parse_method(parser: &Parser) -> Result<MethodDefinition, Unwind> {
    assert_eq!(parser.slice(), "method");
    let span = parser.span();
    let mut selector = String::new();
    let mut parameters = Vec::new();
    let mut prefix = false;
    loop {
        let token = parser.next_token()?;
        selector.push_str(parser.slice());
        match token {
            Token::WORD => {
                assert!(parameters.is_empty());
                if "prefix" == &selector {
                    prefix = true;
                    continue;
                }
                break;
            }
            Token::SIGIL => {
                assert!(parameters.is_empty());
                if prefix {
                    break;
                }
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
    Ok(MethodDefinition::new(span, selector, parameters, body, rtype))
}

/// Tests and tools

pub fn parse_str_in_path<P: AsRef<Path>>(source: &str, root: P) -> Result<Expr, Unwind> {
    // FIXME: Don't like this parse_str/ path.
    Parser::new(source, root).parse().map_err(|unwind| unwind.with_context(source))
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
        Block::expr(span, blockparams, Box::new(body), None)
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
        Block::expr(span, blockparams, Box::new(body), None)
    }

    pub fn binary(span: Span, name: &str, left: Expr, right: Expr) -> Expr {
        left.send(Message {
            span,
            selector: name.to_string(),
            args: vec![right],
        })
    }

    pub fn bind(name: &str, value: Expr, body: Expr) -> Expr {
        Bind::expr(name.to_string(), None, Box::new(value), Some(Box::new(body)))
    }

    pub fn bind_typed(name: &str, typename: &str, value: Expr, body: Expr) -> Expr {
        Bind::expr(
            name.to_string(),
            Some(typename.to_string()),
            Box::new(value),
            Some(Box::new(body)),
        )
    }

    pub fn boolean(span: Span, value: bool) -> Expr {
        Const::expr(span, Literal::Boolean(value))
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
        Const::expr(span, Literal::Float(value))
    }

    pub fn int(span: Span, value: i64) -> Expr {
        Const::expr(span, Literal::Integer(value))
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
        Seq::expr(exprs)
    }

    pub fn string(span: Span, value: &str) -> Expr {
        Const::expr(span, Literal::String(value.to_string()))
    }

    pub fn typecheck(span: Span, expr: Expr, typename: &str) -> Expr {
        Typecheck::expr(span, Box::new(expr), typename.to_string())
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
