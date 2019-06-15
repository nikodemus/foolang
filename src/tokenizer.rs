struct Position {
    line: usize,
    pos: usize,
}

enum Token {
    Comment(Position, String),
    Identifier(Position, usize),
}

struct Tokenization {
    tokens: Vec<Token>,
    symbols: Vec<String>,
}

impl Tokenization {
    fn new() -> Tokenization {
        Tokenization { tokens: Vec::new(), symbols: Vec::new() }
    }
    fn add_comment(&mut self, position: Position, comment: String) {
        tokens.push(Token::Comment(position, comment));
    }
}

struct ParseError {
    position: Position,
    problem: &'static str,
}

struct Grammar {
    comment: &'static str,
}

impl Grammar {
    fn tokenize(&self, input: &mut Stream) -> Result<Tokenization, ParseError> {
        let mut tokenization = Tokenization::new();
        while self.token(input, &mut tokenization)? {}
        Ok(tokenization)
    }
    fn token(&self, input: &mut Stream, tokenization: &mut Tokenization) -> Result<bool, ParseError> {
        let position = input.position();
        if self.at_comment(input) {
            tokenization.add_comment(position, read_comment(input));
            return Ok(true);
        }
        return Err(ParseError {
            position, problem: "Invalid token"
        });
    }
    fn at_comment(&self, input: &mut Stream) -> bool {
        for (index, ch) in self.comment.char_indices() {
            if ch != input.peek(index) {
                return false;
            }
        }
        true
    }
}

fn tokenize(input: &mut Stream) -> Result<Tokenization, ParseError> {
    let grammar = Grammar {
        comment: "#",
    }
    grammar.tokenize(input)
}



fn read_comment(input: &mut Stream) -> String {
    let start = input.index();
    while !input.at_eol_or_eof() {
        input.skip();
    }
    // Eat up the newline, don't sweat about EOF.
    input.try_next();
    String::from(input.slice(start, input.index()));
}
