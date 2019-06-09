use crate::ast::{
    Cascade, ClassDescription, Expr, Identifier, Literal, Method, MethodDescription, ProgramElement,
};
use crate::objects::*;
use crate::parser::parse_expr;
use crate::parser::parse_program;
use lazy_static::lazy_static;
use std::borrow::ToOwned;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use std::sync::Mutex;

pub fn eval_str(code: &str) -> Object {
    eval(parse_expr(code))
}

pub fn load_str(code: &str) -> GlobalEnv {
    let mut env = GlobalEnv::new();
    env.load(parse_program(code));
    env
}

type MethodFunc = fn(Object, Vec<Object>, &GlobalEnv) -> Eval;

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
    static ref BUILTIN_ENV: GlobalEnv =  {
        let mut env = GlobalEnv {
            classes: ClassInfo {
                names: HashMap::new(),
                slots: Vec::new(),
                methods: Vec::new(),
            },
            variables: HashMap::new(),
        };
        // NOTE: Alphabetic order matches objects.rs
        let (array, _) = env.add_builtin_class("Array");
        assert_eq!(array, CLASS_ARRAY, "Bad classId for Array");

        let (class, _) = env.add_builtin_class("Boolean");
        assert_eq!(class, CLASS_BOOLEAN, "Bad classId for Boolean");
        env.classes.add_builtin(&class, "ifTrue:", method_boolean_iftrue);

        let (character, _) = env.add_builtin_class("Character");
        assert_eq!(character, CLASS_CHARACTER, "Bad classId for Character");

        let (character, _) = env.add_builtin_class("Class");
        assert_eq!(character, CLASS_CLASS, "Bad classId for Class");

        let (class, _) = env.add_builtin_class("Closure");
        assert_eq!(class, CLASS_CLOSURE, "Bad classId for Closure");
        env.classes.add_builtin(&class, "repeat", method_closure_repeat);
        env.classes.add_builtin(&class, "repeatWhileFalse", method_closure_repeatwhilefalse);
        env.classes.add_builtin(&class, "value:", method_closure_apply);
        env.classes.add_builtin(&class, "value", method_closure_apply);
        env.classes.add_builtin(&class, "value:value:", method_closure_apply);
        env.classes.add_builtin(&class, "value:value:value:", method_closure_apply);

        let (class, _) = env.add_builtin_class("Float");
        assert_eq!(class, CLASS_FLOAT);
        env.classes.add_builtin(&class, "neg", method_number_neg);
        env.classes.add_builtin(&class, "*", method_number_mul);
        env.classes.add_builtin(&class, "+", method_number_plus);
        env.classes.add_builtin(&class, "-", method_number_minus);
        env.classes.add_builtin(&class, "<", method_number_lt);
        env.classes.add_builtin(&class, ">", method_number_gt);
        env.classes.add_builtin(&class, "==", method_number_eq);

        let (class, _meta) = env.add_builtin_class("Input");
        assert_eq!(class, CLASS_INPUT);
        env.classes.add_builtin(&class, "readline", method_input_readline);

        let (class, _) = env.add_builtin_class("Integer");
        assert_eq!(class, CLASS_INTEGER);
        env.classes.add_builtin(&class, "neg", method_number_neg);
        env.classes.add_builtin(&class, "gcd:", method_integer_gcd);
        env.classes.add_builtin(&class, "*", method_number_mul);
        env.classes.add_builtin(&class, "+", method_number_plus);
        env.classes.add_builtin(&class, "-", method_number_minus);
        env.classes.add_builtin(&class, "<", method_number_lt);
        env.classes.add_builtin(&class, ">", method_number_gt);
        env.classes.add_builtin(&class, "==", method_number_eq);

        let (class, _meta) = env.add_builtin_class("Output");
        assert_eq!(class, CLASS_OUTPUT);
        env.classes.add_builtin(&class, "print:", method_output_print);
        env.classes.add_builtin(&class, "newline", method_output_newline);
        env.classes.add_builtin(&class, "flush", method_output_flush);

        let (string, meta) = env.add_builtin_class("String");
        assert_eq!(string, CLASS_STRING);
        env.classes.add_builtin(&meta, "new", class_method_string_new);
        env.classes.add_builtin(&string, "append:", method_string_append);
        env.classes.add_builtin(&string, "clear", method_string_clear);

        let (symbol, _) = env.add_builtin_class("Symbol");
        assert_eq!(symbol, CLASS_SYMBOL);

        let (class, meta) = env.add_builtin_class("System");
        assert_eq!(class, CLASS_SYSTEM);
        env.classes.add_builtin(&meta, "stdin", class_method_system_stdin);
        env.classes.add_builtin(&meta, "stdout", class_method_system_stdout);

        /* GLOBALS */

        env.variables.insert(String::from("PI"), Object::make_float(std::f64::consts::PI));
        // FIXME: should be literals instead!
        env.variables.insert(String::from("true"), Object::make_boolean(true));
        env.variables.insert(String::from("false"), Object::make_boolean(false));

        env
    };
}

