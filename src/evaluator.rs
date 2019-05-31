use crate::ast::{Expr, Identifier, Literal};
use crate::objects::Object;
use lazy_static::lazy_static;
use std::collections::HashMap;

pub type MethodFunction = fn(Object, Vec<Object>) -> Object;

#[derive(Debug)]
enum MethodImpl {
    Builtin(MethodFunction),
}

pub fn eval(expr: Expr) -> Object {
    match expr {
        Expr::Constant(lit) => eval_literal(lit),
        Expr::Unary(expr, selector) => send_unary(eval(*expr), selector),
        Expr::Binary(left, selector, right) => send_binary(eval(*left), selector, eval(*right)),
        _ => unimplemented!("eval({:?})", expr),
    }
}

fn method_neg(receiver: Object, args: Vec<Object>) -> Object {
    assert!(args.len() == 0);
    match receiver {
        Object::Integer(i) => Object::Integer(-i),
        Object::Float(i) => Object::Float(-i),
    }
}

fn method_plus(receiver: Object, args: Vec<Object>) -> Object {
    assert!(args.len() == 1);
    match receiver {
        Object::Integer(i) => match args[0] {
            Object::Integer(j) => Object::Integer(i + j),
            Object::Float(j) => Object::Float(i as f64 + j),
        },
        Object::Float(i) => match args[0] {
            Object::Integer(j) => Object::Float(i + j as f64),
            Object::Float(j) => Object::Float(i + j),
        },
    }
}

fn method_minus(receiver: Object, args: Vec<Object>) -> Object {
    assert!(args.len() == 1);
    match receiver {
        Object::Integer(i) => match args[0] {
            Object::Integer(j) => Object::Integer(i - j),
            Object::Float(j) => Object::Float(i as f64 - j),
        },
        Object::Float(i) => match args[0] {
            Object::Integer(j) => Object::Float(i - j as f64),
            Object::Float(j) => Object::Float(i - j),
        },
    }
}

lazy_static! {
    static ref INTEGER_METHODS: HashMap<String, MethodFunction> = {
        let mut m: HashMap<String, MethodFunction> = HashMap::new();
        m.insert(String::from("neg"), method_neg);
        m.insert(String::from("+"), method_plus);
        m.insert(String::from("-"), method_minus);
        m
    };
    static ref FLOAT_METHODS: HashMap<String, MethodFunction> = {
        let mut m: HashMap<String, MethodFunction> = HashMap::new();
        m.insert(String::from("neg"), method_neg);
        m.insert(String::from("+"), method_plus);
        m.insert(String::from("-"), method_minus);
        m
    };
}

fn find_method(receiver: &Object, selector: Identifier) -> MethodImpl {
    match receiver {
        Object::Integer(_) => MethodImpl::Builtin(INTEGER_METHODS[&selector.0]),
        Object::Float(_) => MethodImpl::Builtin(FLOAT_METHODS[&selector.0]),
    }
}

fn invoke(method: MethodImpl, receiver: Object, args: Vec<Object>) -> Object {
    match method {
        MethodImpl::Builtin(func) => func(receiver, args),
    }
}

fn send_unary(receiver: Object, selector: Identifier) -> Object {
    invoke(find_method(&receiver, selector), receiver, vec![])
}

fn send_binary(receiver: Object, selector: Identifier, arg: Object) -> Object {
    invoke(find_method(&receiver, selector), receiver, vec![arg])
}

fn eval_literal(lit: Literal) -> Object {
    match lit {
        Literal::Integer(x) => Object::Integer(x),
        Literal::Float(x) => Object::Float(x),
        _ => unimplemented!("eval_literal({:?})", lit),
    }
}
