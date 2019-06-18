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
    Sequence(bool),
    Newline(),
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
    cont_re: Regex,
    sequence_re: Regex,
    chain_re: Regex,
    cascade_re: Regex,
    decimal_re: Regex,
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
            cont_re: Regex::new(r"^\\").unwrap(),
            sequence_re: Regex::new(r"^,").unwrap(),
            chain_re: Regex::new(r"^--").unwrap(),
            cascade_re: Regex::new(r"^;").unwrap(),
            decimal_re: Regex::new(r"^[0-9]+").unwrap(),
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
        use TokenInfo::*;
        match token.info {
            Cascade() => match &self.cascade {
                None => Err(ParseError {
                    position: token.position,
                    problem: "Malformed cascade",
                }),
                Some(expr) => {
                    let receiver = expr.to_owned();
                    self.cascade = None;
                    Ok(receiver)
                }
            },
            Constant(literal) => Ok(Expr::Constant(literal)),
            Identifier(name) => Ok(Expr::Variable(name)),
            // Leading newline, ignore.
            Sequence(false) => self.parse_prefix(),
            Eof() => Err(ParseError {
                position: token.position,
                problem: "Unexpected end of input",
            }),
            _ => {
                println!("PREFIX OOPS: {:?}", token.info);
                Err(ParseError {
                    position: token.position,
                    problem: "Not a value expression",
                })
            }
        }
    }
    fn parse_suffix(&mut self, left: Expr) -> Result<Expr, ParseError> {
        let token = self.consume_token()?;
        let precedence = self.token_precedence(&token);
        use TokenInfo::*;
        match token.info {
            Identifier(name) => Ok(left.unary(name)),
            Sequence(_) => {
                let mut right = self.parse_expression(precedence)?;
                let mut expressions = vec![left];
                if let Expr::Sequence(mut right_expressions) = right {
                    expressions.append(&mut right_expressions);
                } else {
                    expressions.push(right);
                }
                Ok(Expr::Sequence(expressions))
            }
            Chain() => Ok(left),
            Cascade() => {
                assert_eq!(self.cascade, None);
                let mut chains = vec![];
                self.insert_token(token);
                loop {
                    let next = self.peek_token()?;
                    if Cascade() != next.info {
                        break;
                    }
                    let position = next.position;
                    // Need to wrap in cascade in case this is a chain,
                    // which would then accumulate the cascaded messages.
                    let mark = Expr::Variable("(cascade)".to_string());
                    self.cascade = Some(mark.clone());
                    let chain = self.parse_expression(precedence)?;
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
            Operator(name) => {
                let right = self.parse_expression(precedence)?;
                Ok(left.binary(name, right))
            }
            // FIXME: refactor into a separate function.
            Keyword(name) => {
                let mut args = vec![];
                let mut selector = name;
                loop {
                    args.push(self.parse_expression(precedence)?);
                    if let Token {
                        position: _,
                        info: Keyword(next),
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
            Eof() => Err(ParseError {
                position: token.position,
                problem: "Unexpected end of input",
            }),
            _ => Err(ParseError {
                position: token.position,
                problem: "Not valid in suffix position",
            }),
        }
    }
    fn next_precedence(&mut self) -> Result<usize, ParseError> {
        let token = self.peek_token()?;
        Ok(self.token_precedence(&token))
    }
    fn token_precedence(&self, token: &Token) -> usize {
        use TokenInfo::*;
        match &token.info {
            // EOF is the only token that can have precedence zero!
            Eof() => 0,
            Sequence(_) => 1,
            Cascade() => 2,
            Chain() => 3,
            Keyword(_) => 4,
            // 10 is fallback for unknown operators
            Operator(op) => match op.as_str() {
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
        if let Some(_) = self.stream.scan(&self.sequence_re) {
            return Ok(Token {
                position,
                info: Sequence(true),
            });
        }
        if let Some(string) = self.stream.scan(&self.decimal_re) {
            // Asserting termination here would catch 123asd..
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