#[derive(Clone)]
pub struct GlobalEnv {
    classes: ClassInfo,
    variables: HashMap<String, Object>,
}

impl GlobalEnv {
    pub fn new() -> GlobalEnv {
        BUILTIN_ENV.clone()
    }
    fn find_method(&self, classid: &ClassId, name: &str) -> MethodImpl {
        self.classes.find_method(classid, name)
    }
    fn find_slot(&self, class: &ClassId, name: &str) -> Option<usize> {
        self.classes.slots[class.0]
            .iter()
            .position(|id| &id.0 == name)
    }
    fn add_builtin_class(&mut self, name: &str) -> (ClassId, ClassId) {
        if self.variables.contains_key(name) {
            panic!("{} already exists!", name);
        }
        // Our metaclasses don't currently exist as actual objects!
        let metaname = format!("#<metaclass {}>", name);
        let metaid = self.classes.add_class(&metaname, vec![]);
        let id = self.classes.add_class(name, vec![]);
        let class = Object::make_class(metaid.clone(), id.clone(), name);
        self.classes.add_builtin(&metaid, "help:", method_help);
        self.variables.insert(name.to_string(), class);
        (id, metaid)
    }
    fn add_class(&mut self, name: &str, slots: Vec<Identifier>) {
        if self.variables.contains_key(name) {
            panic!("{} already exists!", name);
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
    ) -> Eval {
        self.classes
            .find_method(&receiver.class, &selector.0)
            .invoke(receiver, args, env, self)
    }
    pub fn load_file(&mut self, fname: &str) {
        self.load(parse_program(
            fs::read_to_string(fname)
                .expect("Could not load file.")
                .as_str(),
        ))
    }
    pub fn eval_str(&self, text: &str) -> Object {
        self.eval(parse_expr(text))
    }
    fn load_program_element(&mut self, program_element: ProgramElement) -> Object {
        match program_element {
            ProgramElement::Class(ClassDescription { name, slots }) => {
                self.add_class(&name.0, slots);
                Object::make_symbol(&name.0)
            }
            ProgramElement::InstanceMethod(MethodDescription { class, method }) => {
                match self.variables.get(&class.0) {
                    Some(Object {
                        class: _,
                        datum: Datum::Class(classobj),
                    }) => {
                        let mname = method.selector.0.clone();
                        self.classes.add_method(&classobj.id, &mname, method);
                        Object::make_symbol(&mname)
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
                        Object::make_symbol(&mname)
                    }
                    None => panic!("Cannot install class-method in unknown class: {}", class.0),
                }
            }
        }
    }
    pub fn load(&mut self, program: Vec<ProgramElement>) {
        for p in program {
            self.load_program_element(p);
        }
    }
    pub fn eval(&self, expr: Expr) -> Object {
        match eval_in_env(expr, &Lexenv::null(), self) {
            Eval::Result(value, _) => value,
            Eval::Return(val, to) => {
                panic!("Unexpected return!\n  value = {:?}\n  to = {:?}", val, to)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Lexenv(pub Arc<LexenvFrame>);

impl PartialEq for Lexenv {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Lexenv {
    fn null() -> Lexenv {
        Lexenv(Arc::new(LexenvFrame {
            receiver: None,
            names: vec![],
            values: Mutex::new(vec![]),
            parent: None,
            method_env: None,
        }))
    }

    fn method(
        receiver: &Object,
        tmps: &Vec<Identifier>,
        params: &Vec<Identifier>,
        args: &Vec<Object>,
    ) -> Lexenv {
        let mut names = params.to_owned();
        let mut values = args.to_owned();
        for name in tmps.iter() {
            names.push(name.clone());
            // FIXME: Should be nil
            values.push(Object::make_integer(0));
        }
        Lexenv(Arc::new(LexenvFrame {
            receiver: Some(receiver.to_owned()),
            names,
            values: Mutex::new(values),
            parent: None,
            method_env: None,
        }))
    }

    pub fn parent(&self) -> Option<Lexenv> {
        self.0.parent.clone()
    }

    pub fn method_env(&self) -> Option<Lexenv> {
        self.0.method_env.clone()
    }

    pub fn extend(
        &self,
        tmps: &Vec<Identifier>,
        params: &Vec<Identifier>,
        args: &Vec<Object>,
    ) -> Lexenv {
        let mut names = params.to_owned();
        let mut values = args.to_owned();
        for name in tmps.iter() {
            names.push(name.clone());
            // FIXME: Should be nil
            values.push(Object::make_integer(0));
        }
        Lexenv(Arc::new(LexenvFrame {
            receiver: self.0.receiver.to_owned(),
            names,
            values: Mutex::new(values),
            parent: Some(self.to_owned()),
            method_env: match self.0.method_env {
                None => Some(self.to_owned()),
                Some(_) => self.0.method_env.clone(),
            },
        }))
    }

    fn index(&self, name: &str) -> Option<usize> {
        self.0.names.iter().position(|id| &id.0 == name)
    }

    fn try_set_variable(&self, name: &str, val: &Object) -> bool {
        match self.index(name) {
            Some(idx) => {
                let mut values = self.0.values.lock().unwrap();
                values[idx] = val.to_owned();
                true
            }
            None => match &self.0.parent {
                Some(parent) => parent.try_set_variable(name, val),
                None => false,
            },
        }
    }

    fn find(&self, name: &str) -> Option<Object> {
        match self.index(name) {
            Some(p) => {
                let values = self.0.values.lock().unwrap();
                values.get(p).map(|x| x.to_owned())
            }
            None => match &self.0.parent {
                Some(env) => env.find(name),
                None => None,
            },
        }
    }
}

#[derive(Debug)]
pub struct LexenvFrame {
    pub receiver: Option<Object>,
    pub names: Vec<Identifier>,
    pub values: Mutex<Vec<Object>>,
    pub parent: Option<Lexenv>,
    pub method_env: Option<Lexenv>,
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
    ) -> Eval {
        match self {
            MethodImpl::Builtin(func) => func(receiver, args, global),
            MethodImpl::Evaluator(method) => {
                let env = Lexenv::method(&receiver, &method.temporaries, &method.parameters, &args);
                for stm in method.statements.iter() {
                    if let Eval::Return(val, to) = eval_in_env(stm.to_owned(), &env, global) {
                        if to == Some(env) || to == None {
                            return Eval::Result(val, receiver);
                        } else {
                            return Eval::Return(val, to);
                        }
                    }
                }
                return Eval::Result(receiver.clone(), receiver);
            }
        }
    }
}

enum Eval {
    Result(Object, Object),
    Return(Object, Option<Lexenv>),
}

pub fn eval(expr: Expr) -> Object {
    GlobalEnv::new().eval(expr)
}

fn make_method_result(receiver: Object, result: Object) -> Eval {
    Eval::Result(result, receiver)
}

fn make_result(x: Object) -> Eval {
    Eval::Result(x.clone(), x)
}

fn eval_in_env(expr: Expr, env: &Lexenv, global: &GlobalEnv) -> Eval {
    match expr {
        Expr::Constant(lit) => make_result(eval_literal(lit)),
        Expr::Variable(Identifier(s)) => {
            if s == "self" {
                match &env.0.receiver {
                    None => panic!("Cannot use self outside methods."),
                    Some(me) => return make_result(me.clone()),
                }
            }
            if let Some(value) = env.find(&s) {
                return make_result(value.to_owned());
            }
            if let Some(obj) = &env.0.receiver {
                if let Some(idx) = global.find_slot(&obj.class, &s) {
                    return make_result(obj.slot(idx));
                }
            }
            match global.variables.get(&s) {
                Some(g) => make_result(g.to_owned()),
                None => panic!("Unbound variable: {}", s),
            }
        }
        Expr::Assign(Identifier(s), expr) => {
            let val = match eval_in_env(*expr, env, global) {
                Eval::Result(val, _) => val,
                Eval::Return(res, to) => return Eval::Return(res, to),
            };
            if env.try_set_variable(&s, &val) {
                return make_result(val);
            }
            if let Some(obj) = &env.0.receiver {
                if let Some(idx) = global.find_slot(&obj.class, &s) {
                    obj.set_slot(idx, val.clone());
                    return make_result(val);
                }
            }
            panic!("Cannot assign to an unbound variable: {}.", s)
        }
        Expr::Send(expr, selector, args) => {
            let res = eval_in_env(*expr, env, global);
            match &res {
                Eval::Return(_, _) => res,
                Eval::Result(val, _) => {
                    let mut argvals = Vec::new();
                    for arg in args.into_iter() {
                        match eval_in_env(arg, env, global) {
                            Eval::Return(r, to) => return Eval::Return(r, to),
                            Eval::Result(argval, _) => {
                                argvals.push(argval);
                            }
                        }
                    }
                    global.send(val.to_owned(), &selector, argvals, env)
                }
            }
        }
        Expr::Block(b) => make_result(Object::into_closure(b, env)),
        Expr::Cascade(expr, cascade) => {
            if let Eval::Result(_, receiver) = eval_in_env(*expr, env, global) {
                eval_cascade(receiver.clone(), cascade, env, global)
            } else {
                panic!("Unexpected return in cascade expression.")
            }
        }
        Expr::ArrayCtor(exprs) => {
            let mut data = Vec::new();
            for e in exprs.iter() {
                let elt = match eval_in_env(e.to_owned(), env, global) {
                    Eval::Result(val, _) => val,
                    Eval::Return(res, to) => return Eval::Return(res, to),
                };
                data.push(elt);
            }
            make_result(Object::make_array(&data))
        }
        Expr::Return(expr) => match eval_in_env(*expr, env, global) {
            Eval::Result(val, _) => Eval::Return(val, env.method_env()),
            Eval::Return(val, to) => Eval::Return(val, to),
        },
    }
}

fn class_method_system_stdin(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 0);
    make_method_result(receiver, Object::make_input(Box::new(std::io::stdin())))
}

fn class_method_system_stdout(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 0);
    make_method_result(receiver, Object::make_output(Box::new(std::io::stdout())))
}

fn method_input_readline(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 0);
    let line = match &receiver.datum {
        Datum::Input(input) => Object::into_string(input.read_line()),
        _ => panic!("Bad receiver for Input readline: {}", receiver),
    };
    make_method_result(receiver, line)
}

fn method_output_print(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 1);
    match &receiver.datum {
        Datum::Output(out) => match &args[0].datum {
            Datum::String(s) => {
                out.write(s.lock().unwrap().as_bytes());
            }
            _ => {
                panic!("Bad argument to Output print: {}", args[0]);
            }
        },
        _ => panic!("Bad receiver for Output print: {}", receiver),
    }
    make_method_result(receiver.clone(), receiver)
}

fn method_output_newline(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 0);
    match &receiver.datum {
        Datum::Output(out) => {
            out.write("\n".as_bytes());
        }
        _ => panic!("Bad receiver for Output newline: {}", receiver),
    }
    make_method_result(receiver.clone(), receiver)
}

