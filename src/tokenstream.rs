use std::ops::Range;

pub type Span = Range<usize>;

#[derive(Debug, PartialEq)]
pub enum Token {
    Annotation,
    Character,
    CloseBrace(Span),
    CloseBracket(Span),
    CloseParen(Span),
    Eof,
    Keyword,
    Number,
    GlobalId(Span),
    LocalId(Span),
    OpenBrace(Span),
    OpenBracket(Span),
    OpenParen(Span),
    Sigil(Span),
}

#[derive(Debug, PartialEq)]
pub struct SyntaxError {
    pub span: Span,
    pub problem: &'static str,
    pub context: String,
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
            self.context
                .push_str(format!("{:03} {}\n", lineno, line).as_str());
        }
    }
    pub fn add_context(mut self, source: &str) -> SyntaxError {
        println!("Add context: {:?} from {:?}", &self.span, source);
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
                println!("got it: {}", line);
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

const DELIMITERS: [char; 6] = ['(', ')', '{', '}', '[', ']'];

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

    pub fn slice(&self, span: Span) -> &str {
        &self.source[span]
    }

    pub fn span(&self) -> Span {
        self.span.clone()
    }

    pub fn error<T>(&self, problem: &'static str) -> Result<T, SyntaxError> {
        Err(SyntaxError::new(self.span(), problem))
    }

    fn result(&mut self, token: Token, span: Span) -> Result<Token, SyntaxError> {
        self.span = span;
        Ok(token)
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
            '<' => return self.scan_annotation_or_sigil(start.0),
            _ => {}
        }
        let numeric = start.1.is_numeric();
        let alphanumeric = start.1.is_alphanumeric();
        if start.1 == ':' {
            return self.result(Token::Keyword, start.0..start.0 + 1);
        }
        let mut end = self.getchar()?;
        loop {
            if end.1.is_whitespace() {
                break;
            }
            if self.at_eof() {
                end.0 = self.len();
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
            if DELIMITERS.iter().find(|x| **x == end.1).is_some() {
                break;
            }
            end = self.getchar()?;
        }
        let span = start.0..end.0;
        if !alphanumeric {
            if start.0 + 1 == end.0 {
                match start.1 {
                    '(' => return Ok(Token::OpenParen(span)),
                    ')' => return Ok(Token::CloseParen(span)),
                    '{' => return Ok(Token::OpenBrace(span)),
                    '}' => return Ok(Token::CloseBrace(span)),
                    '[' => return Ok(Token::OpenBracket(span)),
                    ']' => return Ok(Token::CloseBracket(span)),
                    _ => return Ok(Token::Sigil(span)),
                }
            } else {
                return Ok(Token::Sigil(span));
            }
        }
        if start.1.is_digit(10) {
            return self.result(Token::Number, span);
        }
        if start.1.is_uppercase() {
            Ok(Token::GlobalId(span))
        } else {
            Ok(Token::LocalId(span))
        }
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
    fn scan_annotation_or_sigil(&mut self, start: usize) -> Result<Token, SyntaxError> {
        let next = self.getchar()?;
        if next.1.is_whitespace() || next.1.is_digit(10) {
            self.unread(next);
            return Ok(Token::Sigil(start..next.0));
        }
        loop {
            let next = self.getchar()?;
            if next.1 == '>' {
                return self.result(Token::Annotation, start..next.0 + 1);
            }
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
fn scan_local_id() {
    assert_eq!(scan_str_part(" fo1 "), Ok(Token::LocalId(1..4)));
    assert_eq!(scan_str_part("fo1"), Ok(Token::LocalId(0..3)));
}

#[test]
fn scan_local_id2() {
    assert_eq!(scan_str_part(" fo1+ "), Ok(Token::LocalId(1..4)));
}

#[test]
fn scan_binary_op() {
    assert_eq!(
        scan_str(" foo++bar "),
        vec![
            Ok(Token::LocalId(1..4)),
            Ok(Token::Sigil(4..6)),
            Ok(Token::LocalId(6..9)),
        ]
    )
}

#[test]
fn scan_annotations() {
    let mut scanner = TokenStream::new("foo<Foo>+bar<Bar>");
    assert_eq!(scanner.scan(), Ok(Token::LocalId(0..3)));
    assert_eq!(scanner.scan(), Ok(Token::Annotation));
    assert_eq!(scanner.span(), 3..8);
    assert_eq!(scanner.scan(), Ok(Token::Sigil(8..9)));
    assert_eq!(scanner.scan(), Ok(Token::LocalId(9..12)));
    assert_eq!(scanner.scan(), Ok(Token::Annotation));
    assert_eq!(scanner.span(), 12..17);
}

#[test]
fn scan_global_id() {
    assert_eq!(scan_str_part(" Fo1 "), Ok(Token::GlobalId(1..4)));
}

#[test]
fn scan_global_id2() {
    assert_eq!(scan_str_part(" Fo1+ "), Ok(Token::GlobalId(1..4)));
}

#[test]
fn scan_number1() {
    let mut scanner = TokenStream::new(" 1xx ");
    assert_eq!(scanner.scan(), Ok(Token::Number));
    assert_eq!(scanner.slice(scanner.span()), "1xx");
}

#[test]
fn scan_number2() {
    let mut scanner = TokenStream::new(" 1.0 ");
    assert_eq!(scanner.scan(), Ok(Token::Number));
    assert_eq!(scanner.slice(scanner.span()), "1.0");
}

#[test]
fn scan_sigil() {
    assert_eq!(scan_str_part(" + "), Ok(Token::Sigil(1..2)))
}

#[test]
fn scan_sigil2() {
    assert_eq!(scan_str_part(" +foo "), Ok(Token::Sigil(1..2)))
}

#[test]
fn scan_keywords() {
    let mut scanner = TokenStream::new(" foo: 42 bar: 123 ");
    assert_eq!(scanner.scan(), Ok(Token::Keyword));
    assert_eq!(scanner.span(), 1..5);
    assert_eq!(scanner.scan(), Ok(Token::Number));
    assert_eq!(scanner.slice(scanner.span()), "42");
    assert_eq!(scanner.scan(), Ok(Token::Keyword));
    assert_eq!(scanner.span(), 9..13);
    assert_eq!(scanner.scan(), Ok(Token::Number));
    assert_eq!(scanner.slice(scanner.span()), "123");
}

#[test]
fn scan_bound_keywords() {
    let mut scanner = TokenStream::new(" foo:42 bar:123 ");
    assert_eq!(scanner.scan(), Ok(Token::Keyword));
    assert_eq!(scanner.slice(scanner.span()), "foo:");
    assert_eq!(scanner.scan(), Ok(Token::Number));
    assert_eq!(scanner.slice(scanner.span()), "42");
    assert_eq!(scanner.scan(), Ok(Token::Keyword));
    assert_eq!(scanner.slice(scanner.span()), "bar:");
    assert_eq!(scanner.scan(), Ok(Token::Number));
    assert_eq!(scanner.slice(scanner.span()), "123");
}

#[test]
fn scan_keyword2() {
    let mut scanner = TokenStream::new(" : ");
    assert_eq!(scanner.scan(), Ok(Token::Keyword));
    assert_eq!(scanner.slice(scanner.span()), ":")
}

#[test]
fn scan_open_paren() {
    assert_eq!(scan_str_part(" ((  "), Ok(Token::OpenParen(1..2)))
}

#[test]
fn scan_close_paren() {
    assert_eq!(scan_str_part(" )) "), Ok(Token::CloseParen(1..2)))
}

#[test]
fn scan_open_brace() {
    assert_eq!(scan_str_part(" {{ "), Ok(Token::OpenBrace(1..2)))
}

#[test]
fn scan_close_brace() {
    assert_eq!(scan_str_part(" }} "), Ok(Token::CloseBrace(1..2)))
}

#[test]
fn scan_open_bracket() {
    assert_eq!(scan_str_part(" [[ "), Ok(Token::OpenBracket(1..2)))
}

#[test]
fn scan_close_bracket() {
    assert_eq!(scan_str_part(" ]] "), Ok(Token::CloseBracket(1..2)))
}
