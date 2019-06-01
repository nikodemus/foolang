use crate::ast::{Cascade, Expr, Identifier, Literal};
use crate::objects::Object;
use lazy_static::lazy_static;
use std::borrow::ToOwned;
use std::collections::HashMap;
use std::sync::Arc;

pub type MethodFunc = fn(Object, Vec<Object>) -> Object;

// type MethodTable = HashMap<String,MethodImpl>;

struct Lexenv<'a> {
    names: Vec<Identifier>,
    values: Vec<Object>,
    parent: Option<&'a mut Lexenv<'a>>,
}

impl<'a> Lexenv<'a> {
    fn new() -> Lexenv<'a> {
        Lexenv {
            names: vec![],
            values: vec![],
            parent: None,
        }
    }
    fn from(names: Vec<Identifier>, values: Vec<Object>) -> Lexenv<'a> {
        Lexenv {
            names,
            values,
            parent: None,
        }
    }
    fn index(&self, name: &str) -> Option<usize> {
        self.names.iter().position(|id| &id.0 == name)
    }
    fn set_index(&mut self, index: usize, value: Object) {
        self.values[index] = value;
    }
    fn find(&self, name: &str) -> Option<&Object> {
        match self.names.iter().position(|id| &id.0 == name) {
            Some(p) => self.values.get(p),
            None => match &self.parent {
                Some(env) => env.find(name),
                None => None,
            },
        }
    }
}

#[derive(Debug, Clone)]
enum MethodImpl {
    Builtin(MethodFunc),
}

pub fn eval(expr: Expr) -> Object {
    eval_in_env1(expr, &mut Lexenv::new())
}

fn eval_in_env1(expr: Expr, env: &mut Lexenv) -> Object {
    let (val, _) = eval_in_env(expr, env);
    val
}

fn eval_in_env(expr: Expr, env: &mut Lexenv) -> (Object, Object) {
    fn dup(x: Object) -> (Object, Object) {
        (x.clone(), x)
    }
    match expr {
        Expr::Constant(lit) => dup(eval_literal(lit)),
        Expr::Variable(Identifier(s)) => {
            if let Some(value) = env.find(&s) {
                return dup(value.to_owned());
            }
            match GLOBALS.get(&s) {
                Some(g) => dup(g.to_owned()),
                None => panic!("Unbound variable: {}", s),
            }
        }
        Expr::Assign(Identifier(s), expr) => match env.index(&s) {
            Some(idx) => {
                let val = eval_in_env1(*expr, env);
                env.set_index(idx, val.clone());
                dup(val)
            }
            None => panic!(
                "Cannot assign to an unbound variable: {}. Available names: {:?}",
                s, env.names
            ),
        },
        Expr::Unary(expr, selector) => {
            let receiver = eval_in_env1(*expr, env);
            (send_unary(receiver.clone(), &selector), receiver)
        }
        Expr::Binary(left, selector, right) => {
            let receiver = eval_in_env1(*left, env);
            (
                send_binary(receiver.clone(), &selector, eval_in_env1(*right, env)),
                receiver,
            )
        }
        Expr::Keyword(expr, selector, args) => {
            let receiver = eval_in_env1(*expr, env);
            (
                send_keyword(
                    receiver.clone(),
                    &selector,
                    args.into_iter().map(|arg| eval_in_env1(arg, env)).collect(),
                ),
                receiver,
            )
        }
        Expr::Block(b) => dup(Object::Block(Arc::new(b))),
        Expr::Cascade(expr, cascade) => {
            let (_, receiver) = eval_in_env(*expr, env);
            (eval_cascade(receiver.clone(), cascade, env), receiver)
        }
        Expr::Return(_expr) => unimplemented!("TODO: return"),
    }
}

fn method_neg(receiver: Object, args: Vec<Object>) -> Object {
    assert!(args.len() == 0);
    match receiver {
        Object::Integer(i) => Object::Integer(-i),
        Object::Float(i) => Object::Float(-i),
        _ => panic!("Bad receiver for neg!"),
    }
}

