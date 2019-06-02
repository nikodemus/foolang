use crate::ast::{
    Cascade, ClassDescription, Expr, Identifier, Literal, Method, MethodDescription, ProgramElement,
};
use crate::objects::*;
use lazy_static::lazy_static;
use std::borrow::ToOwned;
use std::collections::HashMap;

type MethodFunc = fn(Object, Vec<Object>) -> Object;

type MethodTable = HashMap<String, MethodImpl>;

#[derive(Clone)]
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
    fn add_method(&mut self, class: &ClassId, name: &str, f: Method) {
        self.methods[class.0].insert(String::from(name), MethodImpl::Evaluator(f));
    }
}

lazy_static! {
    static ref CLASSES: ClassInfo = {
        let mut info = ClassInfo { names: HashMap::new(), methods: Vec::new(), };

        // NOTE: Alphabetic order matches objects.rs

        let array = info.add_class("Array");
        assert_eq!(array, CLASS_ARRAY, "Bad classId for Array");

        let array = info.add_class("Block");
        assert_eq!(array, CLASS_BLOCK, "Bad classId for Block");

        let character = info.add_class("Character");
        assert_eq!(character, CLASS_CHARACTER, "Bad classId for Character");

        let character = info.add_class("Class");
        assert_eq!(character, CLASS_CLASS, "Bad classId for Class");

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

pub struct GlobalEnv {
    classes: ClassInfo,
    variables: HashMap<String, Object>,
}

impl GlobalEnv {
    pub fn new() -> GlobalEnv {
        GlobalEnv {
            classes: CLASSES.clone(),
            variables: GLOBALS.clone(),
        }
    }
    fn add_class(&mut self, name: &str, slots: Vec<Identifier>) {
        if self.variables.contains_key(name) {
            panic!("{} alredy exists!", name);
        }
        // Our metaclasses don't currently exist as actual objects!
        let metaname = format!("[{}]", name);
        let metaid = self.classes.add_class(&metaname);
        let id = self.classes.add_class(name);
        let class = Object::make_class(metaid, id, name, slots);
        self.variables.insert(name.to_string(), class);
    }
    fn send(&self, receiver: Object, selector: &Identifier, args: Vec<Object>) -> Object {
        match receiver.datum {
            Datum::Block(_) => block_apply(receiver, args, self),
            _ => self
                .classes
                .find_method(&receiver.class, &selector.0)
                .invoke(receiver, args, self),
        }
    }
    pub fn load(&mut self, program: Vec<ProgramElement>) {
        for p in program {
            match p {
                ProgramElement::Class(ClassDescription { name, slots }) => {
                    self.add_class(&name.0, slots);
                }
                ProgramElement::InstanceMethod(MethodDescription { class, method }) => {
                    match self.variables.get(&class.0) {
                        Some(Object {
                            class: _,
                            datum: Datum::Class(classobj),
                        }) => {
                            let mname = method.selector.0.clone();
                            self.classes.add_method(&classobj.id, &mname, method);
                        }
                        None => panic!("Cannot install method in unknown class: {}", class.0),
                        _ => panic!("Cannot install methods in non-class objects."),
                    }
                }
                ProgramElement::ClassMethod(MethodDescription { class, method }) => {
                    match self.variables.get(&class.0) {
                        Some(Object {
                            class: classid,
                            datum: _,
                        }) => {
                            let mname = method.selector.0.clone();
                            self.classes.add_method(&classid, &mname, method);
                        }
                        None => panic!("Cannot install class-method in unknown class: {}", class.0),
                    }
                }
            }
        }
    }
    pub fn eval(&self, expr: Expr) -> Object {
        eval_in_env1(expr, &mut Lexenv::new(), self)
    }
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
    fn add_temporaries(&mut self, temps: Vec<Identifier>) {
        self.names.extend(temps);
        for _ in 0..(self.names.len() - self.values.len()) {
            // FIXME: nil
            self.values.push(Object::make_integer(0));
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
    Evaluator(Method),
}

impl MethodImpl {
    fn invoke(&self, receiver: Object, args: Vec<Object>, global: &GlobalEnv) -> Object {
        match self {
            MethodImpl::Builtin(func) => func(receiver, args),
            MethodImpl::Evaluator(method) => {
                let mut env = Lexenv::from(method.parameters.clone(), args);
                env.add_temporaries(method.temporaries.clone());
                for stm in method.statements.iter() {
                    if let Eval::Return(val) = eval_in_env(stm.to_owned(), &mut env, global) {
                        return val;
                    }
                }
                return receiver;
            }
        }
    }
}

enum Eval {
    Result(Object, Object),
    Return(Object),
}

pub fn eval(expr: Expr) -> Object {
    GlobalEnv::new().eval(expr)
}

fn eval_in_env1(expr: Expr, env: &mut Lexenv, global: &GlobalEnv) -> Object {
    match eval_in_env(expr, env, global) {
        Eval::Result(value, _) => value,
        Eval::Return(_) => panic!("Unexpected return!"),
    }
}

fn eval_in_env(expr: Expr, env: &mut Lexenv, global: &GlobalEnv) -> Eval {
    fn dup(x: Object) -> Eval {
        Eval::Result(x.clone(), x)
    }
    match expr {
        Expr::Constant(lit) => dup(eval_literal(lit)),
        Expr::Variable(Identifier(s)) => {
            if let Some(value) = env.find(&s) {
                return dup(value.to_owned());
            }
            match global.variables.get(&s) {
                Some(g) => dup(g.to_owned()),
                None => panic!("Unbound variable: {}", s),
            }
        }
        Expr::Assign(Identifier(s), expr) => match env.index(&s) {
            Some(idx) => {
                let val = eval_in_env1(*expr, env, global);
                env.set_index(idx, val.clone());
                dup(val)
            }
            None => panic!(
                "Cannot assign to an unbound variable: {}. Available names: {:?}",
                s, env.names
            ),
        },
        Expr::Unary(expr, selector) => {
            let val = eval_in_env1(*expr, env, global);
            Eval::Result(global.send(val.clone(), &selector, vec![]), val)
        }
        Expr::Binary(left, selector, right) => {
            let val = eval_in_env1(*left, env, global);
            Eval::Result(
                global.send(
                    val.clone(),
                    &selector,
                    vec![eval_in_env1(*right, env, global)],
                ),
                val,
            )
        }
        Expr::Keyword(expr, selector, args) => {
            let val = eval_in_env1(*expr, env, global);
            Eval::Result(
                global.send(
                    val.clone(),
                    &selector,
                    args.into_iter()
                        .map(|arg| eval_in_env1(arg, env, global))
                        .collect(),
                ),
                val,
            )
        }
        Expr::Block(b) => dup(Object::into_block(b)),
        Expr::Cascade(expr, cascade) => {
            if let Eval::Result(_, receiver) = eval_in_env(*expr, env, global) {
                Eval::Result(
                    eval_cascade(receiver.clone(), cascade, env, global),
                    receiver,
                )
            } else {
                panic!("Unexpected return in cascade expression.")
            }
        }
        Expr::ArrayCtor(exprs) => {
            let mut data = Vec::new();
            for e in exprs.iter() {
                data.push(eval_in_env1(e.to_owned(), env, global));
            }
            dup(Object::make_array(&data))
        }
        Expr::Return(expr) => Eval::Return(eval_in_env1(*expr, env, global)),
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

fn block_apply(receiver: Object, mut args: Vec<Object>, global: &GlobalEnv) -> Object {
    let mut res = receiver.clone();
    match receiver.datum {
        Datum::Block(blk) => {
            // FIXME: use lexenv methods!
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
                // FIXME: returns
                res = eval_in_env1(stm.to_owned(), &mut env, global);
            }
            res
        }
        _ => panic!("Bad receiver for block_apply!"),
    }
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

fn eval_cascade(
    receiver: Object,
    cascade: Vec<Cascade>,
    env: &mut Lexenv,
    global: &GlobalEnv,
) -> Object {
    let mut value = receiver.clone();
    for thing in cascade.iter() {
        value = match thing {
            Cascade::Unary(selector) => global.send(receiver.clone(), selector, vec![]),
            Cascade::Binary(selector, expr) => global.send(
                receiver.clone(),
                selector,
                vec![eval_in_env1(expr.to_owned(), env, global)],
            ),
            Cascade::Keyword(selector, exprs) => global.send(
                receiver.clone(),
                selector,
                exprs
                    .iter()
                    .map(|x| eval_in_env1(x.to_owned(), env, global))
                    .collect(),
            ),
        }
    }
    value
}