fn method_output_flush(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 0);
    match &receiver.datum {
        Datum::Output(out) => {
            out.flush();
        }
        _ => panic!("Bad receiver for Output flush: {}", receiver),
    }
    make_method_result(receiver.clone(), receiver)
}

fn class_method_string_new(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 0);
    make_method_result(receiver, Object::make_string(""))
}

fn method_string_append(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 1);
    match (&receiver.datum, &args[0].datum) {
        (Datum::String(s), Datum::String(more)) => {
            s.lock().unwrap().push_str(more.to_string().as_str());
            make_result(receiver)
        }
        _ => panic!("Bad arguments to 'String append:': #{:?}", args),
    }
}

fn method_string_clear(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 0);
    match &receiver.datum {
        Datum::String(s) => {
            s.lock().unwrap().clear();
            make_result(receiver)
        }
        _ => panic!("Bad receiver in 'String clear': #{:?}", args),
    }
}

fn method_boolean_iftrue(receiver: Object, args: Vec<Object>, global: &GlobalEnv) -> Eval {
    assert!(args.len() == 1);
    match receiver.datum {
        Datum::Boolean(boolean) => {
            if boolean {
                match &args[0].datum {
                    Datum::Closure(closure) => closure_apply(receiver, closure, &args, global),
                    _ => panic!("Bad argument to Boolean 'ifTrue:': {}", args[0]),
                }
            } else {
                make_result(receiver)
            }
        }
        _ => panic!("Bad receiver to Boolean 'ifTrue:': {}", receiver),
    }
}

