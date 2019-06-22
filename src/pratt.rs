use regex::Regex;

use std::borrow::ToOwned;
use std::collections::VecDeque;
#[derive(PartialEq)]
struct ParseError {
    position: usize,
    problem: &'static str,
    context: String,
}

impl std::fmt::Debug for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ParseError at {}:\n{}", self.position, self.context)
    }
}

#[derive(Debug, PartialEq, Clone)]
enum Literal {
    Decimal(i64),
    Float(f64),
    Selector(String),
    Character(char),
    String(String),
}

#[derive(Debug, PartialEq, Clone)]
struct Message {
    selector: String,
    arguments: Vec<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
enum Expr {
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
    Sequence(bool),
    Chain(),
    Cascade(),
    // true = comma, false = newline
    BlockBegin(),
    BlockEnd(),
    ParenBegin(),
    ParenEnd(),
    ArrayBegin(),
    ArrayEnd(),
    Bind(),
    Assign(),
    Return(),
}

impl Token {
    fn name(&self) -> &str {
        match &self.info {
            TokenInfo::Operator(name) => name.as_str(),
            TokenInfo::Keyword(name) => name.as_str(),
            TokenInfo::Type(name) => name.as_str(),
            _ => panic!("Token has no name: {:?}", self),
        }
    }
    fn precedence(&self) -> usize {
        match &self.info {
            TokenInfo::Eof() => 0,
            TokenInfo::ArrayBegin() => 0,
            TokenInfo::ArrayEnd() => 0,
            TokenInfo::BlockBegin() => 0,
            TokenInfo::BlockEnd() => 0,
            TokenInfo::ParenBegin() => 0,
            TokenInfo::ParenEnd() => 0,
            TokenInfo::Bind() => 1,
            TokenInfo::Return() => 2,
            TokenInfo::Assign() => 2,
            TokenInfo::Sequence(_) => 2,
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

fn parse_stream(stream: Box<Stream>) -> Result<Expr, ParseError> {
    Parser::new(stream).parse()
}

fn parse_string(string: String) -> Result<Expr, ParseError> {
    parse_stream(Box::new(StringStream::new(string)))
}

fn parse_str(str: &str) -> Result<Expr, ParseError> {
    parse_string(str.to_string())
}

struct Parser {
    stream: Box<Stream>,
    lookahead: VecDeque<Token>,
    cascade: Option<Expr>,
    interpolated_string_re: Regex,
    literal_block_string_re: Regex,
    literal_string_re: Regex,
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
            literal_block_string_re: Regex::new(r#"^\$""""#).unwrap(),
            literal_string_re: Regex::new(r#"^\$""#).unwrap(),
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
    fn parse(&mut self) -> Result<Expr, ParseError> {
        self.parse_expression(0)
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
        let expr = self.parse_expression(token.precedence())?;
        let next = self.consume_token()?;
        match next.info {
            TokenInfo::ArrayEnd() => match expr {
                Expr::Sequence(exprs) => Ok(Expr::Array(exprs)),
                _ => Ok(Expr::Array(vec![expr])),
            },
            _ => self.error(token, "Unmatched array start"),
        }
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
    fn parse_prefix_interpolated_string(&mut self, token: Token) -> Result<Expr, ParseError> {
        unimplemented!("interpolated strings");
        let mut stream = StringStream::new(token.string());
        let mut expressions = vec![];
        let mut string = String::new();
        loop {
            if stream.at_eof() {
                break;
            }
            if &stream.charstr() == "{" {
                let p = stream.position();
                stream.skip(1);
                expressions.push(stream.parse()?);
                if &stream.charstr() != "}" {
                    return self.error(p, "Unterminated interpolation in string")
                }
                match  {
                    Expr::Constant(Literal::St)
                }
                parse_stream(Box::new(stream.clone()));
            }
        }
    }
    fn parse_prefix_operator(&mut self, token: Token) -> Result<Expr, ParseError> {
        let message = match token.name() {
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
                    return self.error(token, "Invalid cascade");
                }
            }
        }
        Ok(Expr::Cascade(Box::new(left.clone()), chains))
    }
    fn parse_suffix_keyword(&mut self, left: Expr, token: Token) -> Result<Expr, ParseError> {
        let mut args = vec![];
        let mut selector = token.name().to_string();
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
        Ok(left.binary(token.name().to_string(), right))
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
        Ok(Expr::Type(token.name().to_string(), Box::new(left)))
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
            self.stream.rewind(mark);
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
        if let Some(s) = self.stream.scan(&self.interpolated_string_re) {
            return self.scan_interpolated_string(s);
        }
        if let Some(s) = self.stream.scan(&self.literal_block_string_re) {
            return self.scan_literal_block_string(s);
        }
        if let Some(s) = self.stream.scan(&self.literal_string_re) {
            return self.scan_literal_string(s);
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
                // Line with the problem.
                if prev.len() > 0 {
                    append_line(&mut context, lineno - 1, prev);
                }
                append_line(&mut context, lineno, line);
                let mut mark = String::from_utf8(vec![b' '; position - start - 1]).unwrap();
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
                return Err(self.error_at(start0, "Unterminated string"));
            }
            if &s[0..1] == "\n" {
                content.push_str(&self.stream.as_str()[start..=self.stream.position()]);
                self.stream.skip(col + 1);
                for _ in 0..col + 1 {
                    if self.stream.at_whitespace() {
                        self.stream.skip(1);
                    } else {
                        break;
                    }
                }
                start = self.stream.position();
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
            let s = self.stream.str();
            if s.len() == 0 {
                return Err(self.error_at(start0, "Unterminated string"));
            }
            if &s[..1] == r#"\"# {
                if s.len() < 2 {
                    return Err(self.error_at(start0, "Unterminated string"));
                }
                match &s[1..2] {
                    r#"""# => {
                        content.push_str(&self.stream.as_str()[start..self.stream.position()]);
                        content.push_str("\"");
                        start = self.stream.skip(2);
                        continue;
                    }
                    r#"n"# => {
                        content.push_str(&self.stream.as_str()[start..self.stream.position()]);
                        content.push_str("\n");
                        start = self.stream.skip(2);
                        continue;
                    }
                    r#"r"# => {
                        content.push_str(&self.stream.as_str()[start..self.stream.position()]);
                        content.push_str("\r");
                        start = self.stream.skip(2);
                        continue;
                    }
                    r#"t"# => {
                        content.push_str(&self.stream.as_str()[start..self.stream.position()]);
                        content.push_str("\t");
                        start = self.stream.skip(2);
                        continue;
                    }
                    r#"\"# => {
                        content.push_str(&self.stream.as_str()[start..self.stream.position()]);
                        content.push_str("\\");
                        start = self.stream.skip(2);
                        continue;
                    }
                    _ => {
                        return Err(self.error_at(
                            self.stream.position() + 1,
                            "Unknown escape sequece in string",
                        ))
                    }
                }
            }
            if &s[..1] == r#"""# {
                break;
            }
            self.stream.skip(1);
        }
        content.push_str(&self.stream.as_str()[start..self.stream.position()]);
        self.stream.skip(1);
        Ok(Token {
            position: start,
            info: TokenInfo::InterpolatedString(content),
        })
    }
}

