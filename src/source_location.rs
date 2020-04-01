use std::ops::Range;
use std::path::PathBuf;
use std::rc::Rc;

pub type Span = Range<usize>;

pub trait TweakSpan {
    fn tweak(&mut self, shift: usize, extend: isize);
    fn shift(&mut self, shift: usize) {
        self.tweak(shift, 0);
    }
    fn extend(&mut self, extend: isize) {
        self.tweak(0, extend);
    }
}

impl TweakSpan for Span {
    fn tweak(&mut self, shift: usize, extend: isize) {
        self.start += shift;
        self.end += shift;
        if extend < 0 {
            self.start -= (-extend) as usize;
        } else {
            self.end += extend as usize;
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SourceLocation {
    Path(SourcePath),
    Span(Span),
}

impl SourceLocation {
    pub fn span(span: &Span) -> SourceLocation {
        SourceLocation::Span(span.clone())
    }
    pub fn path(path: &Rc<PathBuf>, span: &Span) -> SourceLocation {
        SourceLocation::Path(SourcePath {
            span: span.clone(),
            path: path.clone(),
        })
    }
    pub fn get_span(&self) -> Span {
        match &self {
            SourceLocation::Span(span) => span.clone(),
            SourceLocation::Path(path) => path.span.clone(),
        }
    }
    pub fn set_span(&mut self, span: &Span) {
        *self = match &self {
            SourceLocation::Span(_) => SourceLocation::span(span),
            SourceLocation::Path(path) => SourceLocation::path(&path.path, span),
        }
    }
    pub fn end(&self) -> usize {
        match &self {
            SourceLocation::Span(span) => span.end,
            SourceLocation::Path(path) => path.span.end,
        }
    }
    pub fn code(&self) -> Option<String> {
        match &self {
            SourceLocation::Span(_) => None,
            SourceLocation::Path(path) => {
                Some(std::fs::read_to_string(path.path.as_path()).unwrap())
            }
        }
    }
    pub fn extend_to(&mut self, end: usize) {
        self.extend((end - self.end()) as isize);
    }
}

impl TweakSpan for SourceLocation {
    fn tweak(&mut self, shift: usize, extend: isize) {
        match self {
            SourceLocation::Span(span) => span.tweak(shift, extend),
            SourceLocation::Path(path) => path.tweak(shift, extend),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SourcePath {
    span: Span,
    path: Rc<PathBuf>,
}

impl SourcePath {
    fn tweak(&mut self, shift: usize, extend: isize) {
        self.span.tweak(shift, extend)
    }
}