fn method_gcd(receiver: Object, args: Vec<Object>) -> Object {
    assert!(args.len() == 1);
    match receiver {
        Object::Integer(i) => match args[0] {
            Object::Integer(j) => Object::Integer(num::integer::gcd(i, j)),
            _ => panic!("Non-integer in gcd!"),
        },
        _ => panic!("Bad receiver for builtin gcd!"),
    }
}

fn method_plus(receiver: Object, args: Vec<Object>) -> Object {
    assert!(args.len() == 1);
    match receiver {
        Object::Integer(i) => match args[0] {
            Object::Integer(j) => Object::Integer(i + j),
            Object::Float(j) => Object::Float(i as f64 + j),
            _ => panic!("Bad argument for plus!"),
        },
        Object::Float(i) => match args[0] {
            Object::Integer(j) => Object::Float(i + j as f64),
            Object::Float(j) => Object::Float(i + j),
            _ => panic!("Bad argument for plus!"),
        },
        _ => panic!("Bad receiver for plus!"),
    }
}

fn method_minus(receiver: Object, args: Vec<Object>) -> Object {
    assert!(args.len() == 1);
    match receiver {
        Object::Integer(i) => match args[0] {
            Object::Integer(j) => Object::Integer(i - j),
            Object::Float(j) => Object::Float(i as f64 - j),
            _ => panic!("Bad argument for minus!"),
        },
        Object::Float(i) => match args[0] {
            Object::Integer(j) => Object::Float(i - j as f64),
            Object::Float(j) => Object::Float(i - j),
            _ => panic!("Bad argument for minus!"),
        },
        _ => panic!("Bad receiver for minus!"),
    }
}

fn method_mul(receiver: Object, args: Vec<Object>) -> Object {
    assert!(args.len() == 1);
    match receiver {
        Object::Integer(i) => match args[0] {
            Object::Integer(j) => Object::Integer(i * j),
            Object::Float(j) => Object::Float(i as f64 * j),
            _ => panic!("Bad argument for mul!"),
        },
        Object::Float(i) => match args[0] {
            Object::Integer(j) => Object::Float(i * j as f64),
            Object::Float(j) => Object::Float(i * j),
            _ => panic!("Bad argument for mul!"),
        },
        _ => panic!("Bad receiver for mul!"),
    }
}

fn method_block_apply(receiver: Object, mut args: Vec<Object>) -> Object {
    let mut res = receiver.clone();
    match receiver {
        Object::Block(blk) => {
            assert!(args.len() == blk.parameters.len());
            let mut names = blk.parameters.clone();
            names.append(&mut blk.temporaries.clone());
            for _ in 0..(names.len() - args.len()) {
                // FIXME...
                args.push(Object::Integer(0));
            }
            // FIXME: Should refer to outer scope...
            let mut env = Lexenv::from(names, args);
            for stm in blk.statements.iter() {
                res = eval_in_env1(stm.to_owned(), &mut env);
            }
            res
        }
        _ => panic!("Bad receiver for block_apply!"),
    }
}

trait MethodTable {
    fn add_builtin(&mut self, name: &str, f: MethodFunc);
}

impl MethodTable for HashMap<String, MethodImpl> {
    fn add_builtin(&mut self, name: &str, f: MethodFunc) {
        self.insert(String::from(name), MethodImpl::Builtin(f));
    }
}

