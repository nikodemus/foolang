use regex::Regex;

#[derive(Debug, PartialEq)]
enum Expr {
    Constant(Literal),

    Send(Box<Expr>, Selector, Vec<Expr>),
}

#[derive(Debug, PartialEq)]
enum Literal {
    Integer(i64),
}

#[derive(Debug, PartialEq)]
enum Token {
    Comment(String, usize),
    Decimal(String, usize),
    String(String, usize),
    Keyword(String, usize),
    Identifier(String, usize),
    Selector(String, usize),
    MiscOperator(String, usize),
    ChainOperator(usize),
    SeqOperator(usize),
    CascadeOperator(usize),
    /*
    Float(Source),
    OpenBlock(Source),
    CloseBlock(Source),
    OpenExpr(Source),
    CloseExpr(Source),
    OpenLiteralArray(Source),
    OpenRuntimeArray(Source),
    CloseArray(Source),
    */
}

#[cfg(test)]
impl Token {
    fn comment(text: &str, position: usize) -> Token {
        Token::Comment(String::from(text), position)
    }
    fn decimal(text: &str, position: usize) -> Token {
        Token::Decimal(String::from(text), position)
    }
    fn string(text: &str, position: usize) -> Token {
        Token::String(String::from(text), position)
    }
    fn identifier(text: &str, position: usize) -> Token {
        Token::Identifier(String::from(text), position)
    }
    fn keyword(text: &str, position: usize) -> Token {
        Token::Keyword(String::from(text), position)
    }
    fn selector(text: &str, position: usize) -> Token {
        Token::Selector(String::from(text), position)
    }
}

#[derive(Debug)]
struct ParseError {
    position: usize,
    problem: &'static str,
}

struct Grammar {
    comment: &'static str,
    string: &'static str,
    misc_operator: Regex,
    seq_operator: Regex,
    chain_operator: Regex,
    cascade_operator: Regex,
    keyword: Regex,
    identifier: Regex,
    selector: Regex,
}

