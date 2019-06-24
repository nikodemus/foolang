use regex::Regex;
use std::borrow::ToOwned;
use std::collections::VecDeque;

#[derive(PartialEq)]
pub struct ParseError {
    pub position: usize,
    pub problem: &'static str,
    pub context: String,
}

impl std::fmt::Debug for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "ParseError({}) at {}:\n{}",
            self.problem, self.position, self.context
        )
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Decimal(i64),
    Float(f64),
    Selector(String),
    Character(char),
    String(String),
    Record(Vec<String>, Vec<Literal>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Message {
    pub selector: String,
    pub arguments: Vec<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Constant(Literal),
    Variable(String),
    // Explicit receiver for chains means that we can always trivially access
    // the original receiver, instead of having to look for it in a tree of sends.
    Chain(Box<Expr>, Vec<Message>),
    // Each vector of messages is a separate chain using the original receiver.
    Cascade(Box<Expr>, Vec<Vec<Message>>),
    Sequence(Vec<Expr>),
    Block(Vec<String>, Box<Expr>),
    Array(Vec<Expr>),
    Bind(String, Box<Expr>, Box<Expr>),
    Assign(String, Box<Expr>),
    Return(Box<Expr>),
    Type(String, Box<Expr>),
}

impl Expr {
    fn literal(self) -> Literal {
        match self {
            Expr::Constant(lit) => lit,
            _ => panic!("Not a constant: {:?}", self),
        }
    }
    fn is_literal(&self) -> bool {
        match self {
            Expr::Constant(_) => true,
            _ => false,
        }
    }
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
    InterpolatedString(String),
    Identifier(String),
    Keyword(String),
    Operator(String),
    Type(String),
    PositionalParameter(String),
    // true = comma, false = newline
    Sequence(bool),
    Chain(),
    Cascade(),
    BlockBegin(),
    BlockEnd(),
    ParenBegin(),
    ParenEnd(),
    ArrayBegin(),
    ArrayEnd(),
    Bind(),
    Assign(),
    Return(),
    LiteralRecordBegin(),
}

impl Token {
    fn str(&self) -> &str {
        match &self.info {
            TokenInfo::Operator(name) => name.as_str(),
            TokenInfo::Keyword(name) => name.as_str(),
            TokenInfo::Type(name) => name.as_str(),
            TokenInfo::InterpolatedString(string) => string.as_str(),
            _ => panic!("Token has no string content: {:?}", self),
        }
    }
    fn precedence(&self) -> usize {
        match &self.info {
            TokenInfo::Eof() => 0,
            TokenInfo::LiteralRecordBegin() => 0,
            TokenInfo::BlockBegin() => 0,
            TokenInfo::BlockEnd() => 0,
            TokenInfo::ParenBegin() => 0,
            TokenInfo::ParenEnd() => 0,
            TokenInfo::Bind() => 1,
            TokenInfo::Return() => 2,
            TokenInfo::Assign() => 2,
            TokenInfo::Sequence(_) => 2,
            TokenInfo::ArrayBegin() => 2,
            TokenInfo::ArrayEnd() => 2,
            TokenInfo::Cascade() => 3,
            TokenInfo::Chain() => 4,
            TokenInfo::Keyword(_) => 5,
            // 10 is fallback for unknown operators
            TokenInfo::Operator(op) => match op.as_str() {
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
}

fn parse_stream(stream: Box<Stream>, allow_incomplete: bool) -> Result<(Expr, usize), ParseError> {
    Parser::new(stream).parse(allow_incomplete)
}

fn parse_string(string: String, allow_incomplete: bool) -> Result<(Expr, usize), ParseError> {
    parse_stream(Box::new(StringStream::new(string)), allow_incomplete)
}

pub fn parse_str(str: &str) -> Result<Expr, ParseError> {
    match parse_string(str.to_string(), false) {
        Ok((expr, _)) => Ok(expr),
        Err(err) => Err(err),
    }
}

fn partial_parse_str(str: &str) -> Result<(Expr, usize), ParseError> {
    parse_string(str.to_string(), true)
}

struct Parser {
    stream: Box<Stream>,
    lookahead: VecDeque<Token>,
    cascade: Option<Expr>,
    interpolated_block_string_re: Regex,
    interpolated_string_re: Regex,
    literal_block_string_re: Regex,
    literal_string_re: Regex,
    literal_record_re: Regex,
    character_re: Regex,
    selector_re: Regex,
    type_re: Regex,
    return_re: Regex,
    bind_re: Regex,
    assign_re: Regex,
    positional_parameter_re: Regex,
    array_begin_re: Regex,
    array_end_re: Regex,
    block_begin_re: Regex,
    block_end_re: Regex,
    paren_begin_re: Regex,
    paren_end_re: Regex,
    cont_re: Regex,
    sequence_re: Regex,
    chain_re: Regex,
    cascade_re: Regex,
    decimal_re: Regex,
    float_re: Regex,
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
            positional_parameter_re: Regex::new(r"^:[_a-zA-Z][_a-zA-z0-9]*").unwrap(),
            selector_re: Regex::new(r"^\$[_a-zA-Z][_a-zA-Z0-9]*(:[_a-zA-Z][_a-zA-Z0-9]*:)*")
                .unwrap(),
            literal_record_re: Regex::new(r#"^\$\{"#).unwrap(),
            literal_block_string_re: Regex::new(r#"^\$""""#).unwrap(),
            literal_string_re: Regex::new(r#"^\$""#).unwrap(),
            interpolated_block_string_re: Regex::new(r#"^""""#).unwrap(),
            interpolated_string_re: Regex::new(r#"^""#).unwrap(),
            character_re: Regex::new(r"^'.'").unwrap(),
            type_re: Regex::new("^<[A-Z][a-zA-Z0-9]*>").unwrap(),
            return_re: Regex::new("^return ").unwrap(),
            bind_re: Regex::new(r"^let ").unwrap(),
            assign_re: Regex::new(r"^:=").unwrap(),
            array_begin_re: Regex::new(r"^\[").unwrap(),
            array_end_re: Regex::new(r"^\]").unwrap(),
            block_begin_re: Regex::new(r"^\{").unwrap(),
            block_end_re: Regex::new(r"^\}").unwrap(),
            paren_begin_re: Regex::new(r"^\(").unwrap(),
            paren_end_re: Regex::new(r"^\)").unwrap(),
            cont_re: Regex::new(r"^\\").unwrap(),
            sequence_re: Regex::new(r"^,").unwrap(),
            chain_re: Regex::new(r"^--").unwrap(),
            cascade_re: Regex::new(r"^;").unwrap(),
            decimal_re: Regex::new(r"^[0-9]+").unwrap(),
            float_re: Regex::new(r"^[0-9]+\.[0-9]+").unwrap(),
            operator_re: Regex::new(r"^[\-+*/%<>=^|&!\?]+").unwrap(),
            keyword_re: Regex::new(r"^[_a-zA-Z][_a-zA-Z0-9]*:").unwrap(),
            identifier_re: Regex::new(r"^[_a-zA-Z][_a-zA-Z0-9]*").unwrap(),
        }
    }
    fn parse(&mut self, allow_incomplete: bool) -> Result<(Expr, usize), ParseError> {
        let expr = self.parse_expression(0)?;
        let pos = self.peek_token()?.position;
        if self.stream.at_eof() || allow_incomplete {
            Ok((expr, pos))
        } else {
            Err(self.error_at(pos, "Incomplete parse"))
        }
    }
    fn parse_expression(&mut self, precedence: usize) -> Result<Expr, ParseError> {
        let mut expr = self.parse_prefix()?;
        while precedence < self.next_precedence()? {
            expr = self.parse_suffix(expr)?;
        }
        Ok(expr)
    }
    fn parse_prefix(&mut self) -> Result<Expr, ParseError> {
        let token = self.consume_token()?;
        match token.info {
            TokenInfo::Constant(literal) => Ok(Expr::Constant(literal)),
            TokenInfo::Identifier(name) => Ok(Expr::Variable(name)),
            TokenInfo::Return() => self.parse_prefix_return(token),
            TokenInfo::Cascade() => self.parse_prefix_cascade(token),
            TokenInfo::ArrayBegin() => self.parse_prefix_array(token),
            TokenInfo::LiteralRecordBegin() => self.parse_prefix_literal_record(token),
            TokenInfo::BlockBegin() => self.parse_prefix_block(token),
            TokenInfo::ParenBegin() => self.parse_prefix_paren(token),
            TokenInfo::Operator(_) => self.parse_prefix_operator(token),
            TokenInfo::Bind() => self.parse_prefix_bind(token),
            TokenInfo::InterpolatedString(_) => self.parse_prefix_interpolated_string(token),
            // Leading newline, ignore.
            TokenInfo::Sequence(false) => self.parse_prefix(),
            TokenInfo::Eof() => self.error(token, "Unexpected end of input"),
            _ => self.error(token, "Not a value expression"),
        }
    }
    fn parse_suffix(&mut self, left: Expr) -> Result<Expr, ParseError> {
        let token = self.consume_token()?;
        match token.info {
            TokenInfo::Identifier(name) => Ok(left.unary(name)),
            TokenInfo::Chain() => Ok(left),
            TokenInfo::Type(_) => self.parse_suffix_type(left, token),
            TokenInfo::Assign() => self.parse_suffix_assign(left, token),
            TokenInfo::Sequence(_) => self.parse_suffix_sequence(left, token),
            TokenInfo::Cascade() => self.parse_suffix_cascade(left, token),
            TokenInfo::Operator(_) => self.parse_suffix_operator(left, token),
            TokenInfo::Keyword(_) => self.parse_suffix_keyword(left, token),
            TokenInfo::Eof() => self.error(token, "Unexpected end of input"),
            _ => self.error(token, "Not valid in suffix position."),
        }
    }
    fn parse_prefix_array(&mut self, token: Token) -> Result<Expr, ParseError> {
        let mut content = vec![];
        loop {
            let next = self.peek_token()?;
            match next.info {
                TokenInfo::ArrayEnd() => {
                    self.consume_token();
                    break;
                }
                _ => {}
            }
            content.push(self.parse_expression(token.precedence())?);
            let next = self.consume_token()?;
            match next.info {
                TokenInfo::ArrayEnd() => break,
                TokenInfo::Sequence(true) => {}
                _ => return self.error(next, "Malformed array"),
            }
        }
        Ok(Expr::Array(content))
    }
    fn parse_prefix_bind(&mut self, token: Token) -> Result<Expr, ParseError> {
        let var = self.consume_token()?;
        if let TokenInfo::Identifier(name) = var.info {
            let assign = self.consume_token()?;
            if let TokenInfo::Assign() = assign.info {
                let value = self.parse_expression(assign.precedence())?;
                let seq = self.consume_token()?;
                if let TokenInfo::Sequence(_) = seq.info {
                    let body = self.parse_expression(token.precedence())?;
                    Ok(Expr::Bind(
                        name.to_string(),
                        Box::new(value),
                        Box::new(body),
                    ))
                } else {
                    self.error(seq, "Expected sequencing operator")
                }
            } else {
                self.error(assign, "Expected assignment operator")
            }
        } else {
            self.error(var, "Expected variable name")
        }
    }
    fn parse_prefix_block(&mut self, token: Token) -> Result<Expr, ParseError> {
        if let TokenInfo::Keyword(_) = self.peek_token()?.info {
            return self.parse_prefix_record(token);
        }
        let mut args = vec![];
        loop {
            let next = self.peek_token()?;
            match next.info {
                TokenInfo::PositionalParameter(name) => {
                    args.push(name);
                    self.consume_token()?;
                    continue;
                }
                TokenInfo::Operator(ref op) if "|" == op.as_str() => {
                    self.consume_token()?;
                    break;
                }
                _ => {}
            }
            if args.is_empty() {
                break;
            }
            return self.error(next, "Malformed block argument");
        }
        let expr = self.parse_expression(token.precedence())?;
        if let TokenInfo::BlockEnd() = self.consume_token()?.info {
            Ok(Expr::Block(args, Box::new(expr)))
        } else {
            self.error(token, "Block not closed")
        }
    }
    fn parse_prefix_cascade(&mut self, token: Token) -> Result<Expr, ParseError> {
        match &self.cascade {
            None => self.error(token, "Malformed cascade"),
            Some(expr) => {
                let receiver = expr.to_owned();
                self.cascade = None;
                Ok(receiver)
            }
        }
    }
    fn parse_prefix_record_aux(
        &mut self,
        require_literal: bool,
    ) -> Result<(Vec<String>, Vec<Expr>), ParseError> {
        let mut names = vec![];
        let mut values = vec![];
        loop {
            let key = self.consume_token()?;
            match key.info {
                TokenInfo::Keyword(name) => {
                    names.push(name);
                }
                TokenInfo::BlockEnd() => {
                    break;
                }
                _ => {
                    return Err(self.error_at(key.position, "Not a keyword"));
                }
            }
            let next = self.peek_token()?;
            // FIXME: hardcoded precedence
            let expr = self.parse_expression(2)?;
            if require_literal && !expr.is_literal() {
                return Err(self.error_at(next.position, "Not a literal"));
            }
            values.push(expr);
            let term = self.consume_token()?;
            match term.info {
                TokenInfo::BlockEnd() => break,
                TokenInfo::Sequence(_) => continue,
                _ => return Err(self.error_at(term.position, "Malformed record")),
            }
        }
        Ok((names, values))
    }
    fn parse_prefix_literal_record(&mut self, _: Token) -> Result<Expr, ParseError> {
        let (names, values) = self.parse_prefix_record_aux(true)?;
        let literals = values.into_iter().map(|x| x.literal()).collect();
        Ok(Expr::Constant(Literal::Record(names, literals)))
    }
    fn parse_prefix_interpolated_string(&mut self, token: Token) -> Result<Expr, ParseError> {
        fn append(expr: Expr, string: &str) -> Expr {
            if string.is_empty() {
                return expr;
            }
            match expr {
                Expr::Constant(Literal::String(mut orig)) => {
                    orig.push_str(string);
                    Expr::Constant(Literal::String(orig))
                }
                _ => expr.keyword(
                    "append:".to_string(),
                    vec![Expr::Constant(Literal::String(string.to_string()))],
                ),
            }
        }
        let mut stream = StringStream::new(token.str().to_string());
        let mut expr = Expr::Constant(Literal::String("".to_string()));
        let mut start = 0;
        loop {
            if stream.at_eof() {
                expr = append(expr, &stream.as_str()[start..stream.position()]);
                break;
            }
            if stream.charstr() == "{" {
                expr = append(expr, &stream.as_str()[start..stream.position()]);
                stream.skip(1);
                let (part, end) = match partial_parse_str(stream.str()) {
                    Ok(res) => res,
                    Err(err) => {
                        return Err(self.error_at(stream.position() + err.position, err.problem));
                    }
                };
                let pos = stream.position() + end;
                expr = match expr {
                    Expr::Constant(Literal::String(ref s)) if s.is_empty() => {
                        part.unary("toString".to_string())
                    }
                    _ => expr.keyword(
                        "append:".to_string(),
                        vec![part.unary("toString".to_string())],
                    ),
                };
                stream.seek(pos);
                if stream.charstr() != "}" {
                    return Err(self.error_at(start, "Unterminated interpolation"));
                }
                start = pos + 1;
            }
            stream.skip(1);
        }
        Ok(expr)
    }
    fn parse_prefix_operator(&mut self, token: Token) -> Result<Expr, ParseError> {
        let message = match token.str() {
            "-" => "neg",
            _ => return self.error(token, "Not a prefix operator"),
        };
        let expr = self.parse_expression(token.precedence())?;
        Ok(expr.unary(message.to_string()))
    }
    fn parse_prefix_paren(&mut self, token: Token) -> Result<Expr, ParseError> {
        let expr = self.parse_expression(token.precedence())?;
        let next = self.consume_token()?;
        match next.info {
            TokenInfo::ParenEnd() => Ok(expr),
            _ => self.error(token, "Unmatched close parenthesis"),
        }
    }
    fn parse_prefix_record(&mut self, _: Token) -> Result<Expr, ParseError> {
        let (names, arguments) = self.parse_prefix_record_aux(true)?;
        Ok(Expr::Chain(
            Box::new(Expr::Variable("Record".to_string())),
            vec![Message {
                selector: names.join(""),
                arguments,
            }],
        ))
    }
    fn parse_prefix_return(&mut self, token: Token) -> Result<Expr, ParseError> {
        let value = self.parse_expression(token.precedence())?;
        Ok(Expr::Return(Box::new(value)))
    }
    fn parse_suffix_assign(&mut self, left: Expr, token: Token) -> Result<Expr, ParseError> {
        if let Expr::Variable(name) = left {
            Ok(Expr::Assign(
                name,
                Box::new(self.parse_expression(token.precedence())?),
            ))
        } else {
            self.error(token, "Invalid target for assignment")
        }
    }
    fn parse_suffix_cascade(&mut self, left: Expr, token: Token) -> Result<Expr, ParseError> {
        assert_eq!(self.cascade, None);
        let mut chains = vec![];
        self.insert_token(token.clone());
        loop {
            let next = self.peek_token()?;
            if TokenInfo::Cascade() != next.info {
                break;
            }
            // Need to wrap in cascade in case this is a chain,
            // which would then accumulate the cascaded messages.
            let mark = Expr::Variable("(cascade)".to_string());
            self.cascade = Some(mark.clone());
            let chain = self.parse_expression(token.precedence())?;
            match chain {
                Expr::Chain(to, messages) => {
                    assert_eq!(mark, *to);
                    chains.push(messages);
                }
                _ => {
                    return Err(self.error_at(token.position, "Invalid cascade"));
                }
            }
        }
        Ok(Expr::Cascade(Box::new(left.clone()), chains))
    }
    fn parse_suffix_keyword(&mut self, left: Expr, token: Token) -> Result<Expr, ParseError> {
        let mut args = vec![];
        let mut selector = token.str().to_string();
        loop {
            args.push(self.parse_expression(token.precedence())?);
            if let Token {
                position: _,
                info: TokenInfo::Keyword(next),
            } = self.peek_token()?
            {
                self.consume_token()?;
                selector.push_str(next.as_str());
            } else {
                break;
            }
        }
        Ok(left.keyword(selector, args))
    }
    fn parse_suffix_operator(&mut self, left: Expr, token: Token) -> Result<Expr, ParseError> {
        let right = self.parse_expression(token.precedence())?;
        Ok(left.binary(token.str().to_string(), right))
    }
    fn parse_suffix_sequence(&mut self, left: Expr, token: Token) -> Result<Expr, ParseError> {
        let right = self.parse_expression(token.precedence())?;
        let mut expressions = if let Expr::Sequence(left_expressions) = left {
            left_expressions
        } else {
            vec![left]
        };
        if let Expr::Sequence(mut right_expressions) = right {
            expressions.append(&mut right_expressions);
        } else {
            expressions.push(right);
        }
        Ok(Expr::Sequence(expressions))
    }
    fn parse_suffix_type(&mut self, left: Expr, token: Token) -> Result<Expr, ParseError> {
        Ok(Expr::Type(token.str().to_string(), Box::new(left)))
    }
    fn next_precedence(&mut self) -> Result<usize, ParseError> {
        Ok(self.peek_token()?.precedence())
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
            self.stream.skip(1);
        }
        if let Some(_) = self.stream.scan(&self.cont_re) {
            let position = self.stream.position();
            let next = self.parse_token()?;
            if let Sequence(false) = next.info {
                return self.parse_token();
            } else {
                return Err(self.error_at(position, "End-of-line escape not at end of line"));
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
            self.stream.seek(mark);
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
        if let Some(name) = self.stream.scan(&self.type_re) {
            return Ok(Token {
                position,
                info: Type(name[1..name.len() - 1].to_string()),
            });
        }
        if let Some(_) = self.stream.scan(&self.bind_re) {
            return Ok(Token {
                position,
                info: Bind(),
            });
        }
        if let Some(_) = self.stream.scan(&self.assign_re) {
            return Ok(Token {
                position,
                info: Assign(),
            });
        }
        if let Some(_) = self.stream.scan(&self.paren_begin_re) {
            return Ok(Token {
                position,
                info: ParenBegin(),
            });
        }
        if let Some(_) = self.stream.scan(&self.paren_end_re) {
            return Ok(Token {
                position,
                info: ParenEnd(),
            });
        }
        if let Some(_) = self.stream.scan(&self.array_begin_re) {
            return Ok(Token {
                position,
                info: ArrayBegin(),
            });
        }
        if let Some(_) = self.stream.scan(&self.array_end_re) {
            return Ok(Token {
                position,
                info: ArrayEnd(),
            });
        }
        if let Some(_) = self.stream.scan(&self.block_begin_re) {
            return Ok(Token {
                position,
                info: BlockBegin(),
            });
        }
        if let Some(_) = self.stream.scan(&self.block_end_re) {
            return Ok(Token {
                position,
                info: BlockEnd(),
            });
        }
        if let Some(_) = self.stream.scan(&self.sequence_re) {
            return Ok(Token {
                position,
                info: Sequence(true),
            });
        }
        if let Some(s) = self.stream.scan(&self.interpolated_block_string_re) {
            return self.scan_interpolated_block_string(s);
        }
        if let Some(s) = self.stream.scan(&self.interpolated_string_re) {
            return self.scan_interpolated_string(s);
        }
        if let Some(s) = self.stream.scan(&self.literal_block_string_re) {
            return self.scan_literal_block_string(s);
        }
        if let Some(s) = self.stream.scan(&self.literal_string_re) {
            return self.scan_literal_string(s);
        }
        if let Some(s) = self.stream.scan(&self.literal_record_re) {
            return self.scan_literal_record(s);
        }
        if let Some(string) = self.stream.scan(&self.character_re) {
            let mut chars = string.as_str().chars();
            chars.next();
            match chars.next() {
                Some(ch) => {
                    return Ok(Token {
                        position,
                        info: Constant(Literal::Character(ch)),
                    })
                }
                None => return Err(self.error_at(position, "Invalid character")),
            }
        }
        // Must be before decimal.
        if let Some(string) = self.stream.scan(&self.float_re) {
            return Ok(Token {
                position,
                info: Constant(Literal::Float(string.parse().unwrap())),
            });
        }
        if let Some(string) = self.stream.scan(&self.decimal_re) {
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
        if let Some(string) = self.stream.scan(&self.positional_parameter_re) {
            return Ok(Token {
                position,
                info: PositionalParameter(string[1..].to_string()),
            });
        }
        if let Some(string) = self.stream.scan(&self.operator_re) {
            return Ok(Token {
                position,
                info: Operator(string),
            });
        }
        if let Some(string) = self.stream.scan(&self.selector_re) {
            return Ok(Token {
                position,
                info: Constant(Literal::Selector(string[1..].to_string())),
            });
        }
        if let Some(string) = self.stream.scan(&self.keyword_re) {
            return Ok(Token {
                position,
                info: Keyword(string),
            });
        }
        if let Some(_) = self.stream.scan(&self.return_re) {
            return Ok(Token {
                position,
                info: Return(),
            });
        }
        if let Some(string) = self.stream.scan(&self.identifier_re) {
            return Ok(Token {
                position,
                info: Identifier(string),
            });
        }
        Err(self.error_at(position, "Invalid token"))
    }
    fn error(&self, token: Token, problem: &'static str) -> Result<Expr, ParseError> {
        Err(self.error_at(token.position, problem))
    }
    fn error_context(&self, position: usize, problem: &str) -> String {
        fn append_line(ctx: &mut String, n: usize, line: &str) {
            if n == 0 {
                ctx.push_str(format!("    {}\n", line).as_str());
            } else {
                ctx.push_str(format!("{:03} {}\n", n, line).as_str());
            }
        }
        let mut context = String::new();
        let mut prev = "";
        let mut lineno = 1;
        let mut start = 0;
        for line in self.stream.as_str().lines() {
            if start > position {
                // Line after the problem -- done.
                append_line(&mut context, lineno, line);
                break;
            }
            let end = start + line.len();
            if end > position {
                // Previous line if there is one.
                if lineno > 1 {
                    append_line(&mut context, lineno - 1, prev);
                }
                // Line with the problem.
                append_line(&mut context, lineno, line);
                println!("position: {}, start: {}", position, start);
                let span = position - start;
                let mut mark = String::from_utf8(vec![b' '; span]).unwrap();
                mark.push_str("^-- ");
                mark.push_str(problem);
                append_line(&mut context, 0, mark.as_str());
            }
            prev = line;
            start = end + 1;
            lineno += 1;
        }
        return context;
    }
    fn error_at(&self, position: usize, problem: &'static str) -> ParseError {
        ParseError {
            position,
            problem,
            context: self.error_context(position, problem),
        }
    }
    fn scan_literal_block_string(&mut self, quote: String) -> Result<Token, ParseError> {
        let start0 = self.stream.position() - quote.len();
        let col = self.stream.column();
        let mut start = self.stream.position();
        let mut content = String::new();
        loop {
            let s = self.stream.str();
            if s.len() == 0 {
                return Err(self.error_at(start0, "Unterminated raw block string"));
            }
            if &s[0..1] == "\n" {
                content.push_str(&self.stream.as_str()[start..=self.stream.position()]);
                // + 1 for the newline we just pushed.
                start = self.stream.skip_whitespace(col + 1);
                continue;
            }
            if s.len() >= 4 && &s[..4] == r#""""$"# {
                if s.len() >= 5 && &s[..5] == r#""""$$"# {
                    self.stream.skip(4);
                    content.push_str(&self.stream.as_str()[start..self.stream.position()]);
                    start = self.stream.skip(1);
                    continue;
                } else {
                    break;
                }
            }
            self.stream.skip(1);
        }
        content.push_str(&self.stream.as_str()[start..self.stream.position()]);
        self.stream.skip(4);
        Ok(Token {
            position: start - quote.len(),
            info: TokenInfo::Constant(Literal::String(content)),
        })
    }
    fn scan_literal_string(&mut self, quote: String) -> Result<Token, ParseError> {
        let start0 = self.stream.position();
        let mut start = start0;
        let mut content = String::new();
        loop {
            let s = self.stream.str();
            if s.len() == 0 {
                return Err(self.error_at(start, "Unterminated string"));
            }
            if s.len() >= 2 && &s[..2] == r#""$"# {
                if s.len() >= 3 && &s[..3] == r#""$$"# {
                    self.stream.skip(2);
                    content.push_str(&self.stream.as_str()[start..self.stream.position()]);
                    start = self.stream.skip(1);
                    continue;
                } else {
                    break;
                }
            }
            self.stream.skip(1);
        }
        content.push_str(&self.stream.as_str()[start..self.stream.position()]);
        self.stream.skip(2);
        Ok(Token {
            position: start - quote.len(),
            info: TokenInfo::Constant(Literal::String(content)),
        })
    }
    fn scan_interpolated_string(&mut self, quote: String) -> Result<Token, ParseError> {
        let start0 = self.stream.position() - quote.len();
        let mut start = self.stream.position();
        let mut content = String::new();
        loop {
            if self.stream.at_eof() {
                return Err(self.error_at(start0, "Unterminated string"));
            }
            if self.scan_string_escape_sequence(start0, &mut content)? {
                start = self.stream.position();
                continue;
            }
            if &self.stream.str()[0..1] == r#"""# {
                break;
            }
            self.stream.skip(1);
        }
        content.push_str(&self.stream.as_str()[start..self.stream.position()]);
        self.stream.skip(quote.len());
        Ok(Token {
            position: start0,
            info: TokenInfo::InterpolatedString(content),
        })
    }
    fn scan_interpolated_block_string(&mut self, quote: String) -> Result<Token, ParseError> {
        let start0 = self.stream.position() - quote.len() + 1;
        let mut start = self.stream.position();
        let mut content = String::new();
        let col = self.stream.column();
        loop {
            if self.stream.at_eof() {
                return Err(self.error_at(start0, "Unterminated block string"));
            }
            if &self.stream.str()[0..1] == "\n" {
                content.push_str(&self.stream.as_str()[start..=self.stream.position()]);
                start = self.stream.skip_whitespace(col + 1);
                continue;
            }
            if self.scan_string_escape_sequence(start0, &mut content)? {
                start = self.stream.position();
                continue;
            }
            let s = self.stream.str();
            if s.len() >= 3 && &s[..3] == r#"""""# {
                break;
            }
            self.stream.skip(1);
        }
        content.push_str(&self.stream.as_str()[start..self.stream.position()]);
        self.stream.skip(quote.len());
        Ok(Token {
            position: start,
            info: TokenInfo::InterpolatedString(content),
        })
    }
    fn scan_literal_record(&mut self, open: String) -> Result<Token, ParseError> {
        Ok(Token {
            position: self.stream.position() - open.len(),
            info: TokenInfo::LiteralRecordBegin(),
        })
    }
    fn scan_string_escape_sequence(
        &mut self,
        start: usize,
        content: &mut String,
    ) -> Result<bool, ParseError> {
        let s = self.stream.str();
        if &s[..1] == r#"\"# {
            if s.len() < 2 {
                return Err(self.error_at(start, "Unterminated string"));
            }
            content.push_str(&self.stream.as_str()[start..self.stream.position()]);
            match &s[1..2] {
                r#"""# => {
                    content.push_str("\"");
                }
                r#"n"# => {
                    content.push_str("\n");
                }
                r#"r"# => {
                    content.push_str("\r");
                }
                r#"t"# => {
                    content.push_str("\t");
                }
                r#"\"# => {
                    content.push_str("\\");
                }
                _ => {
                    return Err(self.error_at(
                        self.stream.position(),
                        "Unknown escape sequece in block string",
                    ))
                }
            }
            self.stream.skip(2);
            return Ok(true);
        } else {
            return Ok(false);
        }
    }
}