lazy_static! {
    static ref GLOBALS: HashMap<String, Object> = {
        let mut m: HashMap<String, Object> = HashMap::new();
        m.insert(String::from("PI"), Object::Float(std::f64::consts::PI));
        m
    };
    static ref INTEGER_METHODS: HashMap<String, MethodImpl> = {
        let mut m: HashMap<String, MethodImpl> = HashMap::new();
        m.add_builtin("neg", method_neg);
        m.add_builtin("gcd:", method_gcd);
        m.add_builtin("*", method_mul);
        m.add_builtin("+", method_plus);
        m.add_builtin("-", method_minus);
        m
    };
    static ref FLOAT_METHODS: HashMap<String, MethodImpl> = {
        let mut m: HashMap<String, MethodImpl> = HashMap::new();
        m.add_builtin("neg", method_neg);
        m.add_builtin("*", method_mul);
        m.add_builtin("+", method_plus);
        m.add_builtin("-", method_minus);
        m
    };
    static ref STRING_METHODS: HashMap<String, MethodImpl> = {
        let m: HashMap<String, MethodImpl> = HashMap::new();
        m
    };
    static ref CHARACTER_METHODS: HashMap<String, MethodImpl> = {
        let m: HashMap<String, MethodImpl> = HashMap::new();
        m
    };
    static ref SYMBOL_METHODS: HashMap<String, MethodImpl> = {
        let m: HashMap<String, MethodImpl> = HashMap::new();
        m
    };
    static ref ARRAY_METHODS: HashMap<String, MethodImpl> = {
        let m: HashMap<String, MethodImpl> = HashMap::new();
        m
    };
}

fn find_method(receiver: &Object, selector: &Identifier) -> MethodImpl {
    // println!("find_method {:?} {:?}", receiver, selector);
    let item = match receiver {
        Object::Block(_) => return MethodImpl::Builtin(method_block_apply),
        Object::Integer(_) => INTEGER_METHODS.get(&selector.0),
        Object::Float(_) => FLOAT_METHODS.get(&selector.0),
        Object::String(_) => STRING_METHODS.get(&selector.0),
        Object::Symbol(_) => SYMBOL_METHODS.get(&selector.0),
        Object::Character(_) => CHARACTER_METHODS.get(&selector.0),
        Object::Array(_) => ARRAY_METHODS.get(&selector.0),
    };
    match item {
        Some(method) => method.to_owned(),
        None => panic!("No method {} on {:?}", selector.0, receiver),
    }
}

fn invoke(method: MethodImpl, receiver: Object, args: Vec<Object>) -> Object {
    match method {
        MethodImpl::Builtin(func) => func(receiver, args),
    }
}

fn send_unary(receiver: Object, selector: &Identifier) -> Object {
    invoke(find_method(&receiver, selector), receiver, vec![])
}

fn send_binary(receiver: Object, selector: &Identifier, arg: Object) -> Object {
    invoke(find_method(&receiver, selector), receiver, vec![arg])
}

fn send_keyword(receiver: Object, selector: &Identifier, args: Vec<Object>) -> Object {
    invoke(find_method(&receiver, selector), receiver, args)
}

fn eval_literal(lit: Literal) -> Object {
    match lit {
        Literal::Integer(x) => Object::Integer(x),
        Literal::Float(x) => Object::Float(x),
        Literal::String(s) => Object::String(Arc::new(s)),
        Literal::Symbol(s) => Object::Symbol(Arc::new(s)),
        Literal::Character(s) => Object::Character(Arc::new(s)),
        Literal::Array(s) => Object::Array(Arc::new(s.into_iter().map(eval_literal).collect())),
    }
}

fn eval_cascade(receiver: Object, cascade: Vec<Cascade>, env: &mut Lexenv) -> Object {
    let mut value = receiver.clone();
    for thing in cascade.iter() {
        value = match thing {
            Cascade::Unary(selector) => send_unary(receiver.clone(), selector),
            Cascade::Binary(selector, expr) => send_binary(
                receiver.clone(),
                selector,
                eval_in_env1(expr.to_owned(), env),
            ),
            Cascade::Keyword(selector, exprs) => send_keyword(
                receiver.clone(),
                selector,
                exprs
                    .iter()
                    .map(|x| eval_in_env1(x.to_owned(), env))
                    .collect(),
            ),
        }
    }
    value
}
