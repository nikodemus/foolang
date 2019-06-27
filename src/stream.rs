type Span = (usize, usize);

#[derive(Debug, PartialEq)]
enum Token {
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

trait Stream {
    fn scan(&mut self) -> Result<Token, SyntaxError> {
        let mut start;
        loop {
            if self.at_eof() {
                return Ok(Token::Eof((self.len(), self.len())));
            }
            start = self.getchar();
            if !start.1.is_whitespace() {
                break;
            }
        }
        let alphanumeric = start.1.is_alphanumeric();
        let mut keyword = start.1 == ':';
        let mut end = self.getchar();
        loop {
            if end.1.is_whitespace() || self.at_eof() {
                break;
            }
            if end.1 == ':' {
                keyword = true;
            } else if alphanumeric != end.1.is_alphanumeric() {
                break;
            }
            if DELIMITERS.iter().find(|x| **x == end.1).is_some() {
                break;
            }
            end = self.getchar();
        }
        let span = (start.0, end.0);
        if keyword {
            return Ok(Token::Keyword(span));
        }
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
    /*
    fn scan_character(&mut self) -> Result<Token, SyntaxError> {
        let start = self.position();
        let mut indices = self.str()[start..].char_indices();
        indices.next();
        if indices.next().is_none() {
            return Err(SyntaxError::new((start, start), "Unexpected end of file"));
        }
        let end = match indices.next() {
            None => return Err(SyntaxError::new((start, start), "Unexpected end of file")),
            Some((pos, mark)) => {
                if mark != '\'' {
                    return Err(SyntaxError::new(
                        (start, start + pos),
                        "Malformed character literal",
                    ));
                }
                self.seek(start + pos + 1)
            }
        };
        Ok(Token::Character((start, end)))
    }
    */
    // --
    fn at_eof(&self) -> bool;
    fn getchar(&mut self) -> (usize, char);
    fn len(&self) -> usize;
}

fn scan_str(s: &str) -> Result<Token, SyntaxError> {
    StringStream::new(s).scan()
}

#[test]
fn scan_eof() {
    assert_eq!(scan_str("   "), Ok(Token::Eof((3, 3))));
}

#[ignore]
#[test]
fn scan_char() {
    assert_eq!(scan_str(" 'x' "), Ok(Token::Character((1, 4))));
}

#[test]
fn scan_local_id() {
    assert_eq!(scan_str(" fo1 "), Ok(Token::LocalId((1, 4))));
}

#[test]
fn scan_local_id2() {
    assert_eq!(scan_str(" fo1+ "), Ok(Token::LocalId((1, 4))));
}

#[test]
fn scan_global_id() {
    assert_eq!(scan_str(" Fo1 "), Ok(Token::GlobalId((1, 4))));
}

#[test]
fn scan_global_id2() {
    assert_eq!(scan_str(" Fo1+ "), Ok(Token::GlobalId((1, 4))));
}

#[test]
fn scan_decimal() {
    assert_eq!(scan_str(" 1xx "), Ok(Token::Number((1, 4))))
}

#[test]
fn scan_sigil() {
    assert_eq!(scan_str(" + "), Ok(Token::Sigil((1, 2))))
}

#[test]
fn scan_sigil2() {
    assert_eq!(scan_str(" +foo "), Ok(Token::Sigil((1, 2))))
}

#[test]
fn scan_keyword() {
    assert_eq!(scan_str(" foo: "), Ok(Token::Keyword((1, 5))))
}

#[test]
fn scan_keyword2() {
    assert_eq!(scan_str(" : "), Ok(Token::Keyword((1, 2))))
}

#[test]
fn scan_keyword3() {
    assert_eq!(scan_str(" ::: "), Ok(Token::Keyword((1, 4))))
}

#[test]
fn scan_open_paren() {
    assert_eq!(scan_str(" ((  "), Ok(Token::OpenParen((1, 2))))
}

#[test]
fn scan_close_paren() {
    assert_eq!(scan_str(" )) "), Ok(Token::CloseParen((1, 2))))
}

#[test]
fn scan_open_brace() {
    assert_eq!(scan_str(" {{ "), Ok(Token::OpenBrace((1, 2))))
}

#[test]
fn scan_close_brace() {
    assert_eq!(scan_str(" }} "), Ok(Token::CloseBrace((1, 2))))
}

#[test]
fn scan_open_bracket() {
    assert_eq!(scan_str(" [[ "), Ok(Token::OpenBracket((1, 2))))
}

#[test]
fn scan_close_bracket() {
    assert_eq!(scan_str(" ]] "), Ok(Token::CloseBracket((1, 2))))
}

struct StringStream<'a> {
    source: &'a str,
    indices: std::cell::RefCell<std::str::CharIndices<'a>>,
    cache: std::cell::RefCell<Option<(usize, char)>>,
}

impl<'a> StringStream<'a> {
    fn new(source: &'a str) -> StringStream<'a> {
        StringStream {
            source,
            indices: std::cell::RefCell::new(source.char_indices()),
            cache: std::cell::RefCell::new(None),
        }
    }
}

impl<'a> Stream for StringStream<'a> {
    fn len(&self) -> usize {
        self.source.len()
    }
    fn at_eof(&self) -> bool {
        match self.indices.borrow_mut().next() {
            None => return true,
            Some(place) => {
                assert_eq!(None, self.cache.replace(Some(place)));
                return false;
            }
        }
    }
    fn getchar(&mut self) -> (usize, char) {
        if let Some((pos, c)) = self.cache.replace(None) {
            return (pos, c);
        }
        self.indices
            .borrow_mut()
            .next()
            .expect("Unexpected EOF in StringStream::getchar()")
    }
}
