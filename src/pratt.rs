use regex::Regex;

use std::borrow::ToOwned;
use std::collections::VecDeque;
#[derive(Debug, PartialEq)]
struct ParseError {
    position: usize,
    problem: &'static str,
}

#[derive(Debug, PartialEq, Clone)]
enum Literal {
    Decimal(i64),
    Float(f64),
}

#[derive(Debug, PartialEq, Clone)]
struct Message {
    selector: String,
    arguments: Vec<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
enum Expr {
    Constant(Literal),
    Variable(String),
    // Explicit receiver for chains means that we can always trivially access
    // the original receiver, instead of having to look for it in a tree of sends.
    Chain(Box<Expr>, Vec<Message>),
    // Each vector of messages is a separate chain using the original receiver.
    Cascade(Box<Expr>, Vec<Vec<Message>>),
    Sequence(Vec<Expr>),
    Block(Vec<String>, Box<Expr>),
}

impl Expr {
    fn unary(mut self, selector: String) -> Self {
        match &mut self {
            Expr::Chain(_, ref mut msgs) => {
                msgs.push(Message {
                    selector,
                    arguments: vec![],
                });
                self
            }
            _ => Expr::send(self, selector, vec![]),
        }
    }
    fn binary(mut self, selector: String, arg: Expr) -> Self {
        match &mut self {
            Expr::Chain(_, ref mut msgs) => {
                msgs.push(Message {
                    selector,
                    arguments: vec![arg],
                });
                self
            }
            _ => Expr::send(self, selector, vec![arg]),
        }
    }
    fn keyword(mut self, selector: String, arguments: Vec<Expr>) -> Self {
        match &mut self {
            Expr::Chain(_, ref mut msgs) => {
                msgs.push(Message {
                    selector,
                    arguments,
                });
                self
            }
            _ => Expr::send(self, selector, arguments),
        }
    }
    fn send(object: Expr, selector: String, arguments: Vec<Expr>) -> Expr {
        Expr::Chain(
            Box::new(object),
            vec![Message {
                selector,
                arguments,
            }],
        )
    }
}

#[derive(Debug, PartialEq, Clone)]
struct Token {
    position: usize,
    info: TokenInfo,
}

#[derive(Debug, PartialEq, Clone)]
enum TokenInfo {
    Eof(),
    Constant(Literal),
    Identifier(String),
    Keyword(String),
    Operator(String),
    Chain(),
    Cascade(),
    // true = comma, false = newline
    Sequence(bool),
    BlockBegin(),
    BlockEnd(),
    PositionalParameter(String),
}

impl Token {
    fn operator(&self) -> &str {
        match &self.info {
            TokenInfo::Operator(name) => name.as_str(),
            _ => panic!("Not an operator: {:?}", self),
        }
    }
    fn error(self, problem: &'static str) -> Result<Expr, ParseError> {
        Err(ParseError {
            position: self.position,
            problem,
        })
    }
    fn precedence(&self) -> usize {
        match &self.info {
            TokenInfo::Eof() => 0,
            TokenInfo::BlockBegin() => 0,
            TokenInfo::BlockEnd() => 0,
            TokenInfo::Sequence(_) => 1,
            TokenInfo::Cascade() => 2,
            TokenInfo::Chain() => 3,
            TokenInfo::Keyword(_) => 4,
            // 10 is fallback for unknown operators
            TokenInfo::Operator(op) => match op.as_str() {
                "=" => 20,
                "==" => 20,
                "<" => 20,
                ">" => 20,
                "<=" => 20,
                ">=" => 20,
                "+" => 30,
                "-" => 30,
                "*" => 40,
                "/" => 40,
                _ => 10,
            },
            // All unary operators
            _ => 100,
        }
    }
}

fn parse_string(string: String) -> Result<Expr, ParseError> {
    Parser::new(Box::new(StringStream::new(string))).parse()
}

fn parse_str(str: &str) -> Result<Expr, ParseError> {
    parse_string(str.to_string())
}

struct Parser {
    stream: Box<Stream>,
    lookahead: VecDeque<Token>,
    cascade: Option<Expr>,
    positional_parameter_re: Regex,
    block_begin_re: Regex,
    block_end_re: Regex,
    cont_re: Regex,
    sequence_re: Regex,
    chain_re: Regex,
    cascade_re: Regex,
    decimal_re: Regex,
    float_re: Regex,
    operator_re: Regex,
    keyword_re: Regex,
    identifier_re: Regex,
}

impl Parser {
    fn new(stream: Box<Stream>) -> Self {
        Parser {
            stream,
            lookahead: VecDeque::new(),
            cascade: None,
            positional_parameter_re: Regex::new(r"^:[_a-zA-Z][_a-zA-z0-9]*").unwrap(),
            block_begin_re: Regex::new(r"^\{").unwrap(),
            block_end_re: Regex::new(r"^\}").unwrap(),
            cont_re: Regex::new(r"^\\").unwrap(),
            sequence_re: Regex::new(r"^,").unwrap(),
            chain_re: Regex::new(r"^--").unwrap(),
            cascade_re: Regex::new(r"^;").unwrap(),
            decimal_re: Regex::new(r"^[0-9]+").unwrap(),
            float_re: Regex::new(r"^[0-9]+\.[0-9]+").unwrap(),
            operator_re: Regex::new(r"^[\-+*/%<>=^|&!\?]+").unwrap(),
            keyword_re: Regex::new(r"^[_a-zA-Z][_a-zA-Z0-9]*:").unwrap(),
            identifier_re: Regex::new(r"^[_a-zA-Z][_a-zA-Z0-9]*").unwrap(),
        }
    }
    fn parse(&mut self) -> Result<Expr, ParseError> {
        let res = self.parse_expression(0);
        // println!("parse() => {:?}", &res);
        res
    }
    fn parse_expression(&mut self, precedence: usize) -> Result<Expr, ParseError> {
        let mut expr = self.parse_prefix()?;
        //println!("parse_prefix() => {:?}", &expr);
        while precedence < self.next_precedence()? {
            expr = self.parse_suffix(expr)?;
            // println!("  parse_suffix() => {:?}", &expr);
        }
        Ok(expr)
    }
    fn parse_prefix(&mut self) -> Result<Expr, ParseError> {
        let token = self.consume_token()?;
        match token.info {
            TokenInfo::Constant(literal) => Ok(Expr::Constant(literal)),
            TokenInfo::Identifier(name) => Ok(Expr::Variable(name)),
            TokenInfo::Cascade() => self.parse_prefix_cascade(token),
            TokenInfo::BlockBegin() => self.parse_prefix_block(token),
            TokenInfo::Operator(_) => self.parse_prefix_operator(token),
            // Leading newline, ignore.
            TokenInfo::Sequence(false) => self.parse_prefix(),
            TokenInfo::Eof() => token.error("Unexpected end of input"),
            _ => token.error("Not a value expression"),
        }
    }
    fn parse_suffix(&mut self, left: Expr) -> Result<Expr, ParseError> {
        let token = self.consume_token()?;
        let precedence = token.precedence();
        match token.info {
            TokenInfo::Identifier(name) => Ok(left.unary(name)),
            TokenInfo::Chain() => Ok(left),
            TokenInfo::Sequence(_) => self.parse_suffix_sequence(left, token),
            TokenInfo::Cascade() => self.parse_suffix_cascade(left, token),
            TokenInfo::Operator(_) => self.parse_suffix_operator(left, token),
            // FIXME: refactor into a separate function.
            TokenInfo::Keyword(name) => {
                let mut args = vec![];
                let mut selector = name;
                loop {
                    args.push(self.parse_expression(precedence)?);
                    if let Token {
                        position: _,
                        info: TokenInfo::Keyword(next),
                    } = self.peek_token()?
                    {
                        self.consume_token();
                        selector.push_str(next.as_str());
                    } else {
                        break;
                    }
                }
                Ok(left.keyword(selector, args))
            }
            TokenInfo::Eof() => token.error("Unexpected end of input"),
            _ => token.error("Not valid in suffix position."),
        }
    }
    fn parse_prefix_block(&mut self, token: Token) -> Result<Expr, ParseError> {
        let mut args = vec![];
        loop {
            let next = self.peek_token()?;
            match next.info {
                TokenInfo::PositionalParameter(name) => {
                    args.push(name);
                    self.consume_token();
                    continue;
                }
                TokenInfo::Operator(ref op) if "|" == op.as_str() => {
                    self.consume_token();
                    break;
                }
                _ => {}
            }
            if args.is_empty() {
                break;
            }
            println!("Bad block. Args={:?}, next={:?}", &args, &next);
            return Err(ParseError {
                position: next.position,
                problem: "Malformed block argument",
            });
        }
        let expr = self.parse_expression(token.precedence())?;
        if let TokenInfo::BlockEnd() = self.consume_token()?.info {
            Ok(Expr::Block(args, Box::new(expr)))
        } else {
            Err(ParseError {
                position: token.position,
                problem: "Block not closed",
            })
        }
    }
    fn parse_prefix_cascade(&mut self, token: Token) -> Result<Expr, ParseError> {
        match &self.cascade {
            None => Err(ParseError {
                position: token.position,
                problem: "Malformed cascade",
            }),
            Some(expr) => {
                let receiver = expr.to_owned();
                self.cascade = None;
                Ok(receiver)
            }
        }
    }
    fn parse_prefix_operator(&mut self, token: Token) -> Result<Expr, ParseError> {
        let message = match token.operator() {
            "-" => "neg",
            _ => return token.error("Not a prefix operator"),
        };
        let expr = self.parse_expression(token.precedence())?;
        Ok(expr.unary(message.to_string()))
    }
    fn parse_suffix_cascade(&mut self, left: Expr, token: Token) -> Result<Expr, ParseError> {
        assert_eq!(self.cascade, None);
        let mut chains = vec![];
        self.insert_token(token.clone());
        loop {
            let next = self.peek_token()?;
            if TokenInfo::Cascade() != next.info {
                break;
            }
            let position = next.position;
            // Need to wrap in cascade in case this is a chain,
            // which would then accumulate the cascaded messages.
            let mark = Expr::Variable("(cascade)".to_string());
            self.cascade = Some(mark.clone());
            let chain = self.parse_expression(token.precedence())?;
            match chain {
                Expr::Chain(to, messages) => {
                    assert_eq!(mark, *to);
                    chains.push(messages);
                }
                _ => {
                    println!("Not a chain: {:?}", chain);
                    return Err(ParseError {
                        position,
                        problem: "Invalid cascade",
                    });
                }
            }
        }
        Ok(Expr::Cascade(Box::new(left.clone()), chains))
    }
    fn parse_suffix_operator(&mut self, left: Expr, token: Token) -> Result<Expr, ParseError> {
        let right = self.parse_expression(token.precedence())?;
        Ok(left.binary(token.operator().to_string(), right))
    }
    fn parse_suffix_sequence(&mut self, left: Expr, token: Token) -> Result<Expr, ParseError> {
        let right = self.parse_expression(token.precedence())?;
        let mut expressions = vec![left];
        if let Expr::Sequence(mut right_expressions) = right {
            expressions.append(&mut right_expressions);
        } else {
            expressions.push(right);
        }
        Ok(Expr::Sequence(expressions))
    }
    fn next_precedence(&mut self) -> Result<usize, ParseError> {
        Ok(self.peek_token()?.precedence())
    }
    fn consume_token(&mut self) -> Result<Token, ParseError> {
        match self.lookahead.pop_front() {
            Some(token) => Ok(token),
            None => self.parse_token(),
        }
    }
    fn insert_token(&mut self, token: Token) {
        self.lookahead.push_front(token)
    }
    // FIXME: Can I get this to return a ref instead? Or should
    // I turn this into a with_peek?
    fn peek_token(&mut self) -> Result<Token, ParseError> {
        match self.lookahead.front() {
            Some(token) => Ok(token.to_owned()),
            None => {
                let token = self.parse_token()?;
                self.lookahead.push_back(token.clone());
                Ok(token)
            }
        }
    }
    fn parse_token(&mut self) -> Result<Token, ParseError> {
        use TokenInfo::*;
        let mut position = 0;
        let mut newline = false;
        while self.stream.at_whitespace() {
            if !newline && self.stream.at_eol() {
                position = self.stream.position();
                newline = true;
            }
            self.stream.skip();
        }
        if let Some(_) = self.stream.scan(&self.cont_re) {
            let position = self.stream.position();
            let next = self.parse_token()?;
            if let Sequence(false) = next.info {
                return self.parse_token();
            } else {
                return Err(ParseError {
                    position,
                    problem: "End-of-line escape not at end of line",
                });
            }
        }
        if newline {
            // Lookahead of one token to decide if newline is a separator
            // or not. Sometimes this inserts a break where one does not below,
            // but that causes the soft separator to appear in parse_prefix
            // where we just skip it.
            let mark = self.stream.position();
            let next = self.parse_token()?;
            match &next.info {
                Constant(_) => {}
                Identifier(_) => {}
                Operator(_) => {}
                _ => return Ok(next),
            }
            self.stream.rewind(mark);
            return Ok(Token {
                position,
                info: Sequence(false),
            });
        }
        position = self.stream.position();
        if self.stream.at_eof() {
            return Ok(Token {
                position,
                info: Eof(),
            });
        }
        if let Some(_) = self.stream.scan(&self.block_begin_re) {
            return Ok(Token {
                position,
                info: BlockBegin(),
            });
        }
        if let Some(_) = self.stream.scan(&self.block_end_re) {
            return Ok(Token {
                position,
                info: BlockEnd(),
            });
        }
        if let Some(_) = self.stream.scan(&self.sequence_re) {
            return Ok(Token {
                position,
                info: Sequence(true),
            });
        }
        // Must be before decimal.
        if let Some(string) = self.stream.scan(&self.float_re) {
            return Ok(Token {
                position,
                info: Constant(Literal::Float(string.parse().unwrap())),
            });
        }
        if let Some(string) = self.stream.scan(&self.decimal_re) {
            return Ok(Token {
                position,
                info: Constant(Literal::Decimal(string.parse().unwrap())),
            });
        }
        if let Some(_) = self.stream.scan(&self.chain_re) {
            return Ok(Token {
                position,
                info: Chain(),
            });
        }
        if let Some(_) = self.stream.scan(&self.cascade_re) {
            return Ok(Token {
                position,
                info: Cascade(),
            });
        }
        if let Some(string) = self.stream.scan(&self.positional_parameter_re) {
            return Ok(Token {
                position,
                info: PositionalParameter(string[1..].to_string()),
            });
        }
        if let Some(string) = self.stream.scan(&self.operator_re) {
            return Ok(Token {
                position,
                info: Operator(string),
            });
        }
        if let Some(string) = self.stream.scan(&self.keyword_re) {
            return Ok(Token {
                position,
                info: Keyword(string),
            });
        }
        if let Some(string) = self.stream.scan(&self.identifier_re) {
            return Ok(Token {
                position,
                info: Identifier(string),
            });
        }
        Err(ParseError {
            position,
            problem: "Invalid token",
        })
    }
}

trait Stream {
    fn position(&self) -> usize;
    fn at_eof(&self) -> bool;
    fn at_eol(&self) -> bool;
    fn skip(&mut self) -> ();
    fn at_whitespace(&self) -> bool;
    fn str(&self) -> &str;
    fn charstr(&self) -> &str;
    fn string_from(&self, start: usize) -> String;
    fn scan(&mut self, re: &Regex) -> Option<String>;
    fn rewind(&mut self, position: usize);
}

struct StringStream {
    position: usize,
    string: String,
}

impl StringStream {
    fn new(string: String) -> StringStream {
        StringStream {
            position: 0,
            string,
        }
    }
}

impl Stream for StringStream {
    fn rewind(&mut self, position: usize) {
        self.position = position;
    }
    fn scan(&mut self, re: &Regex) -> Option<String> {
        let start = self.position();
        match re.find(self.str()) {
            Some(m) => {
                assert!(m.start() == 0);
                self.position += m.end();
                Some(self.string_from(start))
            }
            None => None,
        }
    }
    fn position(&self) -> usize {
        self.position
    }
    fn at_eof(&self) -> bool {
        self.position == self.string.len()
    }
    fn string_from(&self, start: usize) -> String {
        self.string[start..self.position].to_string()
    }
    fn skip(&mut self) {
        self.position += 1;
    }
    fn charstr(&self) -> &str {
        &self.string[self.position..self.position + 1]
    }
    fn str(&self) -> &str {
        &self.string[self.position..]
    }
    fn at_whitespace(&self) -> bool {
        if self.at_eof() {
            return false;
        }
        let here = self.charstr();
        here == " " || here == "\t" || here == "\r" || here == "\n"
    }
    fn at_eol(&self) -> bool {
        if self.at_eof() {
            return false;
        }
        let here = self.str();
        &here[0..1] == "\n" || (here.len() >= 2 && &here[0..2] == "\r\n")
    }
}

#[cfg(test)]
fn decimal(value: i64) -> Expr {
    Expr::Constant(Literal::Decimal(value))
}

#[cfg(test)]
fn float(value: f64) -> Expr {
    Expr::Constant(Literal::Float(value))
}

#[cfg(test)]
fn var(name: &str) -> Expr {
    Expr::Variable(name.to_string())
}

#[cfg(test)]
fn chain(object: Expr, messages: &[Message]) -> Expr {
    Expr::Chain(Box::new(object), messages.to_vec())
}

#[cfg(test)]
fn unary(name: &str) -> Message {
    Message {
        selector: name.to_string(),
        arguments: vec![],
    }
}

#[cfg(test)]
fn binary(name: &str, expr: Expr) -> Message {
    Message {
        selector: name.to_string(),
        arguments: vec![expr],
    }
}

#[cfg(test)]
fn keyword(name: &str, exprs: &[Expr]) -> Message {
    Message {
        selector: name.to_string(),
        arguments: exprs.to_vec(),
    }
}

#[test]
fn parse_decimal() {
    assert_eq!(parse_str(" 123 "), Ok(decimal(123)));
}

#[test]
fn parse_float() {
    assert_eq!(parse_str(" 123.123 "), Ok(float(123.123)));
}

#[test]
fn parse_variable() {
    assert_eq!(parse_str(" abc "), Ok(var("abc")));
}

#[test]
fn parse_unary_send() {
    assert_eq!(
        parse_str(" abc foo bar "),
        Ok(chain(var("abc"), &[unary("foo"), unary("bar")]))
    );
}

#[test]
fn parse_binary_send() {
    assert_eq!(
        parse_str(" abc + bar "),
        Ok(chain(var("abc"), &[binary("+", var("bar"))]))
    );
}

#[test]
fn parse_unary_prefix() {
    assert_eq!(parse_str(" -a "), Ok(chain(var("a"), &[unary("neg")])));
}

#[test]
fn parse_binary_precedence() {
    assert_eq!(
        parse_str(" abc + bar * quux"),
        Ok(chain(
            var("abc"),
            &[binary("+", chain(var("bar"), &[binary("*", var("quux"))]))]
        ))
    );
}

#[test]
fn parse_unary_and_binary_send() {
    assert_eq!(
        parse_str(" abc foo + bar foo2"),
        Ok(chain(
            var("abc"),
            &[
                unary("foo"),
                binary("+", chain(var("bar"), &[unary("foo2")]))
            ]
        ))
    );
}

#[test]
fn parse_keyword_send() {
    assert_eq!(
        parse_str(" obj key1: arg1 key2: arg2"),
        Ok(chain(
            var("obj"),
            &[keyword("key1:key2:", &[var("arg1"), var("arg2")])]
        ))
    );
}

#[test]
fn parse_keyword_chain() {
    assert_eq!(
        parse_str(" obj send1: arg1 -- send2: arg2"),
        Ok(chain(
            var("obj"),
            &[
                keyword("send1:", &[var("arg1")]),
                keyword("send2:", &[var("arg2")])
            ]
        ))
    );
}

#[test]
fn parse_keyword_unary_chain() {
    assert_eq!(
        parse_str(" obj send1: arg1 -- bar"),
        Ok(chain(
            var("obj"),
            &[keyword("send1:", &[var("arg1")]), unary("bar")]
        ))
    );
}

#[test]
fn parse_keyword_and_binary_send() {
    assert_eq!(
        parse_str(" obj key1: arg1 + x key2: arg2 + y"),
        Ok(chain(
            var("obj"),
            &[keyword(
                "key1:key2:",
                &[
                    chain(var("arg1"), &[binary("+", var("x"))]),
                    chain(var("arg2"), &[binary("+", var("y"))]),
                ]
            )]
        ))
    );
}

#[test]
fn parse_keyword_and_unary_send() {
    assert_eq!(
        parse_str(" obj key1: arg1 foo bar key2: arg2 quux zot"),
        Ok(chain(
            var("obj"),
            &[keyword(
                "key1:key2:",
                &[
                    chain(var("arg1"), &[unary("foo"), unary("bar")]),
                    chain(var("arg2"), &[unary("quux"), unary("zot")]),
                ]
            )]
        ))
    );
}

#[test]
fn parse_cascade() {
    assert_eq!(
        // This is not like smalltalk cascade!
        parse_str(
            "obj zoo
                   ; foo: x bar: y -- zot
                   ; do thing
                   "
        ),
        Ok(Expr::Cascade(
            Box::new(chain(var("obj"), &[unary("zoo")])),
            vec![
                vec![keyword("foo:bar:", &[var("x"), var("y")]), unary("zot")],
                vec![unary("do"), unary("thing"),]
            ]
        ))
    );
}

#[test]
fn test_parse_sequence() {
    assert_eq!(
        parse_str("foo bar, quux zot"),
        Ok(Expr::Sequence(vec![
            chain(var("foo"), &[unary("bar")]),
            chain(var("quux"), &[unary("zot")])
        ]))
    );
    assert_eq!(
        parse_str(
            "
            foo bar
            quux zot"
        ),
        Ok(Expr::Sequence(vec![
            chain(var("foo"), &[unary("bar")]),
            chain(var("quux"), &[unary("zot")])
        ]))
    );
    assert_eq!(
        parse_str(
            r"
            zoo foo +
              barz
            quux \
              + zot"
        ),
        Ok(Expr::Sequence(vec![
            chain(var("zoo"), &[unary("foo"), binary("+", var("barz"))]),
            chain(var("quux"), &[binary("+", var("zot"))])
        ]))
    );
}

#[test]
fn parse_block() {
    assert_eq!(
        parse_str("{ a + b }"),
        Ok(Expr::Block(
            vec![],
            Box::new(chain(var("a"), &[binary("+", var("b"))]))
        ))
    );
}

#[test]
fn parse_block_with_args() {
    assert_eq!(
        parse_str("{ :a :b | a + b }"),
        Ok(Expr::Block(
            vec!["a".to_string(), "b".to_string()],
            Box::new(chain(var("a"), &[binary("+", var("b"))]))
        ))
    );
}