impl Grammar {
    fn new() -> Grammar {
        Grammar {
            comment: "#",
            string: "\"",
            cascade_operator: Regex::new(r"\A;").unwrap(),
            seq_operator: Regex::new(r"\A,").unwrap(),
            chain_operator: Regex::new(r"\A--").unwrap(),
            misc_operator: Regex::new(r"\A[\-+*/=<>]+").unwrap(),
            keyword: Regex::new(r"\A[_a-zA-Z][_a-zA-Z0-9]*:").unwrap(),
            identifier: Regex::new(r"\A[_a-zA-Z][_a-zA-Z0-9]*").unwrap(),
            selector: Regex::new(r"\A\$[_a-zA-Z][_a-zA-Z0-9]*(:([_a-zA-Z]+[0-9]*:)*)?").unwrap(),
        }
    }
    fn tokenize(&self, input: &mut impl Stream) -> Result<Vec<Token>, ParseError> {
        let mut tokens = Vec::new();
        while self.parse_token(input, &mut tokens)? {}
        Ok(tokens)
    }
    fn parse(&self, tokens: Vec<Token>) -> ast::Expr {
        let output = Vec::new();
        let operators = Vec::new();
        for token in tokens.into_iter() {
            use Token::*;
            match token {
                Decimal(s, _) => output.push(ast::Constant(ast::Literal::Integer(s.parse()))),
                MiscOperator(s, _) => operators.push(ast::Oper()),
                _ => panic!("Don't know how to deal with: {}", token),
            }
        }
    }
    fn parse_token(
        &self,
        input: &mut impl Stream,
        tokens: &mut Vec<Token>,
    ) -> Result<bool, ParseError> {
        while input.at_whitespace() {
            input.skip();
        }
        if input.at_eof() {
            return Ok(false);
        }
        if input.matches(self.comment) {
            tokens.push(self.scan_comment(input));
            return Ok(true);
        }
        if input.at_digit() {
            tokens.push(self.scan_number(input));
            return Ok(true);
        }
        if input.matches(self.string) {
            tokens.push(self.scan_string(input));
            return Ok(true);
        }
        if input.re_matches(&self.seq_operator) {
            let (_, pos) = self.scan_re(input, &self.seq_operator);
            tokens.push(Token::SeqOperator(pos));
            return Ok(true);
        }
        if input.re_matches(&self.chain_operator) {
            let (_, pos) = self.scan_re(input, &self.chain_operator);
            tokens.push(Token::ChainOperator(pos));
            return Ok(true);
        }
        if input.re_matches(&self.cascade_operator) {
            let (_, pos) = self.scan_re(input, &self.cascade_operator);
            tokens.push(Token::CascadeOperator(pos));
            return Ok(true);
        }
        // NOTE: It is important that keyword test is before
        // identifier!
        if input.re_matches(&self.keyword) {
            let (string, pos) = self.scan_re(input, &self.keyword);
            tokens.push(Token::Keyword(string, pos));
            return Ok(true);
        }
        // NOTE: It is important that keyword test is before
        // identifier!
        if input.re_matches(&self.identifier) {
            let (string, pos) = self.scan_re(input, &self.identifier);
            tokens.push(Token::Identifier(string, pos));
            return Ok(true);
        }
        if input.re_matches(&self.selector) {
            let (string, pos) = self.scan_re(input, &self.selector);
            tokens.push(Token::Selector(string, pos));
            return Ok(true);
        }
        if input.re_matches(&self.misc_operator) {
            let (string, pos) = self.scan_re(input, &self.misc_operator);
            tokens.push(Token::MiscOperator(string, pos));
            return Ok(true);
        }
        let position = input.position();
        return Err(ParseError {
            position,
            problem: "Invalid token",
        });
    }
    fn scan_comment(&self, input: &mut impl Stream) -> Token {
        let start = input.position();
        while !(input.at_eol() || input.at_eof()) {
            input.skip();
        }
        Token::Comment(input.string_from(start), start)
    }
    fn scan_number(&self, input: &mut impl Stream) -> Token {
        let start = input.position();
        while input.at_digit() {
            input.skip();
        }
        Token::Decimal(input.string_from(start), start)
    }
    fn scan_string(&self, input: &mut impl Stream) -> Token {
        let start = input.position();
        loop {
            input.skip();
            if input.matches("\\") {
                input.skip();
                continue;
            }
            if input.matches(self.string) {
                input.skip();
                break;
            }
        }
        Token::String(input.string_from(start), start)
    }
}

trait Stream {
    fn matches(&self, needle: &str) -> bool;
    fn re_matches(&self, re: &Regex) -> bool;
    fn re_scan(&mut self, re: &Regex) -> String;
    fn position(&self) -> usize;
    fn at_eof(&self) -> bool;
    fn at_eol(&self) -> bool;
    fn try_skip_eol(&mut self) -> ();
    fn skip(&mut self) -> ();
    fn at_whitespace(&self) -> bool;
    fn at_digit(&self) -> bool;
    fn str(&self) -> &str;
    fn string_from(&self, start: usize) -> String;
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
    fn matches(&self, needle: &str) -> bool {
        let i = needle.len() + self.position;
        if i < self.string.len() {
            &self.string[self.position..i] == needle
        } else {
            false
        }
    }
    fn re_matches(&self, re: &Regex) -> bool {
        re.is_match(&self.string[self.position..])
    }
    fn re_scan(&mut self, re: &Regex) -> String {
        let start = self.position;
        match re.find(&self.string[start..]) {
            Some(m) => {
                assert!(m.start() == 0);
                self.position += m.end();
                self.string_from(start)
            }
            None => panic!("No match for {} at {}!", re, &self.string[self.position..]),
        }
    }
    fn position(&self) -> usize {
        self.position
    }
    fn at_eof(&self) -> bool {
        self.position == self.string.len()
    }
    fn at_eol(&self) -> bool {
        self.matches("\n") || self.matches("\r\n")
    }
    fn try_skip_eol(&mut self) {
        if self.matches("\r\n") {
            self.position += 2;
        } else if self.matches("\n") {
            self.position += 1;
        }
    }
    fn string_from(&self, start: usize) -> String {
        self.string[start..self.position].to_string()
    }
    fn skip(&mut self) {
        self.position += 1;
    }
    fn str(&self) -> &str {
        &self.string[self.position..self.position + 1]
    }
    fn at_whitespace(&self) -> bool {
        if self.at_eof() {
            return false;
        }
        let here = self.str();
        here == " " || here == "\t" || here == "\r" || here == "\n"
    }
    fn at_digit(&self) -> bool {
        if self.at_eof() {
            return false;
        }
        self.str().chars().next().unwrap_or('x').is_digit(10)
    }
}

