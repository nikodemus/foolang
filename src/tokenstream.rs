use std::ops::Range;

pub type Span = Range<usize>;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum Token {
    Annotation,
    Character,
    Eof,
    Keyword,
    Number,
    Identifier,
    Operator,
}

#[derive(PartialEq)]
pub struct SyntaxError {
    pub span: Span,
    pub problem: &'static str,
    pub context: String,
}

impl std::fmt::Debug for SyntaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "SyntaxError({:?} at {:?}):\n{}", self.problem, self.span, self.context)
    }
}

impl SyntaxError {
    pub fn new(span: Span, problem: &'static str) -> SyntaxError {
        SyntaxError {
            span,
            problem,
            context: String::new(),
        }
    }
    fn append_context_line(&mut self, lineno: usize, line: &str) {
        if lineno == 0 {
            self.context.push_str(format!("    {}\n", line).as_str());
        } else {
            self.context.push_str(format!("{:03} {}\n", lineno, line).as_str());
        }
    }
    pub fn add_context(mut self, source: &str) -> SyntaxError {
        let mut prev = "";
        let mut lineno = 1;
        let mut start = 0;
        for line in source.lines() {
            if start >= self.span.end {
                // Line after the problem -- done.
                self.append_context_line(lineno, line);
                break;
            }
            let end = start + line.len();
            if end > self.span.start {
                // Previous line if there is one.
                if lineno > 1 {
                    self.append_context_line(lineno - 1, prev);
                }
                // Line with the problem.
                self.append_context_line(lineno, line);
                let mut mark = String::from_utf8(vec![b' '; self.span.start - start]).unwrap();
                mark.push_str(
                    String::from_utf8(vec![b'^'; self.span.end - self.span.start])
                        .unwrap()
                        .as_str(),
                );
                mark.push_str(" ");
                mark.push_str(self.problem);
                self.append_context_line(0, mark.as_str());
            }
            prev = line;
            start = end + 1;
            lineno += 1;
        }
        self
    }
}

fn is_open_delimiter(c: char) -> bool {
    c == '(' || c == '[' || c == '{'
}

fn is_close_delimiter(c: char) -> bool {
    c == ')' || c == ']' || c == '}'
}

fn is_delimiter(c: char) -> bool {
    is_open_delimiter(c) || is_close_delimiter(c) || c.is_whitespace()
}

pub fn scan_str_part(s: &str) -> Result<Token, SyntaxError> {
    TokenStream::new(s).scan()
}

pub fn scan_str(s: &str) -> Vec<Result<Token, SyntaxError>> {
    let mut stream = TokenStream::new(s);
    let mut result = vec![];
    while !stream.at_eof() {
        result.push(stream.scan());
    }
    result
}

pub struct TokenStream<'a> {
    source: &'a str,
    indices: std::cell::RefCell<std::str::CharIndices<'a>>,
    cache: std::cell::RefCell<Vec<(usize, char)>>,
    span: Span,
}