trait Stream {
    fn column(&self) -> usize;
    fn position(&self) -> usize;
    fn at_eof(&self) -> bool;
    fn at_eol(&self) -> bool;
    fn skip(&mut self, n: usize) -> usize;
    fn skip_whitespace(&mut self, n: usize) -> usize;
    fn at_whitespace(&self) -> bool;
    fn str(&self) -> &str;
    fn charstr(&self) -> &str;
    fn as_str(&self) -> &str;
    fn string_from(&self, start: usize) -> String;
    fn scan(&mut self, re: &Regex) -> Option<String>;
    fn seek(&mut self, position: usize);
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
    fn seek(&mut self, position: usize) {
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
    fn column(&self) -> usize {
        let mut prev = self.position;
        loop {
            if prev == 0 || &self.string[prev..prev + 1] == "\n" {
                return self.position - prev;
            }
            prev -= 1;
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
    fn skip(&mut self, n: usize) -> usize {
        self.position += n;
        self.position
    }
    fn skip_whitespace(&mut self, n: usize) -> usize {
        for _ in 0..n {
            if self.at_whitespace() {
                self.skip(1);
            } else {
                break;
            }
        }
        self.position
    }
    fn charstr(&self) -> &str {
        &self.string[self.position..self.position + 1]
    }
    fn str(&self) -> &str {
        &self.string[self.position..]
    }
    fn as_str(&self) -> &str {
        &self.string[..]
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
