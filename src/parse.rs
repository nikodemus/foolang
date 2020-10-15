use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::convert::Into;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::str::FromStr;
use std::string::ToString;

use crate::source_location::{SourceLocation, Span};
use crate::tokenstream::{Token, TokenStream};
use crate::unwind::{Error, Unwind};

use crate::def::*;
use crate::expr::*;
use crate::syntax::Syntax;

pub type Parse = Result<Syntax, Unwind>;
pub type ExprParse = Result<Expr, Unwind>;

type PrefixParser = fn(&Parser) -> Parse;
type SuffixParser = fn(&Parser, Expr, PrecedenceFunction) -> ExprParse;
// FIXME: can I remove the span from here?
type PrecedenceFunction = fn(&Parser, Span) -> Result<usize, Unwind>;

#[derive(Clone)]
enum ParserSyntax {
    General(PrefixParser, SuffixParser, PrecedenceFunction),
    Operator(bool, bool, usize),
}

type TokenTable = HashMap<Token, ParserSyntax>;
type NameTable = HashMap<String, ParserSyntax>;

pub struct ParserState<'a> {
    tokenstream: TokenStream<'a>,
    lookahead: VecDeque<(Token, Span)>,
    span: Span,
}

impl<'a> ParserState<'a> {
    fn _scan(&mut self) -> Result<(Token, Span), Unwind> {
        let token = self.tokenstream.scan()?;
        Ok((token, self.tokenstream.span()))
    }

    fn next_token(&mut self) -> Result<Token, Unwind> {
        let (token, span) = if self.lookahead.is_empty() {
            self._scan()?
        } else {
            self.lookahead.pop_front().unwrap()
        };
        self.span = span;
        Ok(token)
    }

    fn lookahead(&mut self) -> Result<(Token, Span), Unwind> {
        if self.lookahead.is_empty() {
            let look = self._scan()?;
            self.lookahead.push_back(look);
        }
        Ok(self.lookahead.get(0).unwrap().clone())
    }

    fn lookahead2(&mut self) -> Result<((Token, Span), (Token, Span)), Unwind> {
        while self.lookahead.len() < 2 {
            let look = self._scan()?;
            self.lookahead.push_back(look);
        }
        Ok((self.lookahead.get(0).unwrap().clone(), self.lookahead.get(1).unwrap().clone()))
    }

    fn tokenstring(&self) -> String {
        self.tokenstream.slice_at(self.span.clone()).to_string()
    }
}

pub struct Parser<'a> {
    source: &'a str,
    path: Option<Rc<PathBuf>>,
    token_table: TokenTable,
    name_table: NameTable,
    state: RefCell<ParserState<'a>>,
    // Directory to use for relative imports. Normally the directory of
    // the source file, but different in REPL, etc.
    root: PathBuf,
}

impl<'a> Parser<'a> {
    pub(crate) fn parse_file<P: AsRef<Path>, R>(
        file: P,
        root: P,
        fun: impl FnOnce(&mut Parser) -> Result<R, Unwind>,
    ) -> Result<R, Unwind> {
        let file = file.as_ref();
        let source = match std::fs::read_to_string(file) {
            Ok(code) => code,
            Err(_err) => {
                return Unwind::error(&format!("Could not read file: {}", file.to_string_lossy()))
            }
        };
        // println!("PARSE:\n{}", &source);
        let mut parser = Parser::new_with_path(file, &source, root);
        fun(&mut parser)
    }