fn parse_string(string: String) -> Vec<Token> {
    let grammar = Grammar::new();
    let mut stream = StringStream::new(string);
    match grammar.tokenize(&mut stream) {
        Ok(tokens) => tokens,
        Err(ParseError { position, problem }) => {
            panic!("{}: {}", problem, &stream.string[position..])
        }
    }
}

#[allow(unused)]
fn parse_str(string: &str) -> Vec<Token> {
    parse_string(String::from(string))
}

#[test]
fn test_parse_comment() {
    assert_eq!(
        parse_str(
            "
            # foo
            # bar
            "
        ),
        vec![Token::comment("# foo", 13), Token::comment("# bar", 31),]
    );
}

#[test]
fn test_parse_decimal() {
    assert_eq!(
        parse_str(
            "
            # foo
            12312
            "
        ),
        vec![Token::comment("# foo", 13), Token::decimal("12312", 31),]
    );
}

#[test]
fn test_parse_string() {
    assert_eq!(
        parse_str(
            r#"
            "this is a \"test\" with\nnewline"
            "#
        ),
        vec![Token::string(r#""this is a \"test\" with\nnewline""#, 13)]
    );
}

#[test]
fn test_parse_identifier() {
    assert_eq!(
        parse_str(
            r#"
            _fooBar
            "#
        ),
        vec![Token::identifier("_fooBar", 13)]
    );
}

#[test]
fn test_parse_keyword() {
    assert_eq!(
        parse_str(
            r#"
            _fooBar: quux
            "#
        ),
        vec![
            Token::keyword("_fooBar:", 13),
            Token::identifier("quux", 22),
        ]
    );
}

#[test]
fn test_parse_selector() {
    assert_eq!(
        parse_str(
            r#"
            $foo
            "#
        ),
        vec![Token::selector("$foo", 13)]
    );
    assert_eq!(
        parse_str(
            r#"
            $foo:
            "#
        ),
        vec![Token::selector("$foo:", 13)]
    );
    assert_eq!(
        parse_str(
            r#"
            $foo:bar:
            "#
        ),
        vec![Token::selector("$foo:bar:", 13)]
    );
}

#[test]
fn test_parse_chain() {
    assert_eq!(
        parse_str(
            r#"
            --
            "#
        ),
        vec![Token::ChainOperator(13)]
    );
}

#[test]
fn test_parse_seq() {
    assert_eq!(
        parse_str(
            r#"
            ,
            "#
        ),
        vec![Token::SeqOperator(13)]
    );
}

#[test]
fn test_parse_cascade() {
    assert_eq!(
        parse_str(
            r#"
            ;
            "#
        ),
        vec![Token::CascadeOperator(13)]
    );
}

#[test]
fn test_parse_operator() {
    assert_eq!(
        parse_str(
            r#"
            foo-bar
            "#
        ),
        vec![
            Token::identifier("foo", 13),
            Token::MiscOperator("-".to_string(), 16),
            Token::identifier("bar", 17)
        ]
    );
}
