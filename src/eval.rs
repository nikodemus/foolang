use std::borrow::ToOwned;
use std::collections::HashMap;
use std::rc::Rc;

use crate::objects2::{Builtins, Object};
use crate::parse::{parse_str, Expr, Literal, SyntaxError};

struct Env<'a> {
    builtins: &'a Builtins,
    frame: Rc<Frame>,
}

struct Frame {
    local: HashMap<String, Object>,
    parent: Option<Rc<Frame>>,
}

impl<'a> Env<'a> {
    pub fn new(builtins: &Builtins) -> Env {
        Env::from_parts(builtins, HashMap::new(), None)
    }

    pub fn eval(&self, expr: &Expr) -> Result<Object, SyntaxError> {
        match expr {
            Expr::Bind(name, value, body) => self.eval_bind(name, value, body),
            Expr::Constant(_, literal) => self.eval_literal(literal),
            Expr::Send(_, selector, receiver, args) => self.eval_send(selector, receiver, args),
            Expr::Seq(..) => unimplemented!("TODO: eval Seq"),
            Expr::Variable(_, name) => self.eval_variable(name),
        }
    }

    fn from_parts(
        builtins: &'a Builtins,
        local: HashMap<String, Object>,
        parent: Option<Rc<Frame>>,
    ) -> Env<'a> {
        Env {
            builtins,
            frame: Rc::new(Frame {
                local,
                parent,
            }),
        }
    }

    fn bind(&self, name: &String, value: Object) -> Env {
        let mut local = HashMap::new();
        local.insert(name.to_owned(), value);
        Env::from_parts(self.builtins, local, Some(Rc::clone(&self.frame)))
    }

    fn eval_bind(&self, name: &String, value: &Expr, body: &Expr) -> Result<Object, SyntaxError> {
        self.bind(name, self.eval(value)?).eval(body)
    }

    fn eval_literal(&self, literal: &Literal) -> Result<Object, SyntaxError> {
        match literal {
            Literal::Integer(value) => Ok(self.builtins.make_integer(*value)),
            Literal::Float(value) => Ok(self.builtins.make_float(*value)),
        }
    }

    fn eval_send(
        &self,
        selector: &String,
        receiver: &Box<Expr>,
        args: &Vec<Expr>,
    ) -> Result<Object, SyntaxError> {
        let receiver = self.eval(receiver)?;
        let mut values = Vec::new();
        for arg in args {
            values.push(self.eval(arg)?);
        }
        let args: Vec<&Object> = values.iter().collect();
        receiver.send(selector.as_str(), &args[..], &self.builtins)
    }

    fn eval_variable(&self, name: &String) -> Result<Object, SyntaxError> {
        let mut frame = &self.frame;
        loop {
            match frame.local.get(name) {
                Some(value) => return Ok(value.to_owned()),
                None => match &frame.parent {
                    Some(parent_frame) => {
                        frame = parent_frame;
                    }
                    None => panic!("Unbound variable: {}", name),
                },
            }
        }
    }
}

fn eval_str(source: &str) -> Result<Object, SyntaxError> {
    let builtins = Builtins::new();
    let expr = parse_str(source)?;
    Env::new(&builtins).eval(&expr).map_err(|e| e.add_context(source))
}

fn eval_ok(source: &str) -> Object {
    eval_str(source).unwrap()
}

#[test]
fn eval_decimal() {
    assert_eq!(eval_ok("123").integer(), 123);
}

#[test]
fn eval_bad_decimal() {
    assert_eq!(
        eval_str("1x3"),
        Err(SyntaxError {
            span: 0..3,
            problem: "Malformed number",
            context: concat!("001 1x3\n", "    ^^^ Malformed number\n").to_string()
        })
    );
}

#[test]
fn eval_hex() {
    assert_eq!(eval_ok("0xFFFF").integer(), 0xFFFF);
}

#[test]
fn eval_bad_hex() {
    assert_eq!(
        eval_str("0x1x3"),
        Err(SyntaxError {
            span: 0..5,
            problem: "Malformed hexadecimal number",
            context: concat!("001 0x1x3\n", "    ^^^^^ Malformed hexadecimal number\n").to_string()
        })
    );
}

#[test]
fn eval_binary() {
    assert_eq!(eval_ok("0b101").integer(), 0b101);
}

#[test]
fn eval_bad_binary() {
    assert_eq!(
        eval_str("0b123"),
        Err(SyntaxError {
            span: 0..5,
            problem: "Malformed binary number",
            context: concat!("001 0b123\n", "    ^^^^^ Malformed binary number\n").to_string()
        })
    );
}

#[test]
fn eval_float() {
    assert_eq!(eval_ok("1.2").float(), 1.2);
}

#[test]
fn eval_bad_float() {
    assert_eq!(
        eval_str("1.2.3"),
        Err(SyntaxError {
            span: 0..5,
            problem: "Malformed number",
            context: concat!("001 1.2.3\n", "    ^^^^^ Malformed number\n").to_string()
        })
    );
}

#[test]
fn eval_let1() {
    assert_eq!(eval_ok("let x = 42, x").integer(), 42);
}

#[test]
fn eval_let2() {
    assert_eq!(eval_ok("let x = 1, let x = 42, x").integer(), 42);
}

#[test]
fn eval_let3() {
    assert_eq!(eval_ok("let x = 42, let y = 1, x").integer(), 42);
}

#[test]
fn eval_arith1() {
    assert_eq!(eval_ok("1 + 1").integer(), 2);
}

#[test]
fn eval_arith2() {
    assert_eq!(eval_ok("1 + 1 * 2").integer(), 3);
}

#[test]
fn eval_div1() {
    assert_eq!(eval_ok("10 / 5").integer(), 2);
}

#[test]
fn eval_div2() {
    assert_eq!(eval_ok("10.0 / 5.0").float(), 2.0);
}

#[test]
fn eval_div3() {
    assert_eq!(eval_ok("10.0 / 5").float(), 2.0);
}

#[test]
fn eval_div4() {
    assert_eq!(eval_ok("10 / 5.0").float(), 2.0);
}

#[test]
fn eval_sub1() {
    assert_eq!(eval_ok("10 - 5").integer(), 5);
}

#[test]
fn eval_sub2() {
    assert_eq!(eval_ok("10.0 - 5.0").float(), 5.0);
}

#[test]
fn eval_sub3() {
    assert_eq!(eval_ok("10.0 - 5").float(), 5.0);
}

#[test]
fn eval_sub4() {
    assert_eq!(eval_ok("10 - 5.0").float(), 5.0);
}

#[test]
fn eval_add1() {
    assert_eq!(eval_ok("10 + 5").integer(), 15);
}

#[test]
fn eval_add2() {
    assert_eq!(eval_ok("10.0 + 5.0").float(), 15.0);
}

#[test]
fn eval_add3() {
    assert_eq!(eval_ok("10.0 + 5").float(), 15.0);
}

#[test]
fn eval_add4() {
    assert_eq!(eval_ok("10 + 5.0").float(), 15.0);
}

#[test]
fn eval_mul1() {
    assert_eq!(eval_ok("10 * 5").integer(), 50);
}

#[test]
fn eval_mul2() {
    assert_eq!(eval_ok("10.0 * 5.0").float(), 50.0);
}

#[test]
fn eval_mul3() {
    assert_eq!(eval_ok("10.0 * 5").float(), 50.0);
}

#[test]
fn eval_mul4() {
    assert_eq!(eval_ok("10 * 5.0").float(), 50.0);
}
