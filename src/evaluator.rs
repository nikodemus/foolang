use crate::ast::{
    Cascade, ClassDescription, Expr, Identifier, Literal, Method, MethodDescription, ProgramElement,
};
use crate::objects::*;
use lazy_static::lazy_static;
use std::borrow::ToOwned;
use std::collections::HashMap;

type MethodFunc = fn(Object, Vec<Object>, &GlobalEnv) -> Object;

type MethodTable = HashMap<String, MethodImpl>;

#[derive(Clone)]
struct ClassInfo {
    names: HashMap<String, ClassId>,
    slots: Vec<Vec<Identifier>>,
    methods: Vec<MethodTable>,
}

impl ClassInfo {
    fn add_class(&mut self, name: &str, slots: Vec<Identifier>) -> ClassId {
        if self.names.contains_key(name) {
            panic!("Cannot redefine class! {} already exists.", name);
        } else {
            let id = ClassId(self.methods.len());
            self.names.insert(String::from(name), id.clone());
            self.slots.push(slots);
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
            None => {
                let methods: Vec<_> = self.methods[class.0].keys().collect();
                panic!(
                    "No method {} on {}\nAvailable methods: {:?}",
                    name,
                    self.class_name(class),
                    methods
                )
            }
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
        let mut info = ClassInfo {
            names: HashMap::new(),
            slots: Vec::new(),
            methods: Vec::new(),
        };

        // NOTE: Alphabetic order matches objects.rs

        let array = info.add_class("Array", vec![]);
        assert_eq!(array, CLASS_ARRAY, "Bad classId for Array");

        let array = info.add_class("Block", vec![]);
        assert_eq!(array, CLASS_BLOCK, "Bad classId for Block");

        let character = info.add_class("Character", vec![]);
        assert_eq!(character, CLASS_CHARACTER, "Bad classId for Character");

        let character = info.add_class("Class", vec![]);
        assert_eq!(character, CLASS_CLASS, "Bad classId for Class");

        let float = info.add_class("Float", vec![]);
        assert_eq!(float, CLASS_FLOAT);
        info.add_builtin(&float, "neg", method_neg);
        info.add_builtin(&float, "*", method_mul);
        info.add_builtin(&float, "+", method_plus);
        info.add_builtin(&float, "-", method_minus);

        let integer = info.add_class("Integer", vec![]);
        assert_eq!(integer, CLASS_INTEGER);
        info.add_builtin(&integer, "neg", method_neg);
        info.add_builtin(&integer, "gcd:", method_gcd);
        info.add_builtin(&integer, "*", method_mul);
        info.add_builtin(&integer, "+", method_plus);
        info.add_builtin(&integer, "-", method_minus);

        let string = info.add_class("String", vec![]);
        assert_eq!(string, CLASS_STRING);

        let symbol = info.add_class("Symbol", vec![]);
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
    fn find_method(&self, classid: &ClassId, name: &str) -> MethodImpl {
        self.classes.find_method(classid, name)
    }
    fn find_slot(&self, class: &ClassId, name: &str) -> Option<usize> {
        self.classes.slots[class.0]
            .iter()
            .position(|id| &id.0 == name)
    }
    fn add_class(&mut self, name: &str, slots: Vec<Identifier>) {
        if self.variables.contains_key(name) {
            panic!("{} alredy exists!", name);
        }
        // Our metaclasses don't currently exist as actual objects!
        let metaname = format!("#<metaclass {}>", name);
        let metaid = self.classes.add_class(&metaname, vec![]);
        let id = self.classes.add_class(name, slots.clone());
        let class = Object::make_class(metaid.clone(), id.clone(), name);
        self.classes.add_builtin(&metaid, "help:", method_help);
        self.classes
            .add_builtin(&metaid, "createInstance:", method_create_instance);
        self.variables.insert(name.to_string(), class);
    }
    fn send(
        &self,
        receiver: Object,
        selector: &Identifier,
        args: Vec<Object>,
        env: &Lexenv,
    ) -> Object {
        match receiver.datum {
            Datum::Block(_) => method_block_apply(receiver, args, env, self),
            _ => self
                .classes
                .find_method(&receiver.class, &selector.0)
                .invoke(receiver, args, env, self),
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
    receiver: Option<Object>,
    names: Vec<Identifier>,
    values: Vec<Object>,
    parent: Option<&'a mut Lexenv<'a>>,
}

impl<'a> Lexenv<'a> {
    fn new() -> Lexenv<'a> {
        Lexenv {
            receiver: None,
            names: vec![],
            values: vec![],
            parent: None,
        }
    }
    fn from(receiver: Option<Object>, names: Vec<Identifier>, values: Vec<Object>) -> Lexenv<'a> {
        Lexenv {
            receiver,
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

#[derive(Clone)]
enum MethodImpl {
    Builtin(MethodFunc),
    Evaluator(Method),
}

impl MethodImpl {
    fn invoke(
        &self,
        receiver: Object,
        args: Vec<Object>,
        _env: &Lexenv,
        global: &GlobalEnv,
    ) -> Object {
        match self {
            MethodImpl::Builtin(func) => func(receiver, args, global),
            MethodImpl::Evaluator(method) => {
                let mut env = Lexenv::from(Some(receiver.clone()), method.parameters.clone(), args);
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
            if s == "self" {
                match &env.receiver {
                    None => panic!("Cannot use self outside methods."),
                    Some(me) => return dup(me.clone()),
                }
            }
            if let Some(value) = env.find(&s) {
                return dup(value.to_owned());
            }
            if let Some(obj) = &env.receiver {
                if let Some(idx) = global.find_slot(&obj.class, &s) {
                    return dup(obj.slot(idx));
                }
            }
            match global.variables.get(&s) {
                Some(g) => dup(g.to_owned()),
                None => panic!("Unbound variable: {}", s),
            }
        }
        Expr::Assign(Identifier(s), expr) => {
            let val = eval_in_env1(*expr, env, global);
            match env.index(&s) {
                Some(idx) => {
                    env.set_index(idx, val.clone());
                    dup(val)
                }
                None => {
                    if let Some(obj) = &env.receiver {
                        if let Some(idx) = global.find_slot(&obj.class, &s) {
                            obj.set_slot(idx, val.clone());
                            return dup(val);
                        }
                    }
                    panic!(
                        "Cannot assign to an unbound variable: {}. Available names: {:?}",
                        s, env.names
                    )
                }
            }
        }
        Expr::Send(expr, selector, args) => {
            let val = eval_in_env1(*expr, env, global);
            Eval::Result(
                global.send(
                    val.clone(),
                    &selector,
                    args.into_iter()
                        .map(|arg| eval_in_env1(arg, env, global))
                        .collect(),
                    env,
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

fn method_neg(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Object {
    assert!(args.len() == 0);
    match receiver.datum {
        Datum::Integer(i) => Object::make_integer(-i),
        Datum::Float(i) => Object::make_float(-i),
        _ => panic!("Bad receiver for neg!"),
    }
}

fn method_gcd(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Object {
    assert!(args.len() == 1);
    match receiver.datum {
        Datum::Integer(i) => match args[0].datum {
            Datum::Integer(j) => Object::make_integer(num::integer::gcd(i, j)),
            _ => panic!("Non-integer in gcd!"),
        },
        _ => panic!("Bad receiver for builtin gcd!"),
    }
}

fn method_plus(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Object {
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

fn method_minus(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Object {
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

fn method_mul(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Object {
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

fn method_create_instance(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Object {
    assert!(args.len() == 1);
    if let Datum::Class(classobj) = receiver.datum {
        if let Datum::Array(vec) = &args[0].datum {
            return Object::make_instance(classobj.id.clone(), vec.to_vec());
        }
    }
    panic!("Cannot create instance out of a non-class object!")
}

fn method_help(receiver: Object, args: Vec<Object>, global: &GlobalEnv) -> Object {
    assert!(args.len() == 1);
    match &args[0].datum {
        Datum::Symbol(name) => {
            if let MethodImpl::Evaluator(m) = &global.find_method(&receiver.class, &name) {
                if let Some(s) = &m.docstring {
                    return Object::make_string(s);
                }
            }
        }
        _ => panic!("Bad argument to help:!"),
    }
    Object::make_string("No help available.")
}

fn method_block_apply(
    receiver: Object,
    mut args: Vec<Object>,
    env: &Lexenv,
    global: &GlobalEnv,
) -> Object {
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
            let mut env = Lexenv::from(env.receiver.clone(), names, args);
            for stm in blk.statements.iter() {
                // FIXME: returns
                res = eval_in_env1(stm.to_owned(), &mut env, global);
            }
            res
        }
        _ => panic!("Bad receiver for block apply!"),
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
            Cascade::Message(selector, exprs) => global.send(
                receiver.clone(),
                selector,
                exprs
                    .iter()
                    .map(|x| eval_in_env1(x.to_owned(), env, global))
                    .collect(),
                env,
            ),
        }
    }
    value
}
