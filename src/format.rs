use crate::ast::Expr;

const INDENT: usize = 4;

trait FoolangFormat {}

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

impl Pattern {
    fn _format(&self, indent: usize, pos: usize, buf: &mut String) {
        match self {
            Pattern::Unary(Identifier(m)) => buf.push_str(m.as_str()),
            Pattern::Binary(Identifier(m), Identifier(a)) => {
                buf.push_str(m.as_str());
                buf.push_str(" ");
                buf.push_str(a.as_str());
            }
            Pattern::Keyword(Identifier(m), Identifier(a), x) => {
                buf.push_str(m.as_str());
                buf.push_str(": ");
                buf.push_str(a.as_str());
                if let Some(p) = x {
                    buf.push_str(" ");
                    p._format(indent, pos, buf);
                }
            }
        }
    }
}

impl Literal {
    fn _format(&self, mut indent: usize, mut pos: usize, buf: &mut String) -> usize {
        match self {
            Literal::Array(elts) => {
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
            Literal::Symbol(s) => {
                pos = write(s, pos, buf);
            }
            Literal::Character(s) => {
                pos = write(format!("${}", s).as_str(), pos, buf);
            }
            Literal::Integer(i) => {
                let s = i.to_string();
                pos = write(s.as_str(), pos, buf);
            }
            Literal::Float(f) => {
                let s = f.to_string();
                pos = write(s.as_str(), pos, buf);
            }
            Literal::String(s) => {
                pos = write(s.as_str(), pos, buf);
            }
        }
        pos
    }
}

impl Expr {
    fn _format(&self, mut indent: usize, mut pos: usize, buf: &mut String) -> usize {
        match self {
            Expr::Constant(c) => {
                match c {
                    Literal::Array(_) => {
                        pos = write("#", pos, buf);
                    }
                    Literal::Symbol(_) => {
                        pos = write("#", pos, buf);
                    }
                    _ => {}
                }
                pos = c._format(indent, pos, buf);
            }
            Expr::Assign(Identifier(s), val) => {
                pos = write(format!("{} = ", s).as_str(), pos, buf);
                pos = val._format(indent, pos, buf);
            }
            Expr::Variable(Identifier(s)) => {
                pos = write(s.as_str(), pos, buf);
            }
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
            Expr::Unary(obj, Identifier(s)) => {
                pos = obj._format(indent, pos, buf);
                pos = write(format!(" {}", s).as_str(), pos, buf);
            }
            Expr::Keyword(obj, keys, args) => {
                pos = obj._format(indent, pos, buf);
                for (k, a) in keys.into_iter().zip(args.into_iter()) {
                    pos = write(format!(" {} ", k.0).as_str(), pos, buf);
                    pos = a._format(indent, pos, buf);
                }
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
            }
        }
        pos
    }
}

impl Cascade {
    fn _format(&self, mut indent: usize, mut pos: usize, buf: &mut String) -> usize {
        match self {
            Cascade::Unary(Identifier(s)) => {
                pos = write(s, pos, buf);
            }
            Cascade::Binary(Identifier(s), e) => {
                write("XXX binary", pos, buf);
            }
            Cascade::Keyword(Identifier(s), e, _) => {
                write("XXX bkey", pos, buf);
            }
        }
        pos
    }
}
