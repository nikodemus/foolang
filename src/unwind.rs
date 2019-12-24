use std::fmt;

use crate::eval::Env;
use crate::objects::Object;
use crate::tokenstream::Span;

trait LineIndices {
    // FIXME: learn to implement iterators
    fn line_indices(&self) -> Vec<(usize, &str)>;
}

impl LineIndices for str {
    fn line_indices(&self) -> Vec<(usize, &str)> {
        let mut all = Vec::new();
        let mut start: usize = 0;
        while start < self.len() {
            if let Some(newline0) = self[start..].find("\n") {
                let newline = newline0 + start;
                // Check for preceding carriage return.
                let end = if newline > 0 && &self[newline - 1..newline] == "\r" {
                    newline - 1
                } else {
                    newline
                };
                all.push((start, &self[start..end]));
                start = newline + 1;
            } else {
                all.push((start, &self[start..]));
                start = self.len();
            }
        }
        all
    }
}

#[derive(PartialEq, Debug)]
pub enum Unwind {
    Exception(Error, Location),
    ReturnFrom(Env, Object),
}

#[derive(PartialEq, Debug)]
pub enum Error {
    MessageError(MessageError),
    SimpleError(SimpleError),
    TypeError(TypeError),
    EofError(SimpleError),
}

// FIXME: This might break encapsulation too badly?
#[derive(PartialEq, Debug)]
pub struct MessageError {
    pub message: String,
    pub receiver: Object,
    pub arguments: Vec<Object>,
}

#[derive(PartialEq, Debug)]
pub struct SimpleError {
    pub what: String,
}

#[derive(PartialEq, Debug)]
pub struct TypeError {
    pub value: Object,
    pub expected: String,
}

#[derive(PartialEq, Debug)]
pub struct Location {
    pub span: Option<Span>,
    pub context: Option<String>,
}

impl fmt::Display for Unwind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Unwind::Exception(error, location) => {
                write!(f, "ERROR: {}\n{}", error.what(), location.context())
            }
            Unwind::ReturnFrom(_, object) => write!(f, "#<Return {}>", object),
        }
    }
}

impl Unwind {
    // FIXME: The vtable as expected, extract name here.
    pub fn type_error<T>(value: Object, expected: String) -> Result<T, Unwind> {
        Err(Unwind::Exception(
            Error::TypeError(TypeError {
                value,
                expected,
            }),
            Location::none(),
        ))
    }

    pub fn type_error_at<T>(span: Span, value: Object, expected: String) -> Result<T, Unwind> {
        Err(Unwind::Exception(
            Error::TypeError(TypeError {
                value,
                expected,
            }),
            Location::new(span),
        ))
    }

    pub fn message_error<T>(
        receiver: &Object,
        message: &str,
        args: &[Object],
    ) -> Result<T, Unwind> {
        Err(Unwind::Exception(
            Error::MessageError(MessageError {
                receiver: receiver.clone(),
                message: message.to_string(),
                arguments: args.to_vec(),
            }),
            Location::none(),
        ))
    }

    pub fn eof_error_at<T>(span: Span, what: &str) -> Result<T, Unwind> {
        Err(Unwind::Exception(
            Error::EofError(SimpleError {
                what: what.to_string(),
            }),
            Location::new(span),
        ))
    }

    pub fn error<T>(what: &str) -> Result<T, Unwind> {
        Err(Unwind::Exception(
            Error::SimpleError(SimpleError {
                what: what.to_string(),
            }),
            Location::none(),
        ))
    }

    pub fn error_at<T>(span: Span, what: &str) -> Result<T, Unwind> {
        Err(Unwind::Exception(
            Error::SimpleError(SimpleError {
                what: what.to_string(),
            }),
            Location::new(span),
        ))
    }

    pub fn return_from<T>(env: Env, value: Object) -> Result<T, Unwind> {
        Err(Unwind::ReturnFrom(env, value))
    }

    pub fn add_span(&mut self, span: &Span) {
        if let Unwind::Exception(_, location) = self {
            location.add_span(span)
        }
    }

    pub fn with_context(mut self, source: &str) -> Unwind {
        if let Unwind::Exception(error, location) = &mut self {
            location.add_context(source, error.what());
        }
        self
    }
}

impl Error {
    pub fn what(&self) -> String {
        match self {
            Error::MessageError(e) => e.what(),
            Error::SimpleError(e) => e.what(),
            Error::TypeError(e) => e.what(),
            Error::EofError(e) => e.what(),
        }
    }
}

impl MessageError {
    pub fn what(&self) -> String {
        format!("{:?} does not understand: {} {:?}", self.receiver, self.message, self.arguments)
    }
}

impl SimpleError {
    pub fn what(&self) -> String {
        self.what.clone()
    }
}

impl TypeError {
    pub fn what(&self) -> String {
        format!(
            "{} expected, got: {} {}",
            self.expected,
            self.value.vtable.name.clone(),
            self.value
        )
    }
}

impl Location {
    fn new(span: Span) -> Location {
        Location {
            span: Some(span),
            context: None,
        }
    }

    fn none() -> Location {
        Location {
            span: None,
            context: None,
        }
    }

    pub fn context(&self) -> String {
        match &self.context {
            None => "".to_string(),
            Some(ctx) => ctx.clone(),
        }
    }

    fn start(&self) -> usize {
        if let Some(span) = &self.span {
            span.start
        } else {
            panic!("Expected Location with span")
        }
    }

    fn end(&self) -> usize {
        if let Some(span) = &self.span {
            span.end
        } else {
            panic!("Expected Location with span")
        }
    }

    fn add_span(&mut self, span: &Span) {
        assert!(self.span.is_none());
        self.span = Some(span.clone())
    }

    fn add_context(&mut self, source: &str, what: String) {
        if self.context.is_some() {
            return;
        }
        if self.span.is_none() {
            return;
        }
        assert!(self.context.is_none());
        let mut context = String::new();
        let mut prev = "";
        let mut lineno = 1;
        for (start, line) in source.line_indices() {
            if start >= self.end() {
                // Line after the problem -- done.
                _append_context_line(&mut context, lineno, line);
                break;
            }
            let end = start + line.len();
            if end > self.start() {
                // Previous line if there is one.
                if lineno > 1 {
                    _append_context_line(&mut context, lineno - 1, prev);
                }
                // Line with the problem.
                _append_context_line(&mut context, lineno, line);
                let mut mark = if self.start() > start {
                    String::from_utf8(vec![b' '; self.start() - start]).unwrap()
                } else {
                    "".to_string()
                };
                mark.push_str(
                    String::from_utf8(vec![b'^'; self.end() - self.start()]).unwrap().as_str(),
                );
                mark.push_str(" ");
                mark.push_str(what.as_str());
                _append_context_line(&mut context, 0, mark.as_str());
            }
            prev = line;
            lineno += 1;
        }
        self.context = Some(context);
    }
}

fn _append_context_line(context: &mut String, lineno: usize, line: &str) {
    if lineno == 0 {
        context.push_str(format!("    {}\n", line).as_str());
    } else {
        context.push_str(format!("{:03} {}\n", lineno, line).as_str());
    }
}