impl<'a> TokenStream<'a> {
    pub fn new(source: &'a str) -> TokenStream<'a> {
        TokenStream {
            source,
            indices: std::cell::RefCell::new(source.char_indices()),
            cache: std::cell::RefCell::new(Vec::new()),
            span: 0..0,
        }
    }

    pub fn slice(&self) -> &str {
        &self.source[self.span()]
    }

    pub fn slice_at(&self, span: Span) -> &str {
        &self.source[span]
    }

    pub fn tokenstring(&self) -> String {
        self.slice().to_string()
    }

    pub fn span(&self) -> Span {
        self.span.clone()
    }

    pub fn error_at<T>(&self, span: Span, problem: &'static str) -> Result<T, SyntaxError> {
        Err(SyntaxError::new(span, problem))
    }

    pub fn error<T>(&self, problem: &'static str) -> Result<T, SyntaxError> {
        self.error_at(self.span(), problem)
    }

    fn result(&mut self, token: Token, span: Span) -> Result<Token, SyntaxError> {
        self.span = span;
        Ok(token)
    }

    pub fn lookahead(&mut self) -> Result<(Token, Span), SyntaxError> {
        // There has got to be a better way... (Doing this in parser?)
        let indices = self.indices.clone();
        let cache = self.cache.clone();
        let span = self.span.clone();
        let lookahead_token = self.scan()?;
        let lookahead_span = self.span.clone();
        self.indices = indices;
        self.cache = cache;
        self.span = span;
        Ok((lookahead_token, lookahead_span))
    }

    pub fn scan(&mut self) -> Result<Token, SyntaxError> {
        let mut start;
        loop {
            if self.at_eof() {
                return self.result(Token::Eof, self.len()..self.len());
            }
            start = self.getchar()?;
            if !start.1.is_whitespace() {
                break;
            }
        }
        match start.1 {
            '\'' => return self.scan_character(start.0),
            '<' => return self.scan_annotation_or_operator(start.0),
            '@' => return self.scan_toplevel_or_operator(start.0),
            _ => {}
        }
        let numeric = start.1.is_numeric();
        let alphanumeric = start.1.is_alphanumeric();
        if start.1 == ':' {
            return self.result(Token::Keyword, start.0..start.0 + 1);
        }
        if is_open_delimiter(start.1) {
            return self.result(Token::Operator, start.0..start.0 + 1);
        }
        if is_close_delimiter(start.1) {
            return self.result(Token::Operator, start.0..start.0 + 1);
        }
        let mut end = start.clone();
        loop {
            if self.at_eof() {
                end.0 = self.len();
                break;
            }

            end = self.getchar()?;

            if is_delimiter(end.1) {
                break;
            }
            if end.1 == ':' {
                return self.result(Token::Keyword, start.0..end.0 + 1);
            }
            if (!numeric || (end.1 != '.' && end.1 != '_'))
                && alphanumeric != end.1.is_alphanumeric()
            {
                self.unread(end);
                break;
            }
        }
        let span = start.0..end.0;
        if !alphanumeric {
            return self.result(Token::Operator, span);
        }
        if start.1.is_digit(10) {
            return self.result(Token::Number, span);
        }
        self.result(Token::Identifier, span)
    }
    fn len(&self) -> usize {
        self.source.len()
    }
    fn at_eof(&self) -> bool {
        if !self.cache.borrow().is_empty() {
            return false;
        }
        match self.indices.borrow_mut().next() {
            None => return true,
            Some(result) => {
                self.cache.borrow_mut().push(result);
                return false;
            }
        }
    }
    fn unread(&mut self, result: (usize, char)) {
        self.cache.borrow_mut().push(result)
    }
    fn getchar(&mut self) -> Result<(usize, char), SyntaxError> {
        if let Some(cached) = self.cache.borrow_mut().pop() {
            return Ok(cached);
        }
        self.indices
            .borrow_mut()
            .next()
            .ok_or(SyntaxError::new(self.len()..self.len(), "Unexpected EOF"))
    }

    fn scan_annotation_or_operator(&mut self, start: usize) -> Result<Token, SyntaxError> {
        let mut next = self.getchar()?;
        // Annotations always start with alphabetic characters.
        if next.1.is_alphabetic() {
            loop {
                next = self.getchar()?;
                if next.1 == '>' {
                    return self.result(Token::Annotation, start..next.0 + 1);
                }
            }
        }
        loop {
            if is_delimiter(next.1) || next.1.is_digit(10) {
                self.unread(next);
                return self.result(Token::Operator, start..next.0);
            }
            next = self.getchar()?;
        }
    }

    fn scan_character(&mut self, start: usize) -> Result<Token, SyntaxError> {
        self.getchar()?;
        let (end, quote) = self.getchar()?;
        if quote != '\'' {
            return Err(SyntaxError::new(start..end, "Malformed character literal"));
        }
        self.result(Token::Character, start..end + 1)
    }

    fn scan_toplevel_or_operator(&mut self, start: usize) -> Result<Token, SyntaxError> {
        let mut next = self.getchar()?;
        // Toplevels consist only of alphabetic characters.
        if next.1.is_alphabetic() {
            loop {
                next = self.getchar()?;
                if !next.1.is_alphabetic() {
                    self.unread(next);
                    // Toplevels are identifiers
                    return self.result(Token::Identifier, start..next.0);
                }
            }
        }
        loop {
            if is_delimiter(next.1) || next.1.is_digit(10) {
                self.unread(next);
                return self.result(Token::Operator, start..next.0);
            }
            next = self.getchar()?;
        }
    }
}

#[test]
fn scan_eof() {
    let mut scanner = TokenStream::new("   ");
    assert_eq!(scanner.scan(), Ok(Token::Eof));
    assert_eq!(scanner.span(), 3..3);
}

#[test]
fn scan_char() {
    let mut scanner = TokenStream::new(" 'x' ");
    assert_eq!(scanner.scan(), Ok(Token::Character));
    assert_eq!(scanner.span(), 1..4);
    let mut scanner = TokenStream::new("'x'");
    assert_eq!(scanner.scan(), Ok(Token::Character));
    assert_eq!(scanner.span(), 0..3);
}

#[test]
fn scan_identifier() {
    fn test(source: &str, want: &str) {
        let mut scanner = TokenStream::new(source);
        assert_eq!(scanner.scan(), Ok(Token::Identifier));
        assert_eq!(scanner.slice(), want);
    }
    test("f", "f");
    test(" f ", "f");
    test(" fo1o ", "fo1o");
    test("fo1o", "fo1o");
    test(" fo1o+ ", "fo1o");
    test(" fo1+o ", "fo1");

    test("F", "F");
    test(" F ", "F");
    test(" Fo1o ", "Fo1o");
    test("Fo1o", "Fo1o");
    test(" Fo1o+ ", "Fo1o");
    test(" Fo1+O ", "Fo1");

    test(" @foo ", "@foo");
}

#[test]
fn scan_binary_op() {
    let mut scanner = TokenStream::new(" foo++bar ");
    assert_eq!(scanner.scan(), Ok(Token::Identifier));
    assert_eq!(scanner.slice(), "foo");
    assert_eq!(scanner.scan(), Ok(Token::Operator));
    assert_eq!(scanner.slice(), "++");
    assert_eq!(scanner.scan(), Ok(Token::Identifier));
    assert_eq!(scanner.slice(), "bar");
}

#[test]
fn scan_annotations() {
    let mut scanner = TokenStream::new("foo<Foo>+bar<Bar>");
    assert_eq!(scanner.scan(), Ok(Token::Identifier));
    assert_eq!(scanner.slice(), "foo");
    assert_eq!(scanner.scan(), Ok(Token::Annotation));
    assert_eq!(scanner.slice(), "<Foo>");
    assert_eq!(scanner.scan(), Ok(Token::Operator));
    assert_eq!(scanner.slice(), "+");
    assert_eq!(scanner.scan(), Ok(Token::Identifier));
    assert_eq!(scanner.slice(), "bar");
    assert_eq!(scanner.scan(), Ok(Token::Annotation));
    assert_eq!(scanner.slice(), "<Bar>");
}

#[test]
fn scan_lte() {
    let mut scanner = TokenStream::new(" <= ");
    assert_eq!(scanner.scan(), Ok(Token::Operator));
    assert_eq!(scanner.slice(), "<=");
}

#[test]
fn scan_number1() {
    let mut scanner = TokenStream::new(" 1xx ");
    assert_eq!(scanner.scan(), Ok(Token::Number));
    assert_eq!(scanner.slice(), "1xx");
}

#[test]
fn scan_number2() {
    let mut scanner = TokenStream::new(" 1.0 ");
    assert_eq!(scanner.scan(), Ok(Token::Number));
    assert_eq!(scanner.slice(), "1.0");
}

#[test]
fn scan_operator() {
    fn test(mut scanner: TokenStream) {
        assert_eq!(scanner.scan(), Ok(Token::Operator));
        assert_eq!(scanner.slice(), "+");
    }
    test(TokenStream::new(" + "));
    test(TokenStream::new(" +foo "));
}

#[test]
fn scan_keywords() {
    let mut scanner = TokenStream::new(" foo: 42 bar: 123 ");
    assert_eq!(scanner.scan(), Ok(Token::Keyword));
    assert_eq!(scanner.span(), 1..5);
    assert_eq!(scanner.scan(), Ok(Token::Number));
    assert_eq!(scanner.slice(), "42");
    assert_eq!(scanner.scan(), Ok(Token::Keyword));
    assert_eq!(scanner.span(), 9..13);
    assert_eq!(scanner.scan(), Ok(Token::Number));
    assert_eq!(scanner.slice(), "123");
}

#[test]
fn scan_bound_keywords() {
    let mut scanner = TokenStream::new(" foo:42 bar:123 ");
    assert_eq!(scanner.scan(), Ok(Token::Keyword));
    assert_eq!(scanner.slice(), "foo:");
    assert_eq!(scanner.scan(), Ok(Token::Number));
    assert_eq!(scanner.slice(), "42");
    assert_eq!(scanner.scan(), Ok(Token::Keyword));
    assert_eq!(scanner.slice(), "bar:");
    assert_eq!(scanner.scan(), Ok(Token::Number));
    assert_eq!(scanner.slice(), "123");
}

#[test]
fn scan_keyword2() {
    let mut scanner = TokenStream::new(" : ");
    assert_eq!(scanner.scan(), Ok(Token::Keyword));
    assert_eq!(scanner.slice(), ":")
}

#[test]
fn scan_open_paren() {
    let mut scanner = TokenStream::new(" ((  ");
    assert_eq!(scanner.scan(), Ok(Token::Operator));
    assert_eq!(scanner.slice(), "(");
}

#[test]
fn scan_close_paren() {
    let mut scanner = TokenStream::new(" ))  ");
    assert_eq!(scanner.scan(), Ok(Token::Operator));
    assert_eq!(scanner.slice(), ")");
}

#[test]
fn scan_open_brace() {
    let mut scanner = TokenStream::new(" {{  ");
    assert_eq!(scanner.scan(), Ok(Token::Operator));
    assert_eq!(scanner.slice(), "{");
}

#[test]
fn scan_close_brace() {
    let mut scanner = TokenStream::new(" }}  ");
    assert_eq!(scanner.scan(), Ok(Token::Operator));
    assert_eq!(scanner.slice(), "}");
}

#[test]
fn scan_open_bracket() {
    let mut scanner = TokenStream::new(" [[  ");
    assert_eq!(scanner.scan(), Ok(Token::Operator));
    assert_eq!(scanner.slice(), "[");
}

#[test]
fn scan_close_bracket() {
    let mut scanner = TokenStream::new(" ]]  ");
    assert_eq!(scanner.scan(), Ok(Token::Operator));
    assert_eq!(scanner.slice(), "]");
}