fn method_number_neg(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 0);
    make_method_result(
        receiver.clone(),
        match receiver.datum {
            Datum::Integer(i) => Object::make_integer(-i),
            Datum::Float(i) => Object::make_float(-i),
            _ => panic!("Bad receiver for neg!"),
        },
    )
}

fn method_integer_gcd(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 1);
    match receiver.datum.clone() {
        Datum::Integer(i) => match args[0].datum {
            Datum::Integer(j) => {
                make_method_result(receiver, Object::make_integer(num::integer::gcd(i, j)))
            }
            _ => panic!("Non-integer in gcd!"),
        },
        _ => panic!("Bad receiver for builtin gcd!"),
    }
}

fn method_number_mul(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 1);
    make_method_result(
        receiver.clone(),
        match receiver.datum {
            Datum::Integer(i) => match args[0].datum {
                Datum::Integer(j) => Object::make_integer(i * j),
                Datum::Float(j) => Object::make_float((i as f64) * j),
                _ => panic!("Bad argument to Integer *: {}", args[0]),
            },
            Datum::Float(i) => match args[0].datum {
                Datum::Integer(j) => Object::make_float(i * (j as f64)),
                Datum::Float(j) => Object::make_float(i * j),
                _ => panic!("Bad argument to Float *: {}", args[0]),
            },
            _ => panic!("Bad receiver in method_number_mul: {}", receiver),
        },
    )
}

