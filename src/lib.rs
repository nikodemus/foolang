//use std::fmt::Debug;

#[derive(Debug)]
pub struct Ident(pub String);

impl Ident {
    pub fn append(mut self, s: &str) -> Ident {
        self.0.push_str(s);
        self
    }
}

#[derive(Debug)]
pub struct Method {
    pub pattern: Pattern,
    pub temporaries: Vec<Ident>,
    pub statements: Vec<Expr>,
}

const INDENT: usize = 4;

impl Method {
    pub fn format(&self) -> String {
        let mut buf = String::new();
        self._format(0, 0, &mut buf);
        buf
    }
    fn _format(&self, mut indent: usize, mut pos: usize, buf: &mut String) {
        self.pattern._format(indent, pos, buf);
        indent = newline(indent + INDENT, buf);
        pos = indent;
        if self.temporaries.len() > 0 {
            pos = write("|", pos, buf);
            pos = write(self.temporaries[0].0.as_str(), pos, buf);
            if self.temporaries.len() > 1 {
                for tmp in self.temporaries[1..].iter() {
                    pos = write(" ", pos, buf);
                    pos = write(tmp.0.as_str(), pos, buf);
                }
            }
            pos = write("|", pos, buf);
            indent = newline(indent, buf);
            pos = indent;
        }
        if self.statements.len() > 0 {
            self.statements[0]._format(indent, pos, buf);
        }
        for stm in self.statements[1..].iter() {
            write(".", pos, buf);
            newline(indent, buf);
            pos = indent;
            stm._format(indent, pos, buf);
        }
    }
}

fn newline(indent: usize, buf: &mut String) -> usize {
    buf.push_str("\n");
    for _ in 0..(indent) {
        buf.push_str(" ");
    }
    indent
}

fn write(s: &str, pos: usize, buf: &mut String) -> usize {
    buf.push_str(s);
    pos + s.len()
}

#[derive(Debug)]
pub enum Pattern {
    Unary(Ident),
    Binary(Ident, Ident),
    Keyword(Ident, Ident, Option<Box<Pattern>>),
}

impl Pattern {
    fn _format(&self, indent: usize, pos: usize, buf: &mut String) {
        match self {
            Pattern::Unary(Ident(m)) => buf.push_str(m.as_str()),
            Pattern::Binary(Ident(m), Ident(a)) => {
                buf.push_str(m.as_str());
                buf.push_str(" ");
                buf.push_str(a.as_str());
            },
            Pattern::Keyword(Ident(m), Ident(a), x) => {
                buf.push_str(m.as_str());
                buf.push_str(": ");
                buf.push_str(a.as_str());
                if let Some(p) = x {
                    buf.push_str(" ");
                    p._format(indent, pos, buf);
                }
            },
        }
    }
}


#[derive(Debug)]
pub enum Expr {
    Int(i64),
    Float(f64),
    Symbol(Ident),
    Character(String),
    String(String),
    ArrayConstant(Box<Expr>),
    Array(Vec<Expr>),
    Variable(Ident),
    Return(Box<Expr>),
    Block(Vec<Ident>, Vec<Expr>),
    Assign(Ident, Box<Expr>),
    Unary(Box<Expr>, Ident),
    Binary(Box<Expr>, Ident, Box<Expr>),
    Keyword(Box<Expr>, Ident, Box<Expr>),
    Cascade(Box<Expr>, Vec<Cascade>),
}

impl Expr {
    fn _format(&self, mut indent: usize, mut pos: usize, buf: &mut String) -> usize {
        match self {
            Expr::Int(i) => {
                let s = i.to_string();
                pos = write(s.as_str(), pos, buf);
            },
            Expr::Float(f) => {
                let s = f.to_string();
                pos = write(s.as_str(), pos, buf);
            },
            Expr::Character(s) => {
                pos = write(format!("${}", s).as_str(), pos, buf);
            }
            Expr::Symbol(Ident(s)) => {
                pos = write(format!("#{}", s).as_str(), pos, buf);
            }
            Expr::String(s) => {
                pos = write(s.as_str(), pos, buf);
            }
            Expr::ArrayConstant(data) => {
                pos = write("#", pos, buf);
                data._format(indent, pos, buf);
            }
            Expr::Array(elts) => {
                pos = write("(", pos, buf);
                if elts.len() > 0 {
                    pos = elts[0]._format(indent, pos, buf);
                }
                if elts.len() > 1 {
                    for elt in elts[1..].iter() {
                        pos = write(" ", pos, buf);
                        elt._format(indent, pos, buf);
                    }
                }
                pos = write(")", pos, buf);
            }
            Expr::Assign(Ident(s), val) => {
                pos = write(format!("{} = ", s).as_str(), pos, buf);
                pos = val._format(indent, pos, buf);
            }
            Expr::Variable(Ident(s)) => {
                pos = write(s.as_str(), pos, buf);
            },
            Expr::Block(args, stms) => {
                pos = write("{", pos, buf);
                let blk_indent = indent + INDENT;
                for stm in stms.iter() {
                    pos = newline(blk_indent, buf);
                    pos = stm._format(blk_indent, pos, buf);
                    pos = write(".", pos, buf)
                }
                pos = newline(indent, buf);
                pos = write("}", pos, buf);
            }
            Expr::Return(obj) => {
                pos = write("^", pos, buf);
                pos = obj._format(indent, pos, buf);
            }
            Expr::Unary(obj, Ident(s)) => {
                pos = obj._format(indent, pos, buf);
                pos = write(format!(" {}", s).as_str(), pos, buf);
            }
            Expr::Keyword(obj, Ident(s), cont) => {
                pos = obj._format(indent, pos, buf);
                pos = write(format!(" {} ", s).as_str(), pos, buf);
                pos = cont._format(indent, pos, buf);
            }
            Expr::Cascade(obj, cascade) => {
                pos = obj._format(indent, pos, buf);
                let blk_indent = indent + INDENT;
                for c in cascade.iter() {
                    write(";", pos, buf);
                    pos = newline(blk_indent, buf);
                    pos = c._format(blk_indent, pos, buf);
                }
            }
            _ => {
                buf.push_str(format!("XXX: {:?}", self).as_str());
            },
        }
        pos
    }
}


#[derive(Debug)]
pub enum Cascade {
    Unary(Ident),
    Binary(Ident, Expr),
    Keyword(Ident, Expr, Option<Box<Cascade>>),
}

impl Cascade {
    fn _format(&self, mut indent: usize, mut pos: usize, buf: &mut String) -> usize {
        match self {
            Cascade::Unary(Ident(s)) => {
                pos = write(s, pos, buf);
            },
            Cascade::Binary(Ident(s), e) => {
                write("XXX binary", pos, buf);
            },
            Cascade::Keyword(Ident(s), e, _) => {
                write("XXX bkey", pos, buf);
            },
        }
        pos
    }
}

pub fn prepend<T>(e: T, mut es: Vec<T>) -> Vec<T> {
    es.insert(0, e);
    es
}

pub fn chop(mut s: String) -> String {
    s.remove(0);
    s
}
