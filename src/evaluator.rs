use crate::ast::{Cascade, Expr, Identifier, Literal};
use crate::objects::*;
use lazy_static::lazy_static;
use std::borrow::ToOwned;
use std::collections::HashMap;

type MethodFunc = fn(Object, Vec<Object>) -> Object;

type MethodTable = HashMap<String, MethodImpl>;

struct ClassInfo {
    names: HashMap<String, ClassId>,
    methods: Vec<MethodTable>,
}

impl ClassInfo {
    fn add_class(&mut self, name: &str) -> ClassId {
        if self.names.contains_key(name) {
            panic!("Cannot redefine class! {} already exists.", name);
        } else {
            let id = ClassId(self.methods.len());
            self.names.insert(String::from(name), id.clone());
            self.methods.push(MethodTable::new());
            id
        }
    }
    fn class_name(&self, class: &ClassId) -> String {
        for (name, id) in self.names.iter() {
            if id == class {
                return name.to_owned();
            }
        }
        panic!(
            "ClassId not in names?! id={}, size={}",
            class.0,
            self.methods.len()
        );
    }
    fn find_method(&self, class: &ClassId, name: &str) -> MethodImpl {
        match self.methods[class.0].get(name) {
            Some(method) => method.to_owned(),
            None => panic!("No method {} on {}", name, self.class_name(class)),
        }
    }
    fn add_builtin(&mut self, class: &ClassId, name: &str, f: MethodFunc) {
        self.methods[class.0].insert(String::from(name), MethodImpl::Builtin(f));
    }
}

lazy_static! {
    static ref CLASSES: ClassInfo = {
        let mut info = ClassInfo { names: HashMap::new(), methods: Vec::new() };

        // NOTE: Alphabetic order matches objects.rs

        let array = info.add_class("Array");
        assert_eq!(array, CLASS_ARRAY, "Bad classId for Array");

        let array = info.add_class("Block");
        assert_eq!(array, CLASS_BLOCK, "Bad classId for Block");

        let character = info.add_class("Character");
        assert_eq!(character, CLASS_CHARACTER, "Bad classId for Character");

        let float = info.add_class("Float");
        assert_eq!(float, CLASS_FLOAT);
        info.add_builtin(&float, "neg", method_neg);
        info.add_builtin(&float, "*", method_mul);
        info.add_builtin(&float, "+", method_plus);
        info.add_builtin(&float, "-", method_minus);

        let integer = info.add_class("Integer");
        assert_eq!(integer, CLASS_INTEGER);
        info.add_builtin(&integer, "neg", method_neg);
        info.add_builtin(&integer, "gcd:", method_gcd);
        info.add_builtin(&integer, "*", method_mul);
        info.add_builtin(&integer, "+", method_plus);
        info.add_builtin(&integer, "-", method_minus);

        let string = info.add_class("String");
        assert_eq!(string, CLASS_STRING);

        let symbol = info.add_class("Symbol");
        assert_eq!(symbol, CLASS_SYMBOL);

        info
    };
    static ref GLOBALS: HashMap<String, Object> = {
        let mut m: HashMap<String, Object> = HashMap::new();
        m.insert(String::from("PI"), Object::make_float(std::f64::consts::PI));
        m
    };
}

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
        Expr::Block(b) => dup(Object::into_block(b)),
        Expr::Cascade(expr, cascade) => {
            let (_, receiver) = eval_in_env(*expr, env);
            (eval_cascade(receiver.clone(), cascade, env), receiver)
        }
        Expr::ArrayCtor(exprs) => {
            let mut data = Vec::new();
            for e in exprs.iter() {
                data.push(eval_in_env1(e.to_owned(), env));
            }
            dup(Object::make_array(&data))
        }
        Expr::Return(_expr) => unimplemented!("TODO: return"),
    }
}

fn method_neg(receiver: Object, args: Vec<Object>) -> Object {
    assert!(args.len() == 0);
    match receiver.datum {
        Datum::Integer(i) => Object::make_integer(-i),
        Datum::Float(i) => Object::make_float(-i),
        _ => panic!("Bad receiver for neg!"),
    }
}

fn method_gcd(receiver: Object, args: Vec<Object>) -> Object {
    assert!(args.len() == 1);
    match receiver.datum {
        Datum::Integer(i) => match args[0].datum {
            Datum::Integer(j) => Object::make_integer(num::integer::gcd(i, j)),
            _ => panic!("Non-integer in gcd!"),
        },
        _ => panic!("Bad receiver for builtin gcd!"),
    }
}

fn method_plus(receiver: Object, args: Vec<Object>) -> Object {
    assert!(args.len() == 1);
    match receiver.datum {
        Datum::Integer(i) => match args[0].datum {
            Datum::Integer(j) => Object::make_integer(i + j),
            Datum::Float(j) => Object::make_float(i as f64 + j),
            _ => panic!("Bad argument for plus!"),
        },
        Datum::Float(i) => match args[0].datum {
            Datum::Integer(j) => Object::make_float(i + j as f64),
            Datum::Float(j) => Object::make_float(i + j),
            _ => panic!("Bad argument for plus!"),
        },
        _ => panic!("Bad receiver for plus!"),
    }
}

fn method_minus(receiver: Object, args: Vec<Object>) -> Object {
    assert!(args.len() == 1);
    match receiver.datum {
        Datum::Integer(i) => match args[0].datum {
            Datum::Integer(j) => Object::make_integer(i - j),
            Datum::Float(j) => Object::make_float(i as f64 - j),
            _ => panic!("Bad argument for minus!"),
        },
        Datum::Float(i) => match args[0].datum {
            Datum::Integer(j) => Object::make_float(i - j as f64),
            Datum::Float(j) => Object::make_float(i - j),
            _ => panic!("Bad argument for minus!"),
        },
        _ => panic!("Bad receiver for minus!"),
    }
}

fn method_mul(receiver: Object, args: Vec<Object>) -> Object {
    assert!(args.len() == 1);
    match receiver.datum {
        Datum::Integer(i) => match args[0].datum {
            Datum::Integer(j) => Object::make_integer(i * j),
            Datum::Float(j) => Object::make_float(i as f64 * j),
            _ => panic!("Bad argument for mul!"),
        },
        Datum::Float(i) => match args[0].datum {
            Datum::Integer(j) => Object::make_float(i * j as f64),
            Datum::Float(j) => Object::make_float(i * j),
            _ => panic!("Bad argument for mul!"),
        },
        _ => panic!("Bad receiver for mul!"),
    }
}

fn method_block_apply(receiver: Object, mut args: Vec<Object>) -> Object {
    let mut res = receiver.clone();
    match receiver.datum {
        Datum::Block(blk) => {
            assert!(args.len() == blk.parameters.len());
            let mut names = blk.parameters.clone();
            names.append(&mut blk.temporaries.clone());
            for _ in 0..(names.len() - args.len()) {
                // FIXME...
                args.push(Object::make_integer(0));
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

fn find_method(receiver: &Object, selector: &Identifier) -> MethodImpl {
    // println!("find_method {:?} {:?}", receiver, selector);
    match receiver.datum {
        Datum::Block(_) => MethodImpl::Builtin(method_block_apply),
        _ => CLASSES.find_method(&receiver.class, &selector.0),
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
        Literal::Integer(x) => Object::make_integer(x),
        Literal::Float(x) => Object::make_float(x),
        Literal::String(s) => Object::into_string(s),
        Literal::Symbol(s) => Object::into_symbol(s),
        Literal::Character(s) => Object::into_character(s),
        Literal::Array(s) => Object::into_array(s.into_iter().map(eval_literal).collect()),
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
