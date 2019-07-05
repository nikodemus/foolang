use std::borrow::ToOwned;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::objects2::{Builtins, Object, Value};
use crate::parse::{parse_str, Expr, Literal, SyntaxError};

struct Env<'a> {
    builtins: &'a Builtins,
    frame: Rc<Frame>,
}

#[derive(PartialEq, Debug)]
pub struct Frame {
    local: RefCell<HashMap<String, Object>>,
    parent: Option<Rc<Frame>>,
}

impl<'a> Env<'a> {
    pub fn new(builtins: &Builtins) -> Env {
        Env::from_parts(builtins, HashMap::new(), None)
    }

    pub fn eval(&self, expr: &Expr) -> Result<Object, SyntaxError> {
        match expr {
            Expr::Assign(left, right) => self.eval_assign(left, right),
            Expr::Bind(name, value, body) => self.eval_bind(name, value, body),
            Expr::Block(_, params, body) => self.eval_block(params, body),
            Expr::Const(_, literal) => self.eval_literal(literal),
            Expr::Send(_, selector, receiver, args) => self.eval_send(selector, receiver, args),
            Expr::Seq(exprs) => self.eval_seq(exprs),
            Expr::Var(_, name) => self.eval_variable(name),
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
                local: RefCell::new(local),
                parent,
            }),
        }
    }

    fn bind(&self, name: &String, value: Object) -> Env {
        let mut local = HashMap::new();
        local.insert(name.to_owned(), value);
        Env::from_parts(self.builtins, local, Some(Rc::clone(&self.frame)))
    }

    fn eval_assign(&self, name: &String, right: &Box<Expr>) -> Result<Object, SyntaxError> {
        let value = self.eval(right)?;

        let mut frame = &self.frame;
        loop {
            match frame.local.borrow_mut().get_mut(name) {
                Some(place) => {
                    *place = value.clone();
                    return Ok(value);
                }
                None => match &frame.parent {
                    Some(parent_frame) => {
                        frame = parent_frame;
                    }
                    // FIXME: Should be an exception, but a panic -- or better yet,
                    // a syntax-error at parse time...
                    None => panic!("Unbound variable in assignment: {}", name),
                },
            }
        }
    }

    fn eval_bind(&self, name: &String, value: &Expr, body: &Expr) -> Result<Object, SyntaxError> {
        self.bind(name, self.eval(value)?).eval(body)
    }

    fn eval_block(&self, params: &Vec<String>, body: &Expr) -> Result<Object, SyntaxError> {
        Ok(self.builtins.make_closure(Rc::clone(&self.frame), params.to_owned(), body.to_owned()))
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

    fn eval_seq(&self, exprs: &Vec<Expr>) -> Result<Object, SyntaxError> {
        // FIXME: false or nothing
        let mut result = self.builtins.make_integer(0);
        for expr in exprs {
            result = self.eval(expr)?
        }
        Ok(result)
    }

    fn eval_variable(&self, name: &String) -> Result<Object, SyntaxError> {
        let mut frame = &self.frame;
        loop {
            match frame.local.borrow().get(name) {
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

pub fn apply(receiver: &Object, args: &[&Object], builtins: &Builtins) -> Value {
    let closure = receiver.closure();
    // KLUDGE: I'm blind. I would think that iterating over args with IntoIterator
    // would give me an iterator over &Object, but I get &&Object -- so to_owned x 2.
    let locals: HashMap<String, Object> = closure
        .params
        .iter()
        .map(String::clone)
        .zip(args.into_iter().map(|obj| obj.to_owned().to_owned()))
        .collect();
    let env = Env::from_parts(builtins, locals, Some(Rc::clone(&closure.env)));
    env.eval(&closure.body)
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

#[test]
fn eval_assign() {
    assert_eq!(eval_ok("let x = 1, x = x + 1, let y = x, y").integer(), 2);
}

#[test]
fn eval_int_as_int() {
    assert_eq!(eval_ok("42 asInteger").integer(), 42);
}

#[test]
fn eval_float_as_float() {
    assert_eq!(eval_ok("42.3 asFloat").float(), 42.3);
}

#[test]
fn eval_int_as_float() {
    assert_eq!(eval_ok("42 asFloat").float(), 42.0);
}

#[test]
fn eval_float_as_int() {
    assert_eq!(eval_ok("42.1 asInteger").integer(), 42);
    assert_eq!(eval_ok("41.9 asInteger").integer(), 42);
}

#[test]
fn eval_unary() {
    assert_eq!(eval_ok("42 asFloat asInteger").integer(), 42);
}

#[test]
fn eval_keyword() {
    assert_eq!(eval_ok("15 gcd: 100").integer(), 5);
}

#[test]
fn eval_closure1() {
    assert_eq!(eval_ok("let x = 41, { x + 1 } value").integer(), 42);
}

#[test]
fn eval_closure2() {
    assert_eq!(eval_ok("let x = 41, { |y| x + y } value: 1").integer(), 42);
}

#[test]
fn eval_closure3() {
    assert_eq!(
        eval_ok(
            "let thunk = { let counter = 0, { counter = counter + 1, counter } } value,
                        thunk value + thunk value"
        )
        .integer(),
        3
    );
}