trait Stream {
    fn column(&self) -> usize;
    fn position(&self) -> usize;
    fn at_eof(&self) -> bool;
    fn at_eol(&self) -> bool;
    fn skip(&mut self, n: usize) -> usize;
    fn at_whitespace(&self) -> bool;
    fn str(&self) -> &str;
    fn charstr(&self) -> &str;
    fn as_str(&self) -> &str;
    fn string_from(&self, start: usize) -> String;
    fn scan(&mut self, re: &Regex) -> Option<String>;
    fn rewind(&mut self, position: usize);
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
    fn rewind(&mut self, position: usize) {
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

#[cfg(test)]
fn decimal(value: i64) -> Expr {
    Expr::Constant(Literal::Decimal(value))
}

#[cfg(test)]
fn float(value: f64) -> Expr {
    Expr::Constant(Literal::Float(value))
}

#[cfg(test)]
fn var(name: &str) -> Expr {
    Expr::Variable(name.to_string())
}

#[cfg(test)]
fn chain(object: Expr, messages: &[Message]) -> Expr {
    Expr::Chain(Box::new(object), messages.to_vec())
}

#[cfg(test)]
fn unary(name: &str) -> Message {
    Message {
        selector: name.to_string(),
        arguments: vec![],
    }
}

#[cfg(test)]
fn binary(name: &str, expr: Expr) -> Message {
    Message {
        selector: name.to_string(),
        arguments: vec![expr],
    }
}

#[cfg(test)]
fn keyword(name: &str, exprs: &[Expr]) -> Message {
    Message {
        selector: name.to_string(),
        arguments: exprs.to_vec(),
    }
}

#[test]
fn parse_decimal() {
    assert_eq!(parse_str(" 123 "), Ok(decimal(123)));
}

#[test]
fn parse_float() {
    assert_eq!(parse_str(" 123.123 "), Ok(float(123.123)));
}

#[test]
fn parse_variable() {
    assert_eq!(parse_str(" abc "), Ok(var("abc")));
}

#[test]
fn parse_unary_send() {
    assert_eq!(
        parse_str(" abc foo bar "),
        Ok(chain(var("abc"), &[unary("foo"), unary("bar")]))
    );
}

#[test]
fn parse_binary_send() {
    assert_eq!(
        parse_str(" abc + bar "),
        Ok(chain(var("abc"), &[binary("+", var("bar"))]))
    );
}

#[test]
fn parse_unary_prefix() {
    assert_eq!(parse_str(" -a "), Ok(chain(var("a"), &[unary("neg")])));
}

#[test]
fn parse_binary_precedence() {
    assert_eq!(
        parse_str(" abc + bar * quux"),
        Ok(chain(
            var("abc"),
            &[binary("+", chain(var("bar"), &[binary("*", var("quux"))]))]
        ))
    );
}

#[test]
fn parse_parens() {
    assert_eq!(
        parse_str(" abc * (bar + quux)"),
        Ok(chain(
            var("abc"),
            &[binary("*", chain(var("bar"), &[binary("+", var("quux"))]))]
        ))
    );
}

#[test]
fn parse_unary_and_binary_send() {
    assert_eq!(
        parse_str(" abc foo + bar foo2"),
        Ok(chain(
            var("abc"),
            &[
                unary("foo"),
                binary("+", chain(var("bar"), &[unary("foo2")]))
            ]
        ))
    );
}

#[test]
fn parse_keyword_send() {
    assert_eq!(
        parse_str(" obj key1: arg1 key2: arg2"),
        Ok(chain(
            var("obj"),
            &[keyword("key1:key2:", &[var("arg1"), var("arg2")])]
        ))
    );
}

#[test]
fn parse_keyword_chain() {
    assert_eq!(
        parse_str(" obj send1: arg1 -- send2: arg2"),
        Ok(chain(
            var("obj"),
            &[
                keyword("send1:", &[var("arg1")]),
                keyword("send2:", &[var("arg2")])
            ]
        ))
    );
}

#[test]
fn parse_keyword_unary_chain() {
    assert_eq!(
        parse_str(" obj send1: arg1 -- bar"),
        Ok(chain(
            var("obj"),
            &[keyword("send1:", &[var("arg1")]), unary("bar")]
        ))
    );
}

#[test]
fn parse_keyword_and_binary_send() {
    assert_eq!(
        parse_str(" obj key1: arg1 + x key2: arg2 + y"),
        Ok(chain(
            var("obj"),
            &[keyword(
                "key1:key2:",
                &[
                    chain(var("arg1"), &[binary("+", var("x"))]),
                    chain(var("arg2"), &[binary("+", var("y"))]),
                ]
            )]
        ))
    );
}

#[test]
fn parse_keyword_and_unary_send() {
    assert_eq!(
        parse_str(" obj key1: arg1 foo bar key2: arg2 quux zot"),
        Ok(chain(
            var("obj"),
            &[keyword(
                "key1:key2:",
                &[
                    chain(var("arg1"), &[unary("foo"), unary("bar")]),
                    chain(var("arg2"), &[unary("quux"), unary("zot")]),
                ]
            )]
        ))
    );
}

#[test]
fn parse_cascade() {
    assert_eq!(
        // This is not like smalltalk cascade!
        parse_str(
            "obj zoo
                   ; foo: x bar: y -- zot
                   ; do thing
                   "
        ),
        Ok(Expr::Cascade(
            Box::new(chain(var("obj"), &[unary("zoo")])),
            vec![
                vec![keyword("foo:bar:", &[var("x"), var("y")]), unary("zot")],
                vec![unary("do"), unary("thing"),]
            ]
        ))
    );
}

#[test]
fn parse_error_context() {
    assert_eq!(
        // This is not like smalltalk cascade!
        parse_str(
            "obj zoo
                   ; foo: x bar!: y -- zot
                   ; do thing
                   "
        ),
        Err(ParseError {
            position: 40,
            problem: "Invalid token",
            context: "001 obj zoo
002                    ; foo: x bar!: y -- zot
                                   ^-- Invalid token
003                    ; do thing
"
            .to_string()
        })
    );
}

#[test]
fn test_parse_sequence() {
    assert_eq!(
        parse_str("foo bar, quux zot"),
        Ok(Expr::Sequence(vec![
            chain(var("foo"), &[unary("bar")]),
            chain(var("quux"), &[unary("zot")])
        ]))
    );
    assert_eq!(
        parse_str(
            "
            foo bar
            quux zot"
        ),
        Ok(Expr::Sequence(vec![
            chain(var("foo"), &[unary("bar")]),
            chain(var("quux"), &[unary("zot")])
        ]))
    );
    assert_eq!(
        parse_str(
            r"
            zoo foo +
              barz
            quux \
              + zot"
        ),
        Ok(Expr::Sequence(vec![
            chain(var("zoo"), &[unary("foo"), binary("+", var("barz"))]),
            chain(var("quux"), &[binary("+", var("zot"))])
        ]))
    );
}

#[test]
fn parse_block() {
    assert_eq!(
        parse_str("{ a + b }"),
        Ok(Expr::Block(
            vec![],
            Box::new(chain(var("a"), &[binary("+", var("b"))]))
        ))
    );
}

#[test]
fn parse_block_with_args() {
    assert_eq!(
        parse_str("{ :a :b | a + b }"),
        Ok(Expr::Block(
            vec!["a".to_string(), "b".to_string()],
            Box::new(chain(var("a"), &[binary("+", var("b"))]))
        ))
    );
}

#[test]
fn parse_array() {
    assert_eq!(
        parse_str("[1,2,3]"),
        Ok(Expr::Array(vec![decimal(1), decimal(2), decimal(3)]))
    );
}

#[test]
fn parse_bind() {
    assert_eq!(
        parse_str("let x := 42, x foo, x + 1"),
        Ok(Expr::Bind(
            "x".to_string(),
            Box::new(decimal(42)),
            Box::new(Expr::Sequence(vec![
                chain(var("x"), &[unary("foo")]),
                chain(var("x"), &[binary("+", decimal(1))]),
            ]))
        ))
    );
}

#[test]
fn parse_assign() {
    assert_eq!(
        parse_str("x := 42, x"),
        Ok(Expr::Sequence(vec![
            Expr::Assign("x".to_string(), Box::new(decimal(42))),
            var("x")
        ]))
    );
}

#[test]
fn parse_return() {
    assert_eq!(
        parse_str("return 42, 666"),
        Ok(Expr::Sequence(vec![
            Expr::Return(Box::new(decimal(42))),
            decimal(666)
        ]))
    );
}

#[test]
fn parse_type() {
    assert_eq!(
        parse_str("x <Int> + y <Int>"),
        Ok(chain(
            Expr::Type("Int".to_string(), Box::new(var("x"))),
            &[binary(
                "+",
                Expr::Type("Int".to_string(), Box::new(var("y")))
            )]
        ))
    );
}

#[test]
fn parse_selector() {
    assert_eq!(
        parse_str("[$foo, $bar:quux:] "),
        Ok(Expr::Array(vec![
            Expr::Constant(Literal::Selector("foo".to_string())),
            Expr::Constant(Literal::Selector("bar:quux:".to_string()))
        ]))
    );
}

#[test]
fn parse_character() {
    assert_eq!(
        parse_str("'x'"),
        Ok(Expr::Constant(Literal::Character('x')))
    );
}

#[test]
fn parse_literal_string() {
    assert_eq!(
        parse_str(r#" $"foo"$$"$ "#),
        Ok(Expr::Constant(Literal::String(r#"foo"$"#.to_string())))
    );
}

#[test]
fn parse_literal_block_string() {
    assert_eq!(
        parse_str(
            r#"   $"""foo
       bar"""$"#
        ),
        Ok(Expr::Constant(Literal::String("foo\nbar".to_string())))
    );
}

#[test]
fn parse_interpolated_string_no_interpolation() {
    assert_eq!(
        parse_str(r#" "foo bar" "#),
        Ok(Expr::Constant(Literal::String("foo bar".to_string())))
    );
}
