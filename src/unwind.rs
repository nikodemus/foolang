use std::fmt;

use crate::eval::EnvRef;
use crate::objects::Object;
use crate::source_location::SourceLocation;
#[cfg(test)]
use crate::source_location::Span;

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
    Panic(Error, Location),
    ReturnFrom(EnvRef, Object),
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
    pub source_location: Option<SourceLocation>,
    pub context: Option<String>,
}

impl fmt::Display for Unwind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Unwind::Panic(error, location) => match &location.context {
                Some(c) => write!(f, "ERROR: {}\n{}", error.what(), c),
                None => write!(f, "ERROR: {} (no context)", error.what()),
            },
            Unwind::ReturnFrom(_, object) => write!(f, "#<Return {}>", object),
        }
    }
}

impl Unwind {
    // FIXME: The vtable as expected, extract name here.
    pub fn type_error<T>(value: Object, expected: String) -> Result<T, Unwind> {
        // panic!("boom");
        Err(Unwind::Panic(
            Error::TypeError(TypeError {
                value,
                expected,
            }),
            Location::none(),
        ))
    }

    pub fn type_error_at<T>(
        source_location: SourceLocation,
        value: Object,
        expected: String,
    ) -> Result<T, Unwind> {
        // panic!("boom");
        let code = source_location.code();
        let unwind = Unwind::Panic(
            Error::TypeError(TypeError {
                value,
                expected,
            }),
            Location::new(source_location),
        );
        match code {
            Some(code) => Err(unwind.with_context(&code)),
            None => Err(unwind),
        }
    }

    pub fn message_error<T>(
        receiver: &Object,
        message: &str,
        args: &[Object],
    ) -> Result<T, Unwind> {
        Err(Unwind::Panic(
            Error::MessageError(MessageError {
                receiver: receiver.clone(),
                message: message.to_string(),
                arguments: args.to_vec(),
            }),
            Location::none(),
        ))
    }

    pub fn eof_error_at<T>(source_location: SourceLocation, what: &str) -> Result<T, Unwind> {
        Err(Unwind::Panic(
            Error::EofError(SimpleError {
                what: what.to_string(),
            }),
            Location::new(source_location),
        ))
    }

    pub fn error<T>(what: &str) -> Result<T, Unwind> {
        // panic!("BOOM: {}", what);
        Err(Unwind::Panic(
            Error::SimpleError(SimpleError {
                what: what.to_string(),
            }),
            Location::none(),
        ))
    }

    pub fn error_at<T>(source_location: SourceLocation, what: &str) -> Result<T, Unwind> {
        // println!("{} AT: {:?}", what, &source_location);
        let code = source_location.code();
        let unwind = Unwind::Panic(
            Error::SimpleError(SimpleError {
                what: what.to_string(),
            }),
            Location::new(source_location),
        );
        match code {
            Some(code) => Err(unwind.with_context(&code)),
            None => Err(unwind),
        }
    }

    pub fn return_from<T>(env: EnvRef, value: Object) -> Result<T, Unwind> {
        Err(Unwind::ReturnFrom(env, value))
    }

    pub fn add_source_location(&mut self, source_location: &SourceLocation) {
        if let Unwind::Panic(error, location) = self {
            if location.source_location.is_none() {
                let code = source_location.code();
                location.add_source_location(source_location);
                if let Some(code) = code {
                    location.add_context(&code, error.what())
                }
            }
        }
    }

    pub fn shift_span(self, offset: usize) -> Self {
        match self {
            Unwind::Panic(
                err,
                Location {
                    source_location: Some(mut loc),
                    context,
                },
            ) => {
                loc.shift_span(offset);
                Unwind::Panic(
                    err,
                    Location {
                        source_location: Some(loc),
                        context,
                    },
                )
            }
            _ => self,
        }
    }

    pub fn with_context(mut self, source: &str) -> Unwind {
        if let Unwind::Panic(error, location) = &mut self {
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
            "{} expected, got {}: {:?} (bootstrap evaluator)",
            self.expected,
            self.value.vtable.name.clone(),
            self.value,
        )
    }
}

impl Location {
    fn new(source_location: SourceLocation) -> Location {
        Location {
            source_location: Some(source_location),
            context: None,
        }
    }

    #[cfg(test)]
    pub fn from(span: Span, context: &str) -> Location {
        Location {
            source_location: Some(SourceLocation::span(&span)),
            context: Some(context.to_string()),
        }
    }

    pub fn none() -> Location {
        Location {
            source_location: None,
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
        if let Some(source_location) = &self.source_location {
            source_location.get_span().start
        } else {
            panic!("Expected Location with source location")
        }
    }

    fn end(&self) -> usize {
        if let Some(source_location) = &self.source_location {
            source_location.get_span().end
        } else {
            panic!("Expected Location with source location")
        }
    }

    fn add_source_location(&mut self, source_location: &SourceLocation) {
        assert!(self.source_location.is_none());
        self.source_location = Some(source_location.clone())
    }

    fn add_context(&mut self, source: &str, what: String) {
        if self.context.is_some() {
            return;
        }
        if self.source_location.is_none() {
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
