type Span = (usize, usize);

#[derive(Debug, PartialEq)]
enum Token {
    Annotation(Span),
    Character(Span),
    CloseBrace(Span),
    CloseBracket(Span),
    CloseParen(Span),
    Eof(Span),
    Keyword(Span),
    Number(Span),
    GlobalId(Span),
    LocalId(Span),
    OpenBrace(Span),
    OpenBracket(Span),
    OpenParen(Span),
    Sigil(Span),
}

#[derive(Debug, PartialEq)]
struct SyntaxError {
    span: Span,
    problem: &'static str,
}

impl SyntaxError {
    pub fn new(span: Span, problem: &'static str) -> SyntaxError {
        SyntaxError { span, problem }
    }
}

const DELIMITERS: [char; 6] = ['(', ')', '{', '}', '[', ']'];

fn scan_str_part(s: &str) -> Result<Token, SyntaxError> {
    StringStream::new(s).scan()
}

fn scan_str(s: &str) -> Vec<Result<Token, SyntaxError>> {
    let mut stream = StringStream::new(s);
    let mut result = vec![];
    while !stream.at_eof() {
        result.push(stream.scan());
    }
    result
}

trait Stream {
    fn scan(&mut self) -> Result<Token, SyntaxError> {
        let mut start;
        loop {
            if self.at_eof() {
                return Ok(Token::Eof((self.len(), self.len())));
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
        let alphanumeric = start.1.is_alphanumeric();
        if start.1 == ':' {
            return Ok(Token::Keyword((start.0, start.0 + 1)));
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
                return Ok(Token::Keyword((start.0, end.0 + 1)));
            } else if alphanumeric != end.1.is_alphanumeric() {
                self.unread(end);
                break;
            }
            if DELIMITERS.iter().find(|x| **x == end.1).is_some() {
                break;
            }
            end = self.getchar()?;
        }
        let span = (start.0, end.0);
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
            return Ok(Token::Number(span));
        }
        if start.1.is_uppercase() {
            Ok(Token::GlobalId(span))
        } else {
            Ok(Token::LocalId(span))
        }
    }
    fn scan_annotation_or_sigil(&mut self, start: usize) -> Result<Token, SyntaxError> {
        let next = self.getchar()?;
        if next.1.is_whitespace() || next.1.is_digit(10) {
            self.unread(next);
            return Ok(Token::Sigil((start, next.0)));
        }
        loop {
            let next = self.getchar()?;
            if next.1 == '>' {
                return Ok(Token::Annotation((start, next.0 + 1)));
            }
        }
    }
    fn scan_character(&mut self, start: usize) -> Result<Token, SyntaxError> {
        self.getchar()?;
        let (end, quote) = self.getchar()?;
        if quote != '\'' {
            return Err(SyntaxError::new(
                (start, end),
                "Malformed character literal",
            ));
        }
        Ok(Token::Character((start, end + 1)))
    }
    // --
    fn at_eof(&self) -> bool;
    fn unread(&mut self, result: (usize, char));
    fn getchar(&mut self) -> Result<(usize, char), SyntaxError>;
    fn len(&self) -> usize;
}

#[test]
fn scan_eof() {
    assert_eq!(scan_str_part("   "), Ok(Token::Eof((3, 3))));
}

#[test]
fn scan_char() {
    assert_eq!(scan_str_part(" 'x' "), Ok(Token::Character((1, 4))));
    assert_eq!(scan_str_part("'x'"), Ok(Token::Character((0, 3))));
}

#[test]
fn scan_local_id() {
    assert_eq!(scan_str_part(" fo1 "), Ok(Token::LocalId((1, 4))));
    assert_eq!(scan_str_part("fo1"), Ok(Token::LocalId((0, 3))));
}

#[test]
fn scan_local_id2() {
    assert_eq!(scan_str_part(" fo1+ "), Ok(Token::LocalId((1, 4))));
}