    pub(crate) fn new<P: AsRef<Path>>(source: &'a str, root: P) -> Parser<'a> {
        Parser {
            source,
            path: None,
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

    pub(crate) fn new_with_path<P: AsRef<Path>>(
        path: &Path,
        source: &'a str,
        root: P,
    ) -> Parser<'a> {
        Parser {
            source,
            path: Some(Rc::new(path.to_path_buf())),
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

    pub(crate) fn parse(&mut self) -> Parse {
        self._parse()
    }

    fn _parse(&self) -> Parse {
        self.parse_at_precedence(1)
    }

    pub(crate) fn parse_interpolated_block(
        &self,
        source_location: SourceLocation,
    ) -> Result<(Expr, usize), Unwind> {
        let span = source_location.get_span();
        let subparser = Parser::new(self.slice_at(span.clone()), &self.root);
        match subparser.parse_prefix_expr() {
            Err(Unwind::Panic(Error::EofError(_), _)) => {
                Unwind::error_at(source_location, "Unterminated string interpolation.")
            }
            Err(unwind) => Err(unwind.shift_span(span.start)),
            Ok(Expr::Block(mut block)) => {
                block.source_location.shift_span(span.start);
                if !block.params.is_empty() {
                    return Unwind::error_at(
                        block.source_location.clone(),
                        "Interpolated block has variables.",
                    );
                }
                if block.rtype.is_some() {
                    return Unwind::error_at(
                        block.source_location.clone(),
                        "Interpolated block has a return type.",
                    );
                }
                let mut expr = *block.body;
                expr.shift_span(span.start);
                return Ok((expr, block.source_location.end()));
            }
            Ok(other) => {
                let mut errloc = other.source_location();
                errloc.shift_span(span.start);
                Unwind::error_at(errloc, "Interpolation not a block.")
            }
        }
    }

    pub(crate) fn parse_expr(&self, precedence: usize) -> ExprParse {
        match self.parse_at_precedence(precedence)? {
            Syntax::Expr(e) => Ok(e),
            Syntax::Def(d) => {
                Unwind::error_at(d.source_location(), "Definition where expression was expected")
            }
        }
    }

    pub(crate) fn parse_seq(&self) -> ExprParse {
        self.parse_expr(1)
    }

    pub(crate) fn parse_single(&self) -> ExprParse {
        // Dot has precedence 2.
        self.parse_expr(2)
    }

    pub(crate) fn parse_at_precedence(&self, precedence: usize) -> Parse {
        match self.parse_prefix()? {
            Syntax::Def(def) => {
                // println!(" -> def: {:?}", &def);
                Ok(Syntax::Def(def))
            }
            Syntax::Expr(expr) => {
                // println!(" -> exp: {:?}", &expr);
                Ok(Syntax::Expr(self.parse_tail(expr, precedence)?))
            }
        }
    }

    pub(crate) fn parse_tail(&self, mut expr: Expr, precedence: usize) -> ExprParse {
        while precedence < self.next_precedence()? {
            expr = self.parse_suffix(expr)?;
        }
        Ok(expr)
    }

    fn parse_prefix_expr(&self) -> ExprParse {
        match self.parse_prefix()? {
            Syntax::Expr(e) => Ok(e),
            Syntax::Def(_) => Unwind::error("Definition whwre expression was expected!"),
        }
    }

    fn parse_prefix(&self) -> Parse {
        let token = self.next_token()?;
        match self.token_table.get(&token) {
            Some(token_syntax) => self.parse_prefix_syntax(token_syntax),
            None => unimplemented!("Don't know how to parse {:?} in prefix position.", token),
        }
    }

    fn parse_suffix(&self, left: Expr) -> ExprParse {
        let token = self.next_token()?;
        match self.token_table.get(&token) {
            Some(token_syntax) => self.parse_suffix_syntax(token_syntax, left),
            None => unimplemented!("Don't know how to parse {:?} in suffix position.", token),
        }
    }

    fn next_precedence(&self) -> Result<usize, Unwind> {
        let (token, span) = self.lookahead()?;
        match self.token_table.get(&token) {
            Some(token_syntax) => self.syntax_precedence(token_syntax, span),
            None => unimplemented!("No precedence defined for {:?}", token),
        }
    }

    fn parse_prefix_syntax(&self, syntax: &ParserSyntax) -> Parse {
        match syntax {
            ParserSyntax::General(prefix, _, _) => prefix(self),
            ParserSyntax::Operator(_, _, _) => {
                let operator = self.tokenstring();
                let source_location = self.source_location();
                Ok(Syntax::Expr(self.parse_expr(PREFIX_PRECEDENCE)?.send(Message {
                    source_location,
                    selector: format!("prefix{}", operator),
                    args: vec![],
                })))
            }
        }
    }

    fn parse_suffix_syntax(&self, syntax: &ParserSyntax, left: Expr) -> ExprParse {
        match syntax {
            ParserSyntax::General(_, suffix, precedence) => suffix(self, left, *precedence),
            ParserSyntax::Operator(_, _, precedence) => {
                let operator = self.tokenstring();
                let mut source = self.source_location();
                let arg = self.parse_expr(*precedence)?;
                source.extend_span_to(arg.span().end);
                Ok(left.send(Message {
                    source_location: source,
                    selector: operator,
                    args: vec![arg],
                }))
            }
        }
    }

    fn syntax_precedence(&self, syntax: &ParserSyntax, span: Span) -> Result<usize, Unwind> {
        match syntax {
            ParserSyntax::General(_, _, precedence) => precedence(self, span),
            ParserSyntax::Operator(_, _, precedence) => Ok(*precedence),
        }
    }

    pub(crate) fn lookahead(&self) -> Result<(Token, Span), Unwind> {
        self.state.borrow_mut().lookahead()
    }

    pub(crate) fn lookahead2(&self) -> Result<((Token, Span), (Token, Span)), Unwind> {
        self.state.borrow_mut().lookahead2()
    }

    pub(crate) fn next_token(&self) -> Result<Token, Unwind> {
        self.state.borrow_mut().next_token()
    }

    pub(crate) fn tokenstring(&self) -> String {
        self.state.borrow().tokenstring()
    }

    pub(crate) fn span(&self) -> Span {
        self.state.borrow().span.clone()
    }

    pub(crate) fn code(&self) -> &'a str {
        self.source
    }

    pub(crate) fn source_location(&self) -> SourceLocation {
        match &self.path {
            None => SourceLocation::Span(self.span()),
            Some(path) => SourceLocation::path(path, &self.span()),
        }
    }

    pub(crate) fn slice(&self) -> &str {
        &self.source[self.span()]
    }

    pub(crate) fn at_eof(&self) -> bool {
        if let Ok((Token::EOF, _)) = self.lookahead() {
            return true;
        } else {
            return false;
        }
    }

    pub(crate) fn at_comment(&self) -> bool {
        match self.lookahead() {
            Ok((Token::COMMENT, _)) => true,
            Ok((Token::BLOCK_COMMENT, _)) => true,
            _ => false,
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

    pub(crate) fn slice_at(&self, span: Span) -> &str {
        &self.source[span]
    }

    pub(crate) fn eof_error<T>(&self, problem: &str) -> Result<T, Unwind> {
        Unwind::eof_error_at(self.source_location(), problem)
    }

    pub(crate) fn error<T>(&self, problem: &str) -> Result<T, Unwind> {
        Unwind::error_at(self.source_location(), problem)
    }
}

impl ParserSyntax {
    fn def<A, T>(
        table: &mut HashMap<T, ParserSyntax>,
        key: A,
        prefix_parser: PrefixParser,
        suffix_parser: SuffixParser,
        precedence_function: PrecedenceFunction,
    ) where
        T: std::cmp::Eq,
        T: std::hash::Hash,
        A: Into<T>,
    {
        table.insert(
            key.into(),
            ParserSyntax::General(prefix_parser, suffix_parser, precedence_function),
        );
    }
    fn op(table: &mut NameTable, key: &str, is_prefix: bool, is_binary: bool, precedence: usize) {
        assert!(key.len() > 0);
        assert!(is_prefix || is_binary);
        assert!(10 <= precedence);
        assert!(precedence <= 100);
        table.insert(key.to_string(), ParserSyntax::Operator(is_prefix, is_binary, precedence));
    }
}

fn make_token_table() -> TokenTable {
    let mut table: TokenTable = HashMap::new();
    let t = &mut table;
    use Token::*;

    // Literals should appear in prefix-positions only, hence precedence_invald
    ParserSyntax::def(t, HEX_INTEGER, number_prefix, invalid_suffix, precedence_invalid);
    ParserSyntax::def(t, BIN_INTEGER, number_prefix, invalid_suffix, precedence_invalid);
    ParserSyntax::def(t, DEC_INTEGER, number_prefix, invalid_suffix, precedence_invalid);
    ParserSyntax::def(t, SINGLE_FLOAT, number_prefix, invalid_suffix, precedence_invalid);
    ParserSyntax::def(t, DOUBLE_FLOAT, number_prefix, invalid_suffix, precedence_invalid);
    ParserSyntax::def(t, STRING, string_prefix, invalid_suffix, precedence_invalid);
    // Comments
    ParserSyntax::def(t, COMMENT, ignore_prefix, ignore_suffix, precedence_1000);
    ParserSyntax::def(t, BLOCK_COMMENT, ignore_prefix, ignore_suffix, precedence_1000);
    // Others
    ParserSyntax::def(t, WORD, identifier_prefix, identifier_suffix, identifier_precedence);
    ParserSyntax::def(t, SIGIL, operator_prefix, operator_suffix, operator_precedence);
    ParserSyntax::def(t, KEYWORD, invalid_prefix, keyword_suffix, precedence_9);
    ParserSyntax::def(t, EOF, eof_prefix, eof_suffix, precedence_0);

    table
}

// KLUDGE: couple of places which don't have convenient access to the table
// need this.
const PREFIX_PRECEDENCE: usize = 1000;

const UNKNOWN_OPERATOR_SYNTAX: ParserSyntax = ParserSyntax::Operator(true, true, 10);

fn make_name_table() -> NameTable {
    let mut table: NameTable = HashMap::new();
    let t = &mut table;

    ParserSyntax::def(t, "class", class_prefix, invalid_suffix, precedence_0);
    ParserSyntax::def(t, "define", define_prefix, invalid_suffix, precedence_0);
    ParserSyntax::def(t, "extend", extend_prefix, invalid_suffix, precedence_0);
    ParserSyntax::def(t, "import", import_prefix, invalid_suffix, precedence_0);
    ParserSyntax::def(t, "interface", interface_prefix, invalid_suffix, precedence_0);
    ParserSyntax::def(t, ",", invalid_prefix, invalid_suffix, precedence_0);
    ParserSyntax::def(t, "->", invalid_prefix, invalid_suffix, precedence_0);
    ParserSyntax::def(t, "defaultConstructor", invalid_prefix, invalid_suffix, precedence_0);
    // NOTE: parse_seq vs parse_single have special knowledge about precedence
    // 2!
    ParserSyntax::def(t, "!", invalid_prefix, invalid_suffix, precedence_1);
    ParserSyntax::def(t, ".", invalid_prefix, sequence_suffix, precedence_2);
    ParserSyntax::def(t, "let", let_prefix, invalid_suffix, precedence_3);
    ParserSyntax::def(t, "return", return_prefix, invalid_suffix, precedence_3);
    ParserSyntax::def(t, "panic", panic_prefix, invalid_suffix, precedence_3);
    ParserSyntax::def(t, ";", invalid_prefix, cascade_suffix, precedence_3);
    ParserSyntax::def(t, "=", invalid_prefix, assign_suffix, precedence_4);
    ParserSyntax::def(t, "is", invalid_prefix, is_suffix, precedence_10);
    ParserSyntax::def(t, "::", invalid_prefix, typecheck_suffix, precedence_1000);

    // FIXME: Should opening group sigils use prefix precedence?
    ParserSyntax::def(t, "[", array_prefix, invalid_suffix, precedence_3);
    ParserSyntax::def(t, "]", invalid_prefix, invalid_suffix, precedence_0);
    ParserSyntax::def(t, "(", paren_prefix, invalid_suffix, precedence_3);
    ParserSyntax::def(t, ")", invalid_prefix, invalid_suffix, precedence_0);
    ParserSyntax::def(t, "{", block_prefix, invalid_suffix, precedence_3);
    ParserSyntax::def(t, "}", invalid_prefix, invalid_suffix, precedence_0);

    ParserSyntax::def(t, "#", literal_prefix, invalid_suffix, precedence_0);
    ParserSyntax::def(t, "$", dynamic_var_prefix, invalid_suffix, precedence_0);

    ParserSyntax::def(t, "False", false_prefix, invalid_suffix, precedence_3);
    ParserSyntax::def(t, "True", true_prefix, invalid_suffix, precedence_3);

    ParserSyntax::op(t, "^", false, true, 100);

    ParserSyntax::op(t, "*", false, true, 90);
    ParserSyntax::op(t, "/", false, true, 90);
    ParserSyntax::op(t, "%", false, true, 90);

    ParserSyntax::op(t, "+", false, true, 80);
    ParserSyntax::op(t, "-", true, true, 80);

    ParserSyntax::op(t, "<<", false, true, 70);
    ParserSyntax::op(t, ">>", false, true, 70);

    ParserSyntax::op(t, "&", false, true, 60);
    ParserSyntax::op(t, "|", false, true, 60);

    ParserSyntax::op(t, "<", false, true, 50);
    ParserSyntax::op(t, "<=", false, true, 50);
    ParserSyntax::op(t, ">", false, true, 50);
    ParserSyntax::op(t, ">=", false, true, 50);
    ParserSyntax::op(t, "==", false, true, 50);
    ParserSyntax::op(t, "!=", false, true, 50);

    ParserSyntax::op(t, "&&", false, true, 40);
    ParserSyntax::op(t, "||", false, true, 30);

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

fn invalid_prefix(parser: &Parser) -> Parse {
    Unwind::error_at(
        parser.source_location(),
        &format!("Not valid in value position: {}", parser.slice()),
    )
}

fn invalid_suffix(parser: &Parser, _: Expr, _: PrecedenceFunction) -> ExprParse {
    Unwind::error_at(
        parser.source_location(),
        &format!("Not valid in operator position: {}", parser.slice()),
    )
}

fn array_prefix(parser: &Parser) -> Parse {
    let mut source_location = parser.source_location();
    let (token, next) = parser.lookahead()?;
    let next_end = next.end;
    let data = if token == Token::SIGIL && parser.slice_at(next) == "]" {
        parser.next_token()?;
        source_location.extend_span_to(next_end);
        vec![]
    } else {
        let mut data = vec![];
        loop {
            data.push(parser.parse_expr(1)?);
            let token = parser.next_token()?;
            if token == Token::SIGIL && parser.slice() == "]" {
                source_location.extend_span_to(parser.span().end);
                break data;
            }
            if token == Token::SIGIL && parser.slice() == "," {
                continue;
            }
            return parser.error("Expected ] or ,");
        }
    };
    Ok(Syntax::Expr(Array::expr(source_location, data)))
}

fn assign_suffix(
    parser: &Parser,
    left: Expr,
    precedence: PrecedenceFunction,
) -> Result<Expr, Unwind> {
    if !left.is_var() {
        return Unwind::error_at(parser.source_location(), "Invalid assignment");
    }
    let right = parser.parse_expr(precedence(parser, parser.span())?)?;
    // We use the name we're assigning to as the span.
    // FIXME: Maybe this is a sign that we should actually store a Var with it's own span
    // in the Assign, then assign could have the span for just the operator?
    Ok(Assign::expr(left.source_location(), left.name(), right))
}

fn cascade_suffix(
    parser: &Parser,
    left: Expr,
    precedence: PrecedenceFunction,
) -> Result<Expr, Unwind> {
    let receiver = left.to_cascade(true);
    Ok(parser.parse_tail(receiver, precedence(parser, parser.span())?)?.to_cascade(false))
}

fn dynamic_var_prefix(parser: &Parser) -> Parse {
    let source_location = parser.source_location();
    let next = parser.next_token()?;
    if let Token::WORD = next {
        Ok(Syntax::Expr(Expr::Var(parse_var(parser, true)?)))
    } else {
        Unwind::error_at(source_location, "Invalid dynamic variable name")
    }
}

fn literal_prefix(parser: &Parser) -> Parse {
    let mut source_location = parser.source_location();
    let mut selector = String::new();
    match parser.lookahead()? {
        (Token::WORD, span) => {
            parser.next_token()?;
            selector.push_str(parser.slice_at(span.clone()));
        }
        (Token::SIGIL, span) => {
            parser.next_token()?;
            selector.push_str(parser.slice_at(span.clone()));
        }
        (Token::KEYWORD, span) => {
            parser.next_token()?;
            selector.push_str(parser.slice_at(span.clone()));
            while let (Token::KEYWORD, span) = parser.lookahead()? {
                parser.next_token()?;
                selector.push_str(parser.slice_at(span.clone()));
            }
        }
        _ => return Unwind::error_at(source_location, "Invalid token after #"),
    }
    source_location.extend_span_to(parser.span().end);
    Ok(Syntax::Expr(Const::expr(source_location, Literal::Selector(selector))))
}

fn eof_prefix(parser: &Parser) -> Parse {
    parser.eof_error("Unexpected EOF in value position")
}

fn eof_suffix(parser: &Parser, _: Expr, _: PrecedenceFunction) -> Result<Expr, Unwind> {
    parser.eof_error("Unexpected EOF in suffix position")
}

fn false_prefix(parser: &Parser) -> Parse {
    Ok(Syntax::Expr(Const::expr(parser.source_location(), Literal::Boolean(false))))
}

fn identifier_precedence(parser: &Parser, span: Span) -> Result<usize, Unwind> {
    match parser.name_table.get(parser.slice_at(span.clone())) {
        Some(syntax) => parser.syntax_precedence(syntax, span),
        None => return Ok(1000), // unary messages
    }
}

fn identifier_prefix(parser: &Parser) -> Parse {
    let name = parser.slice();
    match parser.name_table.get(name) {
        Some(syntax) => parser.parse_prefix_syntax(syntax),
        None => {
            name.chars().next().expect("BUG: empty identifier");
            Ok(Syntax::Expr(Expr::Var(Var::untyped(
                parser.source_location(),
                parser.tokenstring(),
                false,
            ))))
        }
    }
}

fn identifier_suffix(parser: &Parser, left: Expr, _: PrecedenceFunction) -> Result<Expr, Unwind> {
    let name = parser.slice();
    match parser.name_table.get(name) {
        Some(syntax) => parser.parse_suffix_syntax(syntax, left),
        None => {
            // Unary message
            Ok(left.send(Message {
                source_location: parser.source_location(),
                selector: parser.tokenstring(),
                args: vec![],
            }))
        }
    }
}

fn is_suffix(parser: &Parser, left: Expr, pre: PrecedenceFunction) -> Result<Expr, Unwind> {
    let source_location = parser.source_location();
    let right = parser.parse_expr(pre(parser, parser.span())?)?;
    Ok(Eq::expr(source_location, Box::new(left), Box::new(right)))
}

fn keyword_suffix(
    parser: &Parser,
    left: Expr,
    precedence: PrecedenceFunction,
) -> Result<Expr, Unwind> {
    let precedence = precedence(parser, parser.span())?;
    let mut selector = parser.tokenstring();
    let mut args = vec![];
    let mut source_location = parser.source_location();
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
    // FIXME: Potential multiline span is probably going to cause trouble in
    // error reporting...
    //
    // Might also be nice to mark a non-continuous source location that only
    // underlines the selector fragments.
    source_location.extend_span_to(parser.span().end);
    Ok(left.send(Message {
        source_location,
        selector,
        args,
    }))
}

fn operator_precedence(parser: &Parser, span: Span) -> Result<usize, Unwind> {
    let slice = parser.slice_at(span.clone());
    let syntax = parser.name_table.get(slice).unwrap_or(&UNKNOWN_OPERATOR_SYNTAX);
    parser.syntax_precedence(syntax, span)
}

fn operator_prefix(parser: &Parser) -> Parse {
    let syntax = parser.name_table.get(parser.slice()).unwrap_or(&UNKNOWN_OPERATOR_SYNTAX);
    parser.parse_prefix_syntax(syntax)
}

fn operator_suffix(parser: &Parser, left: Expr, _: PrecedenceFunction) -> Result<Expr, Unwind> {
    let syntax = parser.name_table.get(parser.slice()).unwrap_or(&UNKNOWN_OPERATOR_SYNTAX);
    parser.parse_suffix_syntax(syntax, left)
}

fn paren_prefix(parser: &Parser) -> Parse {
    let expr = parser.parse_seq()?;
    let token = parser.next_token()?;
    if token == Token::SIGIL && parser.slice() == ")" {
        Ok(Syntax::Expr(expr))
    } else {
        // FIXME: EOF
        parser.error("Expected )")
    }
}

fn true_prefix(parser: &Parser) -> Parse {
    Ok(Syntax::Expr(Const::expr(parser.source_location(), Literal::Boolean(true))))
}

fn typecheck_suffix(
    parser: &Parser,
    left: Expr,
    _precedence: PrecedenceFunction,
) -> Result<Expr, Unwind> {
    match parser.next_token()? {
        Token::WORD => {
            Ok(Typecheck::expr(parser.source_location(), Box::new(left), parser.tokenstring()))
        }
        _ => parser.error("Invalid type designator"),
    }
}

fn sequence_suffix(
    parser: &Parser,
    left: Expr,
    precedence: PrecedenceFunction,
) -> Result<Expr, Unwind> {
    while parser.at_comment() {
        parser.next_token()?;
    }
    let (token, span) = parser.lookahead()?;
    let text = parser.slice_at(span);
    // FIXME: Pull this information from a table instead.
    if (token == Token::WORD
        && (text == "required"
            || text == "direct"
            || text == "method"
            || text == "end"
            || text == "class"
            || text == "is"))
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

fn parse_record(parser: &Parser) -> Result<Expr, Unwind> {
    let mut source_location = parser.source_location();
    let mut selector = String::new();
    let mut args = Vec::new();
    loop {
        match parser.next_token()? {
            Token::KEYWORD => {
                selector.push_str(parser.slice());
                args.push(parser.parse_expr(1)?);
                match parser.next_token()? {
                    Token::SIGIL if "," == parser.slice() => continue,
                    Token::SIGIL if "}" == parser.slice() => break,
                    _ => return parser.error("Malformed record"),
                }
            }
            Token::SIGIL if "}" == parser.slice() => break,
            _ => return parser.error("Malformed record"),
        }
    }
    source_location.extend_span_to(parser.span().end);
    // This kind of indicates I need a more felicitious representation
    // in order to be able to reliably print back things without converting
    // {x: 42} to Record x: 42 accidentally. (Or I need to not have this syntax).
    Ok(Expr::Var(Var::untyped(source_location.clone(), "Record".to_string(), false)).send(
        Message {
            source_location,
            selector,
            args,
        },
    ))
}

fn parse_dictionary(
    parser: &Parser,
    mut source_location: SourceLocation,
    mut key: Expr,
) -> Result<Expr, Unwind> {
    let mut assoc = Vec::new();
    loop {
        if "->" != parser.slice() {
            return parser.error(&format!("Expected ->, got {}", parser.slice()));
        }
        assoc.push((key, parser.parse_expr(1)?));
        match parser.next_token()? {
            Token::SIGIL if "}" == parser.slice() => {
                break;
            }
            Token::SIGIL if "," == parser.slice() => {
                match parser.lookahead()? {
                    // Handle trailing comma
                    (Token::SIGIL, span) if "}" == parser.slice_at(span.clone()) => {
                        parser.next_token()?;
                        break;
                    }
                    _ => {
                        key = parser.parse_expr(1)?;
                        parser.next_token()?;
                    }
                }
            }
            _ => return parser.error("Expected ',' or '}'"),
        }
    }
    source_location.extend_span_to(parser.span().end);
    Ok(Dictionary::expr(source_location, assoc))
}

fn parse_block_or_dictionary(parser: &Parser) -> Result<Expr, Unwind> {
    let mut source_location = parser.source_location();
    assert_eq!("{", parser.slice());
    //
    // Blocks only (if any of this happens, we're in a block)
    //
    let mut params = vec![];
    let mut rtype = None;
    let (token, span) = parser.lookahead()?;
    if token == Token::SIGIL && parser.slice_at(span) == "|" {
        parser.next_token()?;
        loop {
            let token = parser.next_token()?;
            if token == Token::WORD {
                params.push(parse_var(parser, false)?);
                continue;
            }
            if token == Token::SIGIL && parser.slice() == "|" {
                break;
            }
            return parser.error("Not valid as block parameter");
        }
    }
    let (token, span) = parser.lookahead()?;
    if token == Token::SIGIL && parser.slice_at(span) == "->" {
        parser.next_token()?;
        rtype = Some(parse_type_designator(parser)?);
    }
    //
    // Moment of truth!
    //
    let (token, span) = parser.lookahead()?;
    let body = if token == Token::SIGIL && parser.slice_at(span.clone()) == "}" {
        source_location.extend_span_to(span.end);
        Const::expr(source_location.clone(), Literal::Boolean(false))
    } else {
        let expr = parser.parse_seq()?;
        let (token, span) = parser.lookahead()?;
        if token == Token::SIGIL && "->" == parser.slice_at(span) {
            if !params.is_empty() || rtype.is_some() {
                return parser.error("Neither block nor dictionary");
            }
            parser.next_token()?;
            return parse_dictionary(parser, source_location, expr);
        } else {
            expr
        }
    };
    //
    // If we're still here we're in a block
    //
    let end = parser.next_token()?;
    source_location.extend_span_to(parser.span().end);
    // FIXME: hardcoded {
    // Would be nice to be able to swap between [] and {} and
    // keep this function same,
    if end == Token::SIGIL && parser.slice() == "}" {
        Ok(Block::expr(source_location, params, Box::new(body), rtype))
    } else if end == Token::EOF {
        parser.eof_error("Unexpected EOF while pasing a block: expected } as block terminator")
    } else {
        // println!("{}", &parser.source[source_location.get_span()]);
        parser.error("Expected } as block terminator")
    }
}

fn block_prefix(parser: &Parser) -> Parse {
    //
    // { keyword: ... } --> Record
    //
    // Otherwise either Block or Dictionary. Easier to diverge
    // later than figure out up front.
    //
    let res = match parser.lookahead() {
        Ok((Token::KEYWORD, _)) => parse_record(parser)?,
        _ => parse_block_or_dictionary(parser)?,
    };
    Ok(Syntax::Expr(res))
}

fn import_prefix(parser: &Parser) -> Parse {
    let mut source_location = parser.source_location();
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
    // println!("import: {}", &spec);
    if spec.len() > 0 {
        // Deal with .*
        if let Some(star) = parser.dotted_name_at(parser.span().end, true)? {
            spec.push_str(parser.slice_at(star));
        }
        source_location.extend_span_to(parser.span().end);
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
                        source_location.clone(),
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
        Ok(Syntax::Def(Def::ImportDef(ImportDef {
            source_location,
            path,
            prefix,
            name,
        })))
    } else {
        source_location.set_span(&name_span);
        return Unwind::error_at(source_location, "Expected module name");
    }
}

fn interface_prefix(parser: &Parser) -> Parse {
    // FIXME: span is the span of the interface, but maybe it would be better if these
    // had all their own spans.
    //
    // FIXME: duplicated extend_prefix pretty much.
    let source_location = parser.source_location();
    let interface_name = match parser.next_token()? {
        Token::WORD => {
            if parser.slice().chars().next().expect("BUG: empty identifier").is_uppercase() {
                parser.slice()
            } else {
                // FIXME: Not all languages use capital letters
                return parser.error("Interface names must start with an uppercase letter");
            }
        }
        _ => return parser.error("Expected interface name"),
    };
    // println!("interface: {}", interface_name);
    let mut interface = InterfaceDef::new(source_location, interface_name);
    loop {
        let next = parser.next_token()?;
        if next == Token::COMMENT || next == Token::BLOCK_COMMENT {
            continue;
        }
        if next == Token::WORD && parser.slice() == "end" {
            break;
        }

        if next == Token::EOF {
            return parser
                .eof_error("Unexpected EOF while parsing interface: expected method or end");
        }
        if next == Token::WORD && parser.slice() == "direct" {
            if parser.next_token()? == Token::WORD && parser.slice() == "method" {
                interface.add_method(MethodKind::Class, parse_method(parser)?);
                continue;
            } else {
                return parser.error("Expected 'method'");
            }
        }
        if next == Token::WORD && parser.slice() == "method" {
            interface.add_method(MethodKind::Instance, parse_method(parser)?);
            continue;
        }
        if next == Token::WORD && parser.slice() == "required" {
            parser.next_token()?;
            interface.add_method(MethodKind::Required, parse_method_signature(parser)?);
            continue;
        }
        if next == Token::WORD && parser.slice() == "is" {
            if let Token::WORD = parser.next_token()? {
                interface.add_interface(parser.slice());
                continue;
            }
            return parser.error("Invalid inherited interface name in interface");
        }
        return parser.error("Expected method or end");
    }
    Ok(Syntax::Def(Def::InterfaceDef(interface)))
}

fn class_prefix(parser: &Parser) -> Parse {
    // FIXME: span is the span of the class, but maybe it would be better if these
    // had all their own spans.
    let source_location = parser.source_location();
    let class_name = loop {
        match parser.next_token()? {
            Token::COMMENT => continue,
            Token::BLOCK_COMMENT => continue,
            Token::WORD => {
                let next = parser.slice().chars().next().expect("BUG: empty identifier");
                if next.is_uppercase() || next == '_' {
                    break parser.tokenstring();
                } else {
                    // FIXME: Not all languages use capital letters
                    return parser
                        .error("Class names must start with an uppercase letter or underscore");
                }
            }
            _ => return parser.error("Expected class name"),
        }
    };
    // println!("class: {}", class_name);
    loop {
        match parser.next_token()? {
            Token::COMMENT => continue,
            Token::BLOCK_COMMENT => continue,
            Token::SIGIL if parser.slice() == "{" => break,
            _ => return parser.error("Expected { to open instance variable block"),
        }
    }
    let mut instance_variables = Vec::new();
    loop {
        let token = parser.next_token()?;
        match token {
            Token::COMMENT => continue,
            Token::BLOCK_COMMENT => continue,
            Token::WORD => {
                instance_variables.push(parse_var(parser, false)?);
            }
            Token::SIGIL if parser.slice() == "}" => {
                break;
            }
            _ => return parser.error("Invalid instance variable specification"),
        }
    }
    let size = instance_variables.len();
    let mut class = ClassDef::new(source_location, class_name, instance_variables);
    loop {
        let next = parser.next_token()?;
        if next == Token::COMMENT || next == Token::BLOCK_COMMENT {
            continue;
        }
        if next == Token::EOF {
            return parser.eof_error("Unexpected EOF while parsing class: expected method or end");
        }
        if next == Token::WORD && parser.slice() == "end" {
            break;
        }
        if next == Token::WORD && parser.slice() == "direct" {
            if parser.next_token()? == Token::WORD && parser.slice() == "method" {
                class.add_method(MethodKind::Class, parse_method(parser)?);
                continue;
            } else {
                return parser.error("Expected 'method'");
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
        if next == Token::WORD && parser.slice() == "is" {
            if let Token::WORD = parser.next_token()? {
                class.add_interface(parser.slice());
                continue;
            }
            return parser.error("Invalid interface name in class");
        }
        return parser
            .error(&format!("Expected method specification or end, got: '{}'", parser.slice()));
    }
    Ok(Syntax::Def(Def::ClassDef(class)))
}

fn define_prefix(parser: &Parser) -> Parse {
    let mut name = String::new();
    let mut next = parser.next_token()?;
    if Token::SIGIL == next && "$" == parser.slice() {
        name.push_str("$");
        next = parser.next_token()?;
    }
    if Token::WORD != next {
        return parser.error("Expected name after 'define'");
    }
    name.push_str(parser.slice());

    let source_location = parser.source_location();
    let init = parser.parse_seq()?;

    parser.next_token()?;
    if "!" != parser.slice() {
        return parser.error(&format!("Expected '!' after definition, got: '{}'", parser.slice()));
    }

    Ok(Syntax::Def(Def::DefineDef(DefineDef {
        source_location,
        name,
        init,
    })))
}

fn extend_prefix(parser: &Parser) -> Parse {
    // FIXME: span is the span of the extension, but maybe it would be better if
    // these had all their own spans.
    //
    // FIXME: duplicated class_prefix pretty much.
    let source_location = parser.source_location();
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
    let mut class = ExtensionDef::new(source_location, &class_name);
    // println!("extend: {}", &class_name);
    loop {
        let next = parser.next_token()?;
        if next == Token::COMMENT || next == Token::BLOCK_COMMENT {
            continue;
        }
        if next == Token::WORD && parser.slice() == "end" {
            break;
        }
        if next == Token::EOF {
            return parser
                .eof_error("Unexpected EOF while parsing extension: expected method or end");
        }
        if next == Token::WORD && parser.slice() == "direct" {
            if parser.next_token()? == Token::WORD && parser.slice() == "method" {
                class.add_method(MethodKind::Class, parse_method(parser)?);
                continue;
            } else {
                return parser.error("Expected 'method'");
            }
        }
        if next == Token::WORD && parser.slice() == "method" {
            class.add_method(MethodKind::Instance, parse_method(parser)?);
            continue;
        }
        if next == Token::WORD && parser.slice() == "is" {
            if let Token::WORD = parser.next_token()? {
                class.add_interface(parser.slice());
                continue;
            }
            return parser.error("Invalid interface name in extend");
        }
        return parser.error(&format!("Expected method or end, got: '{}'", parser.slice()));
    }
    Ok(Syntax::Def(Def::ExtensionDef(class)))
}

fn let_prefix(parser: &Parser) -> Parse {
    let mut next = parser.next_token()?;
    let mut source_location = parser.source_location();

    let dynamic = if Token::SIGIL == next && "$" == parser.slice() {
        next = parser.next_token()?;
        true
    } else {
        false
    };

    if Token::WORD != next {
        return parser.error("Expected variable name after let");
    }

    source_location.extend_span_to(parser.span().end);

    let Var {
        name,
        typename,
        ..
    } = parse_var(parser, dynamic)?;

    if !(parser.next_token()? == Token::SIGIL && parser.slice() == "=") {
        return parser.error("Expected = in let");
    }

    let value = parser.parse_single()?;
    let mut eof = match parser.next_token()? {
        Token::SIGIL if parser.slice() == "." => false,
        Token::EOF => true,
        _ => {
            return parser.error("Expected '.' after let");
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
    Ok(Syntax::Expr(Bind::expr(source_location, name, typename, Box::new(value), body, dynamic)))
}

fn ignore_prefix(parser: &Parser) -> Parse {
    parser._parse()
}

fn ignore_suffix(_parser: &Parser, left: Expr, _pre: PrecedenceFunction) -> Result<Expr, Unwind> {
    Ok(left)
}

fn number_prefix(parser: &Parser) -> Parse {
    let slice = parser.slice();
    // Hexadecimal case
    if slice.len() > 2 && ("0x" == &slice[0..2] || "0X" == &slice[0..2]) {
        let integer = match i64::from_str_radix(&slice[2..], 16) {
            Ok(i) => i,
            Err(_) => return parser.error("Malformed hexadecimal number"),
        };
        return Ok(Syntax::Expr(Const::expr(parser.source_location(), Literal::Integer(integer))));
    }
    // Binary case
    if slice.len() > 2 && ("0b" == &slice[0..2] || "0B" == &slice[0..2]) {
        let integer = match i64::from_str_radix(&slice[2..], 2) {
            Ok(i) => i,
            Err(_) => return parser.error("Malformed binary number"),
        };
        return Ok(Syntax::Expr(Const::expr(parser.source_location(), Literal::Integer(integer))));
    }
    // Decimal and float case
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
                    Ok(f) => {
                        return Ok(Syntax::Expr(Const::expr(
                            parser.source_location(),
                            Literal::Float(f),
                        )))
                    }
                    Err(_) => return parser.error("Malformed number"),
                }
            }
        }
    }
    Ok(Syntax::Expr(Const::expr(parser.source_location(), Literal::Integer(decimal))))
}

fn return_prefix(parser: &Parser) -> Parse {
    // FIXME: what about "return x. dead-expr" ?
    Ok(Syntax::Expr(Return::expr(parser.source_location(), parser.parse_single()?)))
}

fn panic_prefix(parser: &Parser) -> Parse {
    // FIXME: what about "panic x. dead-expr" ?
    Ok(Syntax::Expr(Panic::expr(parser.source_location(), parser.parse_single()?)))
}

/// Takes care of \n, and such. Terminates on { or end of string.
fn scan_string_part(parser: &Parser, mut source_location: SourceLocation) -> Result<Expr, Unwind> {
    // println!("scan: '{}'", parser.slice_at(source_location.get_span()));
    let span = source_location.get_span();
    let start = span.start;
    let mut chars = parser.slice_at(span).char_indices();
    let mut res = String::new();
    loop {
        match chars.next() {
            None => return Ok(Const::expr(source_location, Literal::String(res))),
            Some((pos0, '\\')) => match chars.next() {
                None => {
                    source_location.set_span(&(start + pos0..start + pos0 + 1));
                    return Unwind::error_at(source_location, "Literal string ends on escape.");
                }
                Some((_, '"')) => res.push_str("\""),
                Some((_, '\\')) => res.push_str("\\"),
                Some((_, 'n')) => res.push_str("\n"),
                Some((_, 't')) => res.push_str("\t"),
                Some((_, 'r')) => res.push_str("\r"),
                Some((_, '{')) => res.push_str("{"),
                Some((pos1, _)) => {
                    source_location.set_span(&(start + pos0..start + pos1));
                    return Unwind::error_at(
                        source_location,
                        "Unknown escape sequence in literal string.",
                    );
                }
            },
            Some((pos, '{')) => {
                source_location.set_span(&(start..start + pos));
                return Ok(Const::expr(source_location, Literal::String(res)));
            }
            Some((_, c)) => res.push(c),
        }
    }
}

fn string_prefix(parser: &Parser) -> Parse {
    // FIXME: This should generate either a literal string,
    // or a StringInterpolation, so that pretty printing can work.
    let slice = parser.slice();
    let mut source_location = parser.source_location();
    let mut span = source_location.get_span();

    // Strip quotes from ends of span
    let mut n = 0;
    while n < slice.len() - n && &slice[n..n + 1] == "\"" {
        n += 1;
    }
    span = (span.start + n)..(span.end - n);

    // Collect literal and interpolated parts
    let mut parts = Vec::new();

    loop {
        source_location.set_span(&span);
        // println!("scan: {:?}", &span);
        let literal = scan_string_part(parser, source_location.clone())?;
        // println!("literal: {:?}", parser.slice_at(literal.span()));
        span = literal.span().end..span.end;
        parts.push(literal);
        source_location.set_span(&span);
        if span.start < span.end {
            let (interp, end) = parser.parse_interpolated_block(source_location.clone())?;
            parts.push(interp);
            span = end..span.end;
        } else {
            break;
        }
    }

    // Extend first and last part spans to cover the quotes, leaving
    // the parts reversed for what comes next.
    parts[0].extend_span(-(n as isize));
    parts.reverse();
    parts[0].extend_span(n as isize);

    // Append them all togather.
    let mut expr = match parts.pop() {
        None => {
            return Ok(Syntax::Expr(Const::expr(source_location, Literal::String("".to_string()))));
        }
        Some(part) => part,
    };
    while let Some(right) = parts.pop() {
        let rsource = right.source_location();
        expr = expr.send(Message {
            source_location: rsource.clone(),
            selector: "append:".to_string(),
            args: vec![right.send(Message {
                source_location: rsource,
                selector: "toString".to_string(),
                args: vec![],
            })],
        })
    }

    Ok(Syntax::Expr(expr))
}

/// Utils

fn parse_type_designator(parser: &Parser) -> Result<String, Unwind> {
    if let Token::WORD = parser.next_token()? {
        Ok(parser.tokenstring())
    } else {
        parser.error("Invalid type designator")
    }
}

fn parse_var(parser: &Parser, dynamic: bool) -> Result<Var, Unwind> {
    let mut loc = parser.source_location();
    let mut name = String::new();
    if dynamic {
        let span = parser.span();
        loc.set_span(&(span.start - 1..span.end));
        name.push_str("$");
    };
    name.push_str(parser.slice());
    let (token, span) = parser.lookahead()?;
    let var = if token == Token::SIGIL && parser.slice_at(span) == "::" {
        parser.next_token()?;
        Var::typed(loc, name, parse_type_designator(parser)?, dynamic)
    } else {
        Var::untyped(loc, name, dynamic)
    };
    Ok(var)
}

fn parse_method(parser: &Parser) -> Result<MethodDefinition, Unwind> {
    let mut method = parse_method_signature(parser)?;
    // println!("- method: {}", method.selector);
    // NOTE: This is the place where I could inform parser about instance
    // variables.
    // FIXME: Would be nice to add "while parsing method Bar#foo"
    // type info to the error.
    method.body = Some(Box::new(parser.parse_seq()?));
    if parser.at_eof() {
        return parser.eof_error("End of input inside method");
    }
    parser.next_token()?;
    if parser.slice() != "!" {
        return Unwind::error_at(
            method.body.unwrap().source_location(),
            &format!("Expected '!' after method {}, got: '{}'", method.selector, parser.slice()),
        );
    }
    Ok(method)
}

fn parse_method_signature(parser: &Parser) -> Result<MethodDefinition, Unwind> {
    assert_eq!(parser.slice(), "method");
    let source_location = parser.source_location();
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
                    parameters.push(parse_var(parser, false)?);
                } else {
                    return parser.error("Expected binary selector parameter");
                }
                break;
            }
            Token::KEYWORD => {
                if let Token::WORD = parser.next_token()? {
                    parameters.push(parse_var(parser, false)?);
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
    // FIXME: Would be nice to have a --verbose-parser which would print
    // things like this
    Ok(MethodDefinition::new(source_location, selector, parameters, rtype))
}

/// Tests and tools

#[cfg(test)]
pub mod utils {

    use crate::parse::*;

    pub(crate) fn block(span: Span, params: Vec<&str>, body: Expr) -> Expr {
        let mut p = span.start + 3;
        let mut blockparams = vec![];
        for param in params {
            let start = p;
            let end = start + param.len();
            p = end + 2;
            blockparams.push(Var::untyped(
                SourceLocation::span(&(start..end)),
                param.to_string(),
                false,
            ))
        }
        Block::expr(SourceLocation::span(&span), blockparams, Box::new(body), None)
    }

    pub(crate) fn block_typed(span: Span, params: Vec<(&str, &str)>, body: Expr) -> Expr {
        let mut p = span.start + 3;
        let mut blockparams = vec![];
        for param in params {
            let start = p;
            let end = start + param.0.len();
            p = end + 4 + param.1.len();
            blockparams.push(Var::typed(
                SourceLocation::span(&(start..end)),
                param.0.to_string(),
                param.1.to_string(),
                false,
            ));
        }
        Block::expr(SourceLocation::span(&span), blockparams, Box::new(body), None)
    }

    pub(crate) fn binary(span: Span, name: &str, left: Expr, right: Expr) -> Expr {
        left.send(Message {
            source_location: SourceLocation::span(&span),
            selector: name.to_string(),
            args: vec![right],
        })
    }

    pub(crate) fn bind(
        source_location: SourceLocation,
        name: &str,
        value: Expr,
        body: Expr,
    ) -> Expr {
        Bind::expr(
            source_location,
            name.to_string(),
            None,
            Box::new(value),
            Some(Box::new(body)),
            false,
        )
    }

    pub(crate) fn bind_typed(
        source_location: SourceLocation,
        name: &str,
        typename: &str,
        value: Expr,
        body: Expr,
    ) -> Expr {
        Bind::expr(
            source_location,
            name.to_string(),
            Some(typename.to_string()),
            Box::new(value),
            Some(Box::new(body)),
            false,
        )
    }

    pub(crate) fn class(span: Span, name: &str, instance_variables: Vec<&str>) -> Def {
        let mut p = span.start + "class ".len() + name.len() + " { ".len();
        let mut vars = Vec::new();
        for v in instance_variables {
            vars.push(Var::untyped(SourceLocation::span(&(p..p + v.len())), v.to_string(), false));
            p += v.len() + " ".len()
        }
        ClassDef::syntax(span, name.to_string(), vars)
    }

    pub(crate) fn float(span: Span, value: f64) -> Expr {
        Const::expr(SourceLocation::span(&span), Literal::Float(value))
    }

    pub(crate) fn int(span: Span, value: i64) -> Expr {
        Const::expr(SourceLocation::span(&span), Literal::Integer(value))
    }

    pub(crate) fn keyword(span: Span, name: &str, left: Expr, args: Vec<Expr>) -> Expr {
        left.send(Message {
            source_location: SourceLocation::span(&span),
            selector: name.to_string(),
            args,
        })
    }

    pub(crate) fn method(
        span: Span,
        selector: &str,
        parameters: Vec<&str>,
        body: Expr,
    ) -> MethodDefinition {
        let mut method = method_signature(SourceLocation::span(&span), selector, parameters);
        method.body = Some(Box::new(body));
        method
    }

    pub(crate) fn method_signature(
        source_location: SourceLocation,
        selector: &str,
        parameters: Vec<&str>,
    ) -> MethodDefinition {
        MethodDefinition::new(
            source_location,
            selector.to_string(),
            // FIXME: span
            parameters
                .iter()
                .map(|name| Var::untyped(SourceLocation::span(&(0..0)), name.to_string(), false))
                .collect(),
            None,
        )
    }

    pub(crate) fn seq(exprs: Vec<Expr>) -> Expr {
        Seq::expr(exprs)
    }

    pub(crate) fn string(span: Span, value: &str) -> Expr {
        Const::expr(SourceLocation::span(&span), Literal::String(value.to_string()))
    }

    pub(crate) fn typecheck(source_location: SourceLocation, expr: Expr, typename: &str) -> Expr {
        Typecheck::expr(source_location, Box::new(expr), typename.to_string())
    }

    pub(crate) fn unary(span: Span, name: &str, left: Expr) -> Expr {
        left.send(Message {
            source_location: SourceLocation::span(&span),
            selector: name.to_string(),
            args: vec![],
        })
    }

    pub(crate) fn var(span: Span, name: &str) -> Expr {
        Expr::Var(Var::untyped(SourceLocation::span(&span), name.to_string(), false))
    }
}

#[test]
fn test_tokenstring_after_lookahead() {
    let parser = Parser::new("foo bar", "dummy");
    parser.next_token().unwrap();
    parser.lookahead().unwrap();
    assert_eq!("foo", &parser.tokenstring());
}

#[test]
fn test_tokenstring_after_lookahead2() {
    let parser = Parser::new("foo bar", "dummy");
    parser.next_token().unwrap();
    parser.lookahead2().unwrap();
    assert_eq!("foo", &parser.tokenstring());
}

#[test]
fn test_parser_error_after_lookahead() {
    let parser = Parser::new("foo bar", "dummy");
    parser.next_token().unwrap();
    parser.lookahead().unwrap();
    let err: Result<(), Unwind> = Unwind::error_at(SourceLocation::span(&(0..3)), "oops");
    assert_eq!(err, parser.error("oops"));
}

#[test]
fn test_parser_error_after_lookahead2() {
    let parser = Parser::new("foo bar", "dummy");
    parser.next_token().unwrap();
    parser.lookahead2().unwrap();
    let err: Result<(), Unwind> = Unwind::error_at(SourceLocation::span(&(0..3)), "oops");
    assert_eq!(err, parser.error("oops"));
}
