use crate::source_location::Span;
use crate::unwind::Unwind;

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum Token {
    EOF,
    HEX_INTEGER,
    BIN_INTEGER,
    DEC_INTEGER,
    SINGLE_FLOAT,
    DOUBLE_FLOAT,
    BLOCK_COMMENT,
    COMMENT,
    BLOCK_STRING,
    STRING,
    WORD,
    KEYWORD,
    SIGIL,
}

#[cfg(test)]
impl Token {
    pub(crate) fn name(&self) -> String {
        format!("{:?}", self)
    }
}

pub struct TokenStream<'a> {
    source: &'a str,
    indices: std::cell::RefCell<std::str::CharIndices<'a>>,
    span: Span,
    current: (usize, char),
    offset: usize,
}

impl<'a> TokenStream<'a> {
    pub(crate) fn new(source: &'a str) -> TokenStream<'a> {
        let mut stream = TokenStream {
            source,
            indices: std::cell::RefCell::new(source.char_indices()),
            span: 0..0,
            current: (0, '.'),
            offset: 0,
        };
        if source.len() > 0 {
            // println!("non-zero source, init to first char");
            stream.next();
        } else {
            // println!("zero source, init to eof");
        }
        return stream;
    }

    #[cfg(test)]
    pub(crate) fn slice(&self) -> &str {
        &self.source[self.span()]
    }

    pub(crate) fn slice_at(&self, span: Span) -> &str {
        &self.source[span]
    }

    pub(crate) fn span(&self) -> Span {
        self.span.clone()
    }

    fn len(&self) -> usize {
        self.source.len()
    }

    fn pos(&self) -> usize {
        self.current.0
    }

    fn character(&self) -> char {
        self.current.1
    }

    fn result(&mut self, token: Token, span: Span) -> Result<Token, Unwind> {
        self.span = span;
        Ok(token)
    }

    // Implements the algorithm from Tokenization.md
    pub(crate) fn scan(&mut self) -> Result<Token, Unwind> {
        //
        // 1. If at end of file, return EOF.
        //
        if self.at_eof() {
            // println!("scan 1: eof");
            // println!("=> eof");
            return self.result(Token::EOF, self.len()..self.len());
        }
        //
        // 2. If at whitespace, consume it, then continue from 1.
        //
        if self.at_whitespace() {
            while self.at_whitespace() {
                self.next();
            }
            return self.scan();
        }
        //
        // 3. If at a special character, consume it and return SIGIL.
        //
        if self.at_special() {
            // println!("scan 3: special");
            let start = self.next();
            // println!("=> special");
            return self.result(Token::SIGIL, start..self.pos());
        }
        //
        // 4. If at a digit character, consume it. Then continue as below,
        //    returning the appropriate type of number token.
        //
        if self.at_digit(10) {
            // println!("scan 4: number");
            return self.scan_number();
        }
        //
        // 5. If at --- consume until ---- and return BLOCK_COMMENT.
        //
        if self.at_str("---") {
            // println!("scan 5: block comment");
            let start = self.consume("---");
            while !self.at_str("---") {
                if self.at_eof() {
                    return self.result(Token::EOF, self.len()..self.len());
                } else {
                    self.next();
                }
            }
            self.consume("---");
            // println!("=> block comment");
            return self.result(Token::BLOCK_COMMENT, start..self.pos());
        }
        //
        // 6. If at -- consume to end of line and return COMMENT.
        //
        if self.at_str("--") {
            // println!("scan 6: comment");
            let start = self.consume("--");
            while !self.at_newline() {
                self.next();
            }
            // println!("=> comment");
            return self.result(Token::COMMENT, start..self.pos());
        }
        //
        // 7. If at """ consume until non-escaped """ and return BLOCK_STRING.
        //
        if self.at_str(r#"""""#) {
            // println!("scan 7: block string");
            let start = self.consume(r#"""""#);
            while !self.at_str(r#"""""#) {
                self.next();
                if self.at_str("\\") {
                    // Since \ consumes two, \"""" will not match
                    self.next();
                    self.next();
                }
            }
            self.consume(r#"""""#);
            // println!("=> block string");
            return self.result(Token::BLOCK_STRING, start..self.pos());
        }
        //
        //  8. If at " consume until non-escaped " and return STRING.
        //
        if self.at_str(r#"""#) {
            // println!("scan 8: string");
            let start = self.consume(r#"""#);
            while !self.at_str(r#"""#) {
                // println!("s: '{}'", &self.source[self.pos()..self.pos() + 1]);
                if self.at_str("\\") {
                    // Since \ consumes two, \" will not match
                    // println!("q1: '{}'", &self.source[self.pos()..self.pos() + 1]);
                    self.next();
                    // println!("q2: '{}'", &self.source[self.pos()..self.pos() + 1]);
                    self.next();
                } else {
                    self.next();
                }
            }
            self.consume(r#"""#);
            // println!("=> string");
            return self.result(Token::STRING, start..self.pos());
        }
        //
        // 9. If at a word character, until eof or non-word character.
        //
        if self.at_word() {
            let start = self.next();
            loop {
                // println!("scan 9: word or keyword");
                while self.at_word() {
                    self.next();
                }
                // 9.1 If the word is immediately followed by a single
                //     colon (ie. not double) and whitespace, consume the
                //     colon and return KEYWORD.
                if self.at_char(':') {
                    // println!("scan 9.1: word followed by colon");
                    let pos = self.next();
                    if self.at_char(':') {
                        self.reset(pos);
                    } else {
                        // println!("=> keyword");
                        return self.result(Token::KEYWORD, start..self.pos());
                    }
                }
                // 9.2 If the word is immediately followed by a dot
                //     followed by a word character consume the dot and
                //     continue parsing the word at step 9.
                if self.at_char('.') {
                    // println!("scan 9.2: word followed by dot");
                    let pos = self.next();
                    if self.at_word() {
                        continue;
                    } else {
                        self.reset(pos);
                    }
                }
                // println!("=> word {}", &self.source[start..self.pos()]);
                return self.result(Token::WORD, start..self.pos());
            }
        }
        //
        // 10. At a sigil character, consume the sigil and return SIGIL.
        //
        // println!("scan 10: sigil");
        assert!(self.at_sigil());
        let start = self.next();
        while self.at_sigil() {
            self.next();
        }
        // println!("=> sigil");
        return self.result(Token::SIGIL, start..self.pos());
    }

    fn scan_number(&mut self) -> Result<Token, Unwind> {
        let start = self.next();
        //
        // 4.1. If at x or X, consume word characters, return HEX_INTEGER.
        //
        if self.at_char('x') || self.at_char('X') {
            self.next();
            while self.at_word() {
                self.next();
            }
            // println!("=> hex number");
            return self.result(Token::HEX_INTEGER, start..self.pos());
        }
        //
        // 4.2. If at b or B, consume word characters, return BIN_INTEGER.
        //
        if self.at_char('b') || self.at_char('B') {
            self.next();
            while self.at_word() {
                self.next();
            }
            // println!("=> binary number");
            return self.result(Token::BIN_INTEGER, start..self.pos());
        }
        //
        // 4.3. Consume decimal digits and underscore.
        //
        while self.at_digit(10) || self.at_char('_') {
            self.next();
        }
        let dot = self.at_char('.');
        if dot {
            //
            // 4.4. If at dot: if followed by whitespace return DEC_INTEGER,
            //      otherwise consume the dot and the digits plus possible
            //      underscores.
            //
            let p = self.pos();
            self.next();
            if self.at_whitespace() {
                self.reset(p);
                return self.result(Token::DEC_INTEGER, start..self.pos());
            }
            while self.at_digit(10) || self.at_char('_') {
                self.next();
            }
        }
        //
        // 4.5. If at e or f, consume. If at + or -, consume. Consume word
        //      characters. For e return DOUBLE_FLOAT, for f return
        //      SINGLE_FLOAT.
        //
        let single = self.at_char('f') || self.at_char('F');
        let double = self.at_char('e') || self.at_char('E');
        if single || double {
            self.next();
            if self.at_char('+') || self.at_char('-') {
                self.next();
            }
            while self.at_word() {
                self.next();
            }
            if single {
                return self.result(Token::SINGLE_FLOAT, start..self.pos());
            } else {
                return self.result(Token::DOUBLE_FLOAT, start..self.pos());
            }
        }
        //
        // 4.6. Consume word characters. If consumed a dot earlier, return
        //      DOUBLE_FLOAT, otherwise DEC_INTEGER.
        //
        while self.at_word() {
            self.next();
        }
        if dot {
            return self.result(Token::DOUBLE_FLOAT, start..self.pos());
        } else {
            return self.result(Token::DEC_INTEGER, start..self.pos());
        }
    }

    fn at_eof(&self) -> bool {
        self.pos() >= self.len()
    }

    fn at_whitespace(&self) -> bool {
        !self.at_eof() && self.character().is_whitespace()
    }

    fn at_alphanumeric(&self) -> bool {
        !self.at_eof() && self.character().is_alphanumeric()
    }

    fn at_digit(&self, base: u32) -> bool {
        !self.at_eof() && self.character().is_digit(base)
    }

    fn at_newline(&self) -> bool {
        self.at_char('\n')
    }

    fn at_special(&self) -> bool {
        if self.at_eof() {
            return false;
        }
        let c = self.character();
        return c == '('
            || c == ')'
            || c == '['
            || c == ']'
            || c == '{'
            || c == '}'
            || c == ','
            || c == '.'
            || c == ';'
            || c == '$'
            || c == '!'
            || c == '#';
    }

    fn at_terminating(&self) -> bool {
        self.at_whitespace() || self.at_special()
    }

    fn at_word(&self) -> bool {
        self.at_alphanumeric() || self.at_char('_')
    }

    fn at_sigil(&self) -> bool {
        !(self.at_eof() || self.at_word() || self.at_terminating())
    }

    fn at_char(&self, c: char) -> bool {
        !self.at_eof() && c == self.character()
    }

    fn at_str(&self, target: &str) -> bool {
        let start = self.pos();
        let end = start + target.len();
        if self.len() < end {
            return false;
        }
        &self.source[start..end] == target
    }

    fn next(&mut self) -> usize {
        let p = self.pos();
        self.current = match self.indices.borrow_mut().next() {
            Some((p, ch)) => (p + self.offset, ch),
            None => (self.len(), '.'),
        };
        return p;
    }

    fn consume(&mut self, target: &str) -> usize {
        assert!(self.at_str(target));
        let p = self.pos();
        self.reset(p + target.len());
        return p;
    }

    fn reset(&mut self, position: usize) {
        self.offset = position;
        self.indices = std::cell::RefCell::new(self.source[position..].char_indices());
        self.next();
    }
}