#[test]
fn scan_binary_op() {
    assert_eq!(
        scan_str(" foo++bar "),
        vec![
            Ok(Token::LocalId((1, 4))),
            Ok(Token::Sigil((4, 6))),
            Ok(Token::LocalId((6, 9))),
        ]
    )
}

#[test]
fn scan_annotations() {
    assert_eq!(
        scan_str("foo<Foo>+bar<Bar>"),
        vec![
            Ok(Token::LocalId((0, 3))),
            Ok(Token::Annotation((3, 8))),
            Ok(Token::Sigil((8, 9))),
            Ok(Token::LocalId((9, 12))),
            Ok(Token::Annotation((12, 17))),
        ]
    )
}

#[test]
fn scan_global_id() {
    assert_eq!(scan_str_part(" Fo1 "), Ok(Token::GlobalId((1, 4))));
}

#[test]
fn scan_global_id2() {
    assert_eq!(scan_str_part(" Fo1+ "), Ok(Token::GlobalId((1, 4))));
}

#[test]
fn scan_decimal() {
    assert_eq!(scan_str_part(" 1xx "), Ok(Token::Number((1, 4))))
}

#[test]
fn scan_sigil() {
    assert_eq!(scan_str_part(" + "), Ok(Token::Sigil((1, 2))))
}

#[test]
fn scan_sigil2() {
    assert_eq!(scan_str_part(" +foo "), Ok(Token::Sigil((1, 2))))
}

#[test]
fn scan_keyword() {
    assert_eq!(scan_str_part(" foo: "), Ok(Token::Keyword((1, 5))))
}

#[test]
fn scan_keywords() {
    assert_eq!(
        scan_str(" foo: 42 bar: 123 "),
        vec![
            Ok(Token::Keyword((1, 5))),
            Ok(Token::Number((6, 8))),
            Ok(Token::Keyword((9, 13))),
            Ok(Token::Number((14, 17))),
        ]
    );
    assert_eq!(
        scan_str(" foo:42 bar:123 "),
        vec![
            Ok(Token::Keyword((1, 5))),
            Ok(Token::Number((5, 7))),
            Ok(Token::Keyword((8, 12))),
            Ok(Token::Number((12, 15))),
        ]
    );
}

#[test]
fn scan_keyword2() {
    assert_eq!(scan_str_part(" : "), Ok(Token::Keyword((1, 2))))
}

#[test]
fn scan_open_paren() {
    assert_eq!(scan_str_part(" ((  "), Ok(Token::OpenParen((1, 2))))
}

#[test]
fn scan_close_paren() {
    assert_eq!(scan_str_part(" )) "), Ok(Token::CloseParen((1, 2))))
}

#[test]
fn scan_open_brace() {
    assert_eq!(scan_str_part(" {{ "), Ok(Token::OpenBrace((1, 2))))
}

#[test]
fn scan_close_brace() {
    assert_eq!(scan_str_part(" }} "), Ok(Token::CloseBrace((1, 2))))
}

#[test]
fn scan_open_bracket() {
    assert_eq!(scan_str_part(" [[ "), Ok(Token::OpenBracket((1, 2))))
}

#[test]
fn scan_close_bracket() {
    assert_eq!(scan_str_part(" ]] "), Ok(Token::CloseBracket((1, 2))))
}

struct StringStream<'a> {
    source: &'a str,
    indices: std::cell::RefCell<std::str::CharIndices<'a>>,
    cache: std::cell::RefCell<Vec<(usize, char)>>,
}

impl<'a> StringStream<'a> {
    fn new(source: &'a str) -> StringStream<'a> {
        StringStream {
            source,
            indices: std::cell::RefCell::new(source.char_indices()),
            cache: std::cell::RefCell::new(Vec::new()),
        }
    }
}

impl<'a> Stream for StringStream<'a> {
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
            .ok_or(SyntaxError::new((self.len(), self.len()), "Unexpected EOF"))
    }
}