fn method_number_plus(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 1);
    make_method_result(
        receiver.clone(),
        match receiver.datum {
            Datum::Integer(i) => match args[0].datum {
                Datum::Integer(j) => Object::make_integer(i + j),
                Datum::Float(j) => Object::make_float((i as f64) + j),
                _ => panic!("Bad argument to Integer +: {}", args[0]),
            },
            Datum::Float(i) => match args[0].datum {
                Datum::Integer(j) => Object::make_float(i + (j as f64)),
                Datum::Float(j) => Object::make_float(i + j),
                _ => panic!("Bad argument to Float +: {}", args[0]),
            },
            _ => panic!("Bad receiver in method_number_plus: {}", receiver),
        },
    )
}

fn method_number_minus(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 1);
    make_method_result(
        receiver.clone(),
        match receiver.datum {
            Datum::Integer(i) => match args[0].datum {
                Datum::Integer(j) => Object::make_integer(i - j),
                Datum::Float(j) => Object::make_float((i as f64) - j),
                _ => panic!("Bad argument to Integer -: {}", args[0]),
            },
            Datum::Float(i) => match args[0].datum {
                Datum::Integer(j) => Object::make_float(i - (j as f64)),
                Datum::Float(j) => Object::make_float(i - j),
                _ => panic!("Bad argument to Float -: {}", args[0]),
            },
            _ => panic!("Bad receiver in method_number_minus: {}", receiver),
        },
    )
}

fn method_number_lt(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 1);
    make_method_result(
        receiver.clone(),
        match receiver.datum {
            Datum::Integer(i) => match args[0].datum {
                Datum::Integer(j) => Object::make_boolean(i < j),
                Datum::Float(j) => Object::make_boolean((i as f64) < j),
                _ => panic!("Bad argument to Integer <: {}", args[0]),
            },
            Datum::Float(i) => match args[0].datum {
                Datum::Integer(j) => Object::make_boolean(i < (j as f64)),
                Datum::Float(j) => Object::make_boolean(i < j),
                _ => panic!("Bad argument to Float <: {}", args[0]),
            },
            _ => panic!("Bad receiver in method_number_lt: {}", receiver),
        },
    )
}

fn method_number_gt(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 1);
    make_method_result(
        receiver.clone(),
        match receiver.datum {
            Datum::Integer(i) => match args[0].datum {
                Datum::Integer(j) => Object::make_boolean(i > j),
                Datum::Float(j) => Object::make_boolean((i as f64) > j),
                _ => panic!("Bad argument to Integer >: {}", args[0]),
            },
            Datum::Float(i) => match args[0].datum {
                Datum::Integer(j) => Object::make_boolean(i > (j as f64)),
                Datum::Float(j) => Object::make_boolean(i > j),
                _ => panic!("Bad argument to Float >: {}", args[0]),
            },
            _ => panic!("Bad receiver in method_number_gt: {}", receiver),
        },
    )
}

