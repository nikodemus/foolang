use regex::Regex;

#[derive(Debug, PartialEq)]
enum Token {
    Comment(String, usize),
    Decimal(String, usize),
    String(String, usize),
    Keyword(String, usize),
    Identifier(String, usize),
    Selector(String, usize),
    /*
    Float(Source),
    SequenceOperator(Source),
    CascadeOperator(Source),
    ChainOperator(Source)
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
    keyword: Regex,
    identifier: Regex,
    selector: Regex,
}

impl Grammar {
    fn new() -> Grammar {
        Grammar {
            comment: "#",
            string: r#"""#,
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
        // NOTE: It is important that keyword test is before
        // identifier!
        if input.re_matches(&self.keyword) {
            tokens.push(self.scan_keyword(input));
            return Ok(true);
        }
        // NOTE: It is important that keyword test is before
        // identifier!
        if input.re_matches(&self.identifier) {
            tokens.push(self.scan_identifier(input));
            return Ok(true);
        }
        if input.re_matches(&self.selector) {
            tokens.push(self.scan_selector(input));
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
        while !input.at_eol_or_eof() {
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
    fn scan_keyword(&self, input: &mut impl Stream) -> Token {
        let start = input.position();
        Token::Keyword(input.re_scan(&self.keyword), start)
    }
    fn scan_identifier(&self, input: &mut impl Stream) -> Token {
        let start = input.position();
        Token::Identifier(input.re_scan(&self.identifier), start)
    }
    fn scan_selector(&self, input: &mut impl Stream) -> Token {
        let start = input.position();
        Token::Selector(input.re_scan(&self.selector), start)
    }
}

trait Stream {
    fn matches(&self, needle: &str) -> bool;
    fn re_matches(&self, re: &Regex) -> bool;
    fn re_scan(&mut self, re: &Regex) -> String;
    fn position(&self) -> usize;
    fn at_eol_or_eof(&self) -> bool;
    fn at_eof(&self) -> bool;
    fn try_skip_eol(&mut self) -> ();
    fn skip(&mut self) -> ();
    fn at_whitespace(&self) -> bool;
    fn at_digit(&self) -> bool;
    fn here(&self) -> &str;
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
    fn at_eol_or_eof(&self) -> bool {
        self.at_eof() || self.matches("\n") || self.matches("\r\n")
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
    fn here(&self) -> &str {
        &self.string[self.position..self.position + 1]
    }
    fn at_whitespace(&self) -> bool {
        if self.at_eof() {
            return false;
        }
        let here = self.here();
        here == " " || here == "\t" || here == "\r" || here == "\n"
    }
    fn at_digit(&self) -> bool {
        if self.at_eof() {
            return false;
        }
        self.here().chars().next().unwrap_or('x').is_digit(10)
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
            _fooBar:
            "#
        ),
        vec![Token::keyword("_fooBar:", 13)]
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
