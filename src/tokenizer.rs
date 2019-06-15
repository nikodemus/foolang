#[derive(Debug, PartialEq)]
struct Source {
    start: usize,
    end: usize,
}

#[derive(Debug, PartialEq)]
enum Token {
    Comment(Source),
    Decimal(Source),
    /*
    Float(Source),
    String(Source),
    Symbol(Source),
    Identifier(Source),
    Keyword(Source),
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

#[derive(Debug)]
struct ParseError {
    source: Source,
    problem: &'static str,
}

struct Grammar {
    comment: &'static str,
}

impl Grammar {
    fn new() -> Grammar {
        Grammar { comment: "#" }
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
        let position = input.position();
        return Err(ParseError {
            source: Source {
                start: position,
                end: position,
            },
            problem: "Invalid token",
        });
    }
    fn scan_comment(&self, input: &mut impl Stream) -> Token {
        let start = input.position();
        while !input.at_eol_or_eof() {
            input.skip();
        }
        Token::Comment(Source {
            start,
            end: input.position(),
        })
    }
    fn scan_number(&self, input: &mut impl Stream) -> Token {
        let start = input.position();
        while input.at_digit() {
            input.skip();
        }
        Token::Decimal(Source {
            start,
            end: input.position(),
        })
    }
}

trait Stream {
    fn matches(&self, needle: &str) -> bool;
    fn position(&self) -> usize;
    fn at_eol_or_eof(&self) -> bool;
    fn at_eof(&self) -> bool;
    fn try_skip_eol(&mut self) -> ();
    fn skip(&mut self) -> ();
    fn at_whitespace(&self) -> bool;
    fn at_digit(&self) -> bool;
    fn here(&self) -> &str;
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
    grammar.tokenize(&mut stream).unwrap()
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
        vec![
            Token::Comment(Source { start: 13, end: 18 }),
            Token::Comment(Source { start: 31, end: 36 }),
        ]
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
        vec![
            Token::Comment(Source { start: 13, end: 18 }),
            Token::Decimal(Source { start: 31, end: 36 }),
        ]
    );
}