fn method_number_eq(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 1);
    make_method_result(
        receiver.clone(),
        match receiver.datum {
            Datum::Integer(i) => match args[0].datum {
                Datum::Integer(j) => Object::make_boolean(i == j),
                Datum::Float(j) => Object::make_boolean((i as f64) == j),
                _ => panic!("Bad argument to Integer ==: {}", args[0]),
            },
            Datum::Float(i) => match args[0].datum {
                Datum::Integer(j) => Object::make_boolean(i == (j as f64)),
                Datum::Float(j) => Object::make_boolean(i == j),
                _ => panic!("Bad argument to Float ==: {}", args[0]),
            },
            _ => panic!("Bad receiver in method_number_eq: {}", receiver),
        },
    )
}

fn method_create_instance(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 1);
    if let Datum::Class(classobj) = &receiver.datum {
        if let Datum::Array(vec) = &args[0].datum {
            return make_method_result(
                receiver.clone(),
                Object::make_instance(classobj.id.to_owned(), vec.to_vec()),
            );
        }
    }
    panic!("Cannot create instance out of a non-class object!")
}

fn method_help(receiver: Object, args: Vec<Object>, global: &GlobalEnv) -> Eval {
    assert!(args.len() == 1);
    match &args[0].datum {
        Datum::Symbol(name) => {
            if let MethodImpl::Evaluator(m) = &global.find_method(&receiver.class, &name) {
                if let Some(s) = &m.docstring {
                    return make_method_result(receiver, Object::make_string(s));
                }
            }
        }
        _ => panic!("Bad argument to help:!"),
    }
    make_method_result(receiver, Object::make_string("No help available."))
}

fn method_closure_apply(receiver: Object, args: Vec<Object>, global: &GlobalEnv) -> Eval {
    match receiver.datum.clone() {
        Datum::Closure(closure) => closure_apply(receiver, &closure, &args, global),
        _ => panic!("Bad receiver for block apply!"),
    }
}

fn method_closure_repeat(receiver: Object, args: Vec<Object>, global: &GlobalEnv) -> Eval {
    match receiver.datum.clone() {
        Datum::Closure(closure) => loop {
            if let Eval::Return(val, to) = closure_apply(receiver.clone(), &closure, &args, global)
            {
                return Eval::Return(val, to);
            }
        },
        _ => panic!("Bad receiver for block apply!"),
    }
}

fn method_closure_repeatwhilefalse(
    receiver: Object,
    args: Vec<Object>,
    global: &GlobalEnv,
) -> Eval {
    match receiver.datum.clone() {
        Datum::Closure(closure) => loop {
            match closure_apply(receiver.clone(), &closure, &args, global) {
                Eval::Return(val, to) => return Eval::Return(val, to),
                Eval::Result(val, _) => {
                    if let Datum::Boolean(false) = val.datum {
                        // repeat
                        ;
                    } else {
                        return make_method_result(receiver.clone(), val);
                    }
                }
            }
        },
        _ => panic!("Bad receiver for block apply!"),
    }
}

fn closure_apply(
    receiver: Object,
    closure: &Arc<ClosureObject>,
    args: &Vec<Object>,
    global: &GlobalEnv,
) -> Eval {
    let mut result = receiver.clone();
    let env = closure
        .env
        .extend(&closure.block.temporaries, &closure.block.parameters, args);
    for stm in closure.block.statements.iter() {
        match eval_in_env(stm.to_owned(), &env, global) {
            Eval::Result(value, _) => {
                result = value;
            }
            Eval::Return(value, to) => return Eval::Return(value, to),
        }
    }
    Eval::Result(result, receiver)
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

fn eval_cascade(receiver: Object, cascade: Vec<Cascade>, env: &Lexenv, global: &GlobalEnv) -> Eval {
    let mut value = receiver.clone();
    for thing in cascade.iter() {
        let res = match thing {
            Cascade::Message(selector, exprs) => {
                let mut vals = Vec::new();
                for exp in exprs.iter() {
                    let val = match eval_in_env(exp.to_owned(), env, global) {
                        Eval::Result(val, _) => val,
                        Eval::Return(val, to) => return Eval::Return(val, to),
                    };
                    vals.push(val);
                }
                global.send(receiver.clone(), selector, vals, env)
            }
        };
        match res {
            Eval::Result(val, _) => {
                value = val;
            }
            Eval::Return(val, to) => return Eval::Return(val, to),
        }
    }
    Eval::Result(value, receiver)
}
