use crate::source_location::{SourceLocation, Span, TweakSpan};
use crate::syntax::Syntax;

#[derive(Debug, PartialEq, Clone)]
pub struct Message {
    pub span: Span,
    pub selector: String,
    pub args: Vec<Expr>,
}

impl Message {
    fn tweak_span(&mut self, shift: usize, ext: isize) {
        self.span.tweak(shift, ext);
        for arg in &mut self.args {
            arg.tweak_span(shift, ext);
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    // FIXME: Add distinct Record type.
    Array(Array),
    Assign(Assign),
    Bind(Bind),
    Block(Block),
    Cascade(Cascade),
    Chain(Chain),
    Const(Const),
    Dictionary(Dictionary),
    Eq(Eq),
    Raise(Raise),
    Return(Return),
    Seq(Seq),
    Typecheck(Typecheck),
    Var(Var),
}

impl Expr {
    pub fn syntax(self) -> Syntax {
        Syntax::Expr(self)
    }

    pub fn is_var(&self) -> bool {
        match self {
            Expr::Var(..) => true,
            _ => false,
        }
    }

    pub fn name(&self) -> String {
        match self {
            Expr::Var(var) => var.name.to_owned(),
            _ => panic!("BUG: cannot extract name from {:?}", self),
        }
    }

    pub fn to_cascade(self, in_cascade: bool) -> Expr {
        // If we're already in cascade then self is a Chain whose receiver is a
        // cascade and we splice the messages into the cascade, which becomes
        // our receiver.
        //
        // Otherwise left becomes the initial receiver of an initially empty
        // cascade.
        match self {
            Expr::Cascade(..) => self,
            Expr::Chain(chain) => {
                if let Expr::Cascade(mut cascade) = *chain.receiver {
                    cascade.chains.push(chain.messages);
                    Expr::Cascade(cascade)
                } else {
                    assert!(in_cascade);
                    Cascade::expr(Box::new(Expr::Chain(chain)), vec![])
                }
            }
            _ => {
                assert!(in_cascade);
                Cascade::expr(Box::new(self), vec![])
            }
        }
    }

    pub fn send(mut self, message: Message) -> Expr {
        match self {
            Expr::Chain(ref mut chain) => {
                chain.messages.push(message);
                self
            }
            _ => Chain::expr(Box::new(self), vec![message]),
        }
    }

    pub fn span(&self) -> Span {
        use Expr::*;
        let span = match self {
            Array(array) => &array.span,
            Assign(assign) => &assign.span,
            Bind(bind) => return bind.value.span(),
            Block(block) => &block.span,
            Cascade(cascade) => return cascade.receiver.span(),
            Dictionary(dictionary) => &dictionary.span,
            Const(constant) => &constant.span,
            Eq(eq) => &eq.span,
            Chain(chain) => return chain.receiver.span(),
            Raise(raise) => &raise.span,
            Return(ret) => &ret.span,
            // FIXME: Wrong span
            Seq(seq) => return seq.exprs[seq.exprs.len() - 1].span(),
            Typecheck(typecheck) => &typecheck.span,
            Var(var) => &var.source_location.span,
        };
        span.to_owned()
    }

    pub fn shift_span(&mut self, n: usize) {
        self.tweak_span(n, 0);
    }

    pub fn extend_span(&mut self, n: isize) {
        self.tweak_span(0, n);
    }

    pub fn tweak_span(&mut self, shift: usize, extend: isize) {
        use Expr::*;
        match self {
            Array(array) => array.tweak_span(shift, extend),
            Assign(assign) => assign.tweak_span(shift, extend),
            Bind(bind) => bind.tweak_span(shift, extend),
            Block(block) => block.tweak_span(shift, extend),
            Cascade(cascade) => cascade.tweak_span(shift, extend),
            Chain(chain) => chain.tweak_span(shift, extend),
            Const(constant) => constant.tweak_span(shift, extend),
            Dictionary(dictionary) => dictionary.tweak_span(shift, extend),
            Eq(eq) => eq.tweak_span(shift, extend),
            Seq(seq) => seq.tweak_span(shift, extend),
            Raise(raise) => raise.tweak_span(shift, extend),
            Return(ret) => ret.tweak_span(shift, extend),
            Typecheck(typecheck) => typecheck.tweak_span(shift, extend),
            Var(var) => {
                var.source_location.tweak(shift, extend);
            }
        };
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Array {
    pub span: Span,
    pub data: Vec<Expr>,
}

impl Array {
    pub fn expr(span: Span, data: Vec<Expr>) -> Expr {
        Expr::Array(Array {
            span,
            data,
        })
    }
    fn tweak_span(&mut self, shift: usize, extend: isize) {
        self.span.tweak(shift, extend);
        for elt in &mut self.data {
            elt.tweak_span(shift, extend);
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Assign {
    pub span: Span,
    pub name: String,
    pub value: Box<Expr>,
}

impl Assign {
    pub fn expr(span: Span, name: String, value: Expr) -> Expr {
        Expr::Assign(Assign {
            span,
            name,
            value: Box::new(value),
        })
    }
    fn tweak_span(&mut self, shift: usize, extend: isize) {
        self.span.tweak(shift, extend);
        self.value.tweak_span(shift, extend);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Bind {
    pub name: String,
    pub typename: Option<String>,
    pub value: Box<Expr>,
    pub body: Option<Box<Expr>>,
}

impl Bind {
    pub fn expr(
        name: String,
        typename: Option<String>,
        value: Box<Expr>,
        body: Option<Box<Expr>>,
    ) -> Expr {
        Expr::Bind(Bind {
            name,
            typename,
            value,
            body,
        })
    }
    fn tweak_span(&mut self, shift: usize, extend: isize) {
        self.value.tweak_span(shift, extend);
        if let Some(ref mut expr) = self.body {
            expr.tweak_span(shift, extend);
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub span: Span,
    pub params: Vec<Var>,
    pub body: Box<Expr>,
    pub rtype: Option<String>,
}

impl Block {
    pub fn expr(span: Span, params: Vec<Var>, body: Box<Expr>, rtype: Option<String>) -> Expr {
        Expr::Block(Block {
            span,
            params,
            body,
            rtype,
        })
    }
    fn tweak_span(&mut self, shift: usize, extend: isize) {
        self.span.tweak(shift, extend);
        for p in &mut self.params {
            p.source_location.tweak(shift, extend);
        }
        self.body.tweak_span(shift, extend);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Cascade {
    pub receiver: Box<Expr>,
    pub chains: Vec<Vec<Message>>,
}

impl Cascade {
    pub fn expr(receiver: Box<Expr>, chains: Vec<Vec<Message>>) -> Expr {
        Expr::Cascade(Cascade {
            receiver,
            chains,
        })
    }
    fn tweak_span(&mut self, shift: usize, extend: isize) {
        self.receiver.tweak_span(shift, extend);
        for chain in &mut self.chains {
            for message in chain {
                message.tweak_span(shift, extend);
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Chain {
    pub receiver: Box<Expr>,
    pub messages: Vec<Message>,
}

impl Chain {
    pub fn expr(receiver: Box<Expr>, messages: Vec<Message>) -> Expr {
        Expr::Chain(Chain {
            receiver,
            messages,
        })
    }
    fn tweak_span(&mut self, shift: usize, extend: isize) {
        self.receiver.tweak_span(shift, extend);
        for message in &mut self.messages {
            message.tweak_span(shift, extend);
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Const {
    pub span: Span,
    pub literal: Literal,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
}

impl Const {
    pub fn expr(span: Span, literal: Literal) -> Expr {
        Expr::Const(Const {
            span,
            literal,
        })
    }
    fn tweak_span(&mut self, shift: usize, extend: isize) {
        self.span.tweak(shift, extend);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Dictionary {
    pub span: Span,
    pub assoc: Vec<(Expr, Expr)>,
}

impl Dictionary {
    pub fn expr(span: Span, assoc: Vec<(Expr, Expr)>) -> Expr {
        Expr::Dictionary(Dictionary {
            span,
            assoc,
        })
    }
    fn tweak_span(&mut self, shift: usize, extend: isize) {
        self.span.tweak(shift, extend);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Eq {
    pub span: Span,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

impl Eq {
    pub fn expr(span: Span, left: Box<Expr>, right: Box<Expr>) -> Expr {
        Expr::Eq(Eq {
            span,
            left,
            right,
        })
    }
    fn tweak_span(&mut self, shift: usize, extend: isize) {
        self.span.tweak(shift, extend);
        self.left.tweak_span(shift, extend);
        self.right.tweak_span(shift, extend);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Raise {
    pub span: Span,
    pub value: Box<Expr>,
}

impl Raise {
    pub fn expr(span: Span, value: Expr) -> Expr {
        Expr::Raise(Raise {
            span,
            value: Box::new(value),
        })
    }
    fn tweak_span(&mut self, shift: usize, extend: isize) {
        self.span.tweak(shift, extend);
        self.value.tweak_span(shift, extend);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Return {
    pub span: Span,
    pub value: Box<Expr>,
}

impl Return {
    pub fn expr(span: Span, value: Expr) -> Expr {
        Expr::Return(Return {
            span,
            value: Box::new(value),
        })
    }
    fn tweak_span(&mut self, shift: usize, extend: isize) {
        self.span.tweak(shift, extend);
        self.value.tweak_span(shift, extend);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Seq {
    pub exprs: Vec<Expr>,
}

impl Seq {
    pub fn expr(exprs: Vec<Expr>) -> Expr {
        Expr::Seq(Seq {
            exprs,
        })
    }
    fn tweak_span(&mut self, shift: usize, extend: isize) {
        for expr in &mut self.exprs {
            expr.tweak_span(shift, extend);
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Typecheck {
    pub span: Span,
    pub expr: Box<Expr>,
    pub typename: String,
}

impl Typecheck {
    pub fn expr(span: Span, expr: Box<Expr>, typename: String) -> Expr {
        Expr::Typecheck(Typecheck {
            span,
            expr,
            typename,
        })
    }
    fn tweak_span(&mut self, shift: usize, extend: isize) {
        self.span.tweak(shift, extend);
        self.expr.tweak_span(shift, extend);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Var {
    pub source_location: SourceLocation,
    pub name: String,
    pub typename: Option<String>,
}

impl Var {
    pub fn untyped(span: Span, name: String) -> Var {
        Var {
            source_location: SourceLocation {
                span,
            },
            name,
            typename: None,
        }
    }
    pub fn typed(span: Span, name: String, typename: String) -> Var {
        Var {
            source_location: SourceLocation {
                span,
            },
            name,
            typename: Some(typename),
        }
    }
}
