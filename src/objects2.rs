use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::io::Read;
use std::io::Write;
use std::rc::Rc;

use crate::eval;
use crate::eval::{Env, Frame};
use crate::parse::{ClassDefinition, Expr, Literal, Parser, Var};
use crate::tokenstream::Span;
use crate::unwind::Unwind;

use crate::classes;

pub type Eval = Result<Object, Unwind>;

pub trait Source {
    fn source(self, span: &Span) -> Self;
    fn context(self, source: &str) -> Self;
}

impl Source for Eval {
    fn source(mut self, span: &Span) -> Self {
        if let Err(unwind) = &mut self {
            unwind.add_span(span);
        }
        self
    }
    fn context(self, context: &str) -> Self {
        if let Err(unwind) = self {
            Err(unwind.with_context(context))
        } else {
            self
        }
    }
}

type MethodFunction = fn(&Object, &[Object], &Foolang) -> Eval;

pub enum Method {
    Primitive(MethodFunction),
    Interpreter(Closure),
    Reader(usize),
}

#[derive(Debug, PartialEq)]
pub struct Slot {
    pub index: usize,
    pub vtable: Option<Rc<Vtable>>,
}

pub struct Vtable {
    pub name: String,
    pub methods: HashMap<String, Method>,
    pub slots: HashMap<String, Slot>,
}

impl Vtable {
    pub fn new(class: &str) -> Vtable {
        Vtable {
            name: class.to_string(),
            methods: HashMap::new(),
            slots: HashMap::new(),
        }
    }

    pub fn def(&mut self, name: &str, method: MethodFunction) {
        self.methods.insert(name.to_string(), Method::Primitive(method));
    }

    pub fn add_method(&mut self, selector: &str, method: Closure) {
        self.methods.insert(selector.to_string(), Method::Interpreter(method));
    }

    pub fn add_reader(&mut self, selector: &str, index: usize) {
        self.methods.insert(selector.to_string(), Method::Reader(index));
    }

    pub fn add_slot(&mut self, name: &str, index: usize, vtable: Option<Rc<Vtable>>) {
        self.slots.insert(
            name.to_string(),
            Slot {
                index,
                vtable,
            },
        );
    }

    pub fn selectors(&self) -> Vec<String> {
        let mut selectors = vec![];
        for key in self.methods.keys() {
            selectors.push(key.clone());
        }
        selectors
    }

    pub fn get(&self, name: &str) -> Option<&Method> {
        self.methods.get(name)
    }
}

impl fmt::Debug for Vtable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Vtable[{}]", self.name)
    }
}

impl PartialEq for Vtable {
    fn eq(&self, other: &Self) -> bool {
        self as *const _ == other as *const _
    }
}

#[derive(PartialEq, Clone)]
pub struct Object {
    pub vtable: Rc<Vtable>,
    pub datum: Datum,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Arg {
    pub span: Span,
    pub name: String,
    pub vtable: Option<Rc<Vtable>>,
}

impl Arg {
    pub fn new(span: Span, name: String, vtable: Option<Rc<Vtable>>) -> Arg {
        Arg {
            span,
            name,
            vtable,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Closure {
    env: Option<Frame>,
    pub params: Vec<Arg>,
    pub body: Expr,
}

impl Closure {
    pub fn env(&self) -> Option<Frame> {
        self.env.clone()
    }
}

#[derive(PartialEq)]
pub struct Class {
    pub instance_vtable: Rc<Vtable>,
}

// XXX: The interesting thing about this is that it doesn't give
// access to the global environment... but I actually like that.
#[derive(PartialEq)]
pub struct Compiler {
    pub foolang: Foolang,
    pub source: RefCell<String>,
    pub expr: RefCell<Expr>,
}

pub struct Input {
    pub name: String,
    stream: RefCell<Box<dyn Read>>,
    buffer: RefCell<Vec<u8>>,
}

impl PartialEq for Input {
    fn eq(&self, other: &Self) -> bool {
        self as *const _ == other as *const _
    }
}

impl Input {
    pub fn readline(&self) -> Option<String> {
        let mut stream = self.stream.borrow_mut();
        let mut buf = self.buffer.borrow_mut();
        const LF: u8 = 10;
        const CR: u8 = 13;
        loop {
            // UTF-8 bytes after first always have the high bit set,
            // so this is safe.
            if let Some(newline) = buf.iter().position(|x| *x == LF) {
                // Check for preceding carriage return.
                let end = if newline > 0 && buf[newline - 1] == CR {
                    newline - 1
                } else {
                    newline
                };
                let line =
                    std::str::from_utf8(&buf[0..end]).expect("Invalid UTF-8 in input").to_string();
                buf.drain(0..=newline);
                return Some(line);
            }
            buf.reserve(1024);
            let len = buf.len();
            let cap = buf.capacity();
            unsafe {
                buf.set_len(cap);
                let n = stream.read(&mut buf[len..]).expect("Could not read from Input");
                buf.set_len(len + n);
            }
            if len == buf.len() {
                return None; // EOF
            }
        }
    }
}

#[derive(PartialEq)]
pub struct Instance {
    pub instance_variables: RefCell<Vec<Object>>,
}

pub struct Output {
    pub name: String,
    stream: RefCell<Box<dyn Write>>,
}

impl PartialEq for Output {
    fn eq(&self, other: &Self) -> bool {
        self as *const _ == other as *const _
    }
}

impl Output {
    pub fn write(&self, string: &str) {
        let end = string.len();
        let mut start = 0;
        let mut out = self.stream.borrow_mut();
        while start < end {
            match out.write(string[start..].as_bytes()) {
                Ok(n) => start += n,
                Err(e) => panic!("BUG: unhandled write error: {}", e),
            }
        }
    }
}

#[derive(PartialEq, Clone)]
pub enum Datum {
    Boolean(bool),
    Class(Rc<Class>),
    Closure(Rc<Closure>),
    Compiler(Rc<Compiler>),
    Float(f64),
    Input(Rc<Input>),
    Instance(Rc<Instance>),
    Integer(i64),
    Output(Rc<Output>),
    String(Rc<String>),
    // XXX: Null?
    System,
}

#[derive(PartialEq, Clone)]
pub struct Foolang {
    boolean_vtable: Rc<Vtable>,
    closure_vtable: Rc<Vtable>,
    compiler_vtable: Rc<Vtable>,
    float_vtable: Rc<Vtable>,
    input_vtable: Rc<Vtable>,
    integer_vtable: Rc<Vtable>,
    output_vtable: Rc<Vtable>,
    string_vtable: Rc<Vtable>,
    pub globals: RefCell<HashMap<String, Object>>,
}

impl Foolang {
    pub fn new() -> Foolang {
        let mut globals = HashMap::new();

        let boolean_vtable = Rc::new(classes::boolean2::vtable());
        globals.insert(
            "Boolean".to_string(),
            Object {
                vtable: Rc::new(Vtable::new("class Boolean")),
                datum: Datum::Class(Rc::new(Class {
                    instance_vtable: Rc::clone(&boolean_vtable),
                })),
            },
        );

        let compiler_vtable = Rc::new(classes::compiler2::instance_vtable());
        globals.insert(
            "Compiler".to_string(),
            Object {
                vtable: Rc::new(classes::compiler2::class_vtable()),
                datum: Datum::Class(Rc::new(Class {
                    instance_vtable: Rc::clone(&compiler_vtable),
                })),
            },
        );

        let float_vtable = Rc::new(classes::float2::vtable());
        globals.insert(
            "Float".to_string(),
            Object {
                vtable: Rc::new(Vtable::new("class Float")),
                datum: Datum::Class(Rc::new(Class {
                    instance_vtable: Rc::clone(&float_vtable),
                })),
            },
        );

        let input_vtable = Rc::new(classes::input2::vtable());
        globals.insert(
            "Input".to_string(),
            Object {
                vtable: Rc::new(Vtable::new("class Input")),
                datum: Datum::Class(Rc::new(Class {
                    instance_vtable: Rc::clone(&input_vtable),
                })),
            },
        );

        let integer_vtable = Rc::new(classes::integer2::vtable());
        globals.insert(
            "Integer".to_string(),
            Object {
                vtable: Rc::new(Vtable::new("class Integer")),
                datum: Datum::Class(Rc::new(Class {
                    instance_vtable: Rc::clone(&integer_vtable),
                })),
            },
        );

        let output_vtable = Rc::new(classes::output2::vtable());
        globals.insert(
            "Output".to_string(),
            Object {
                vtable: Rc::new(Vtable::new("class Output")),
                datum: Datum::Class(Rc::new(Class {
                    instance_vtable: Rc::clone(&output_vtable),
                })),
            },
        );

        let string_vtable = Rc::new(classes::string2::instance_vtable());
        globals.insert(
            "String".to_string(),
            Object {
                vtable: Rc::new(classes::string2::class_vtable()),
                datum: Datum::Class(Rc::new(Class {
                    instance_vtable: Rc::clone(&string_vtable),
                })),
            },
        );

        let foo = Foolang {
            boolean_vtable,
            closure_vtable: Rc::new(classes::closure2::vtable()),
            compiler_vtable,
            float_vtable,
            input_vtable,
            integer_vtable,
            output_vtable,
            string_vtable,
            globals: RefCell::new(globals),
        };

        foo
    }

    pub fn run(&self, program: String, system: Object) -> Result<(), Unwind> {
        let env = Env::new(self);
        let mut parser = Parser::new(&program);
        while !parser.at_eof() {
            let expr = match parser.parse() {
                Ok(expr) => expr,
                Err(unwind) => return Err(unwind.with_context(&program)),
            };
            env.eval(&expr).context(&program)?;
        }
        // FIXME: Bad error "Unknown class" with bogus span.
        let main = self.find_class("Main", 0..0)?;
        let instance = main.send("system:", &[system], self).context(&program)?;
        instance.send("run", &[], self).context(&program)?;
        Ok(())
    }

    pub fn find_class(&self, name: &str, span: Span) -> Eval {
        match self.globals.borrow().get(name) {
            None => return Unwind::error_at(span, "Unknown class"),
            Some(global) => match global.datum {
                Datum::Class(_) => Ok(global.to_owned()),
                _ => Unwind::error_at(span, "Not a class name"),
            },
        }
    }

    pub fn make_boolean(&self, x: bool) -> Object {
        Object {
            vtable: Rc::clone(&self.boolean_vtable),
            datum: Datum::Boolean(x),
        }
    }

    // FIXME: inconsistent return type vs other make_foo methods.
    // Should others be Eval as well?
    pub fn make_class(&self, classdef: &ClassDefinition) -> Eval {
        let mut vtable_name = "class ".to_string();
        vtable_name.push_str(&classdef.name);
        let mut class_vtable = Vtable::new(vtable_name.as_str());
        class_vtable.def(&classdef.constructor(), generic_ctor);
        let mut instance_vtable = Vtable::new(&classdef.name);
        let mut index = 0;
        for var in &classdef.instance_variables {
            index += 1;
            let vtable = match &var.typename {
                None => None,
                Some(typename) => {
                    let slotclass = self.find_class(typename, var.span.clone())?.class();
                    Some(slotclass.instance_vtable.clone())
                }
            };
            instance_vtable.add_slot(&var.name, index - 1, vtable);
            if &var.name[0..1] == "_" {
                continue;
            }
            instance_vtable.add_reader(&var.name, index - 1);
        }
        for method in &classdef.methods {
            instance_vtable.add_method(
                &method.selector,
                self.make_method_function(&method.parameters, &method.body),
            );
        }
        Ok(Object {
            vtable: Rc::new(class_vtable),
            datum: Datum::Class(Rc::new(Class {
                instance_vtable: Rc::new(instance_vtable),
            })),
        })
    }

    pub fn make_closure(&self, frame: Frame, params: Vec<Arg>, body: Expr) -> Object {
        Object {
            vtable: Rc::clone(&self.closure_vtable),
            datum: Datum::Closure(Rc::new(Closure {
                env: Some(frame),
                params,
                body,
            })),
        }
    }

    pub fn make_compiler(&self) -> Object {
        Object {
            vtable: Rc::clone(&self.compiler_vtable),
            datum: Datum::Compiler(Rc::new(Compiler {
                foolang: self.clone(),
                source: RefCell::new(String::new()),
                expr: RefCell::new(Expr::Const(0..0, Literal::Boolean(false))),
            })),
        }
    }

    pub fn make_float(&self, x: f64) -> Object {
        Object {
            vtable: Rc::clone(&self.float_vtable),
            datum: Datum::Float(x),
        }
    }

    pub fn make_input(&self, name: &str, input: Box<dyn Read>) -> Object {
        Object {
            vtable: Rc::clone(&self.input_vtable),
            datum: Datum::Input(Rc::new(Input {
                name: name.to_string(),
                stream: RefCell::new(input),
                buffer: RefCell::new(Vec::new()),
            })),
        }
    }

    pub fn make_integer(&self, x: i64) -> Object {
        Object {
            vtable: Rc::clone(&self.integer_vtable),
            datum: Datum::Integer(x),
        }
    }

    pub fn make_method_function(&self, params: &[Var], body: &Expr) -> Closure {
        let mut args = vec![];
        for param in params {
            let vtable = match &param.typename {
                None => None,
                // FIXME: unwrap
                Some(tn) => Some(
                    self.find_class(tn, param.span.clone())
                        .unwrap()
                        .class()
                        .instance_vtable
                        .clone(),
                ),
            };
            args.push(Arg::new(param.span.clone(), param.name.clone(), vtable));
        }
        Closure {
            env: None,
            params: args,
            body: body.to_owned(),
        }
    }

    pub fn make_output(&self, name: &str, output: Box<dyn Write>) -> Object {
        Object {
            vtable: Rc::clone(&self.output_vtable),
            datum: Datum::Output(Rc::new(Output {
                name: name.to_string(),
                stream: RefCell::new(output),
            })),
        }
    }

    pub fn make_string(&self, string: &str) -> Object {
        self.into_string(string.to_string())
    }

    pub fn into_string(&self, string: String) -> Object {
        Object {
            vtable: Rc::clone(&self.string_vtable),
            datum: Datum::String(Rc::new(string)),
        }
    }

    pub fn make_system(&self) -> Object {
        Object {
            vtable: Rc::new(classes::system2::vtable()),
            datum: Datum::System,
        }
    }
}

impl Object {
    pub fn boolean(&self) -> bool {
        match self.datum {
            Datum::Boolean(value) => value,
            _ => panic!("BUG: {} is not a Boolean", self),
        }
    }

    pub fn class(&self) -> Rc<Class> {
        match &self.datum {
            Datum::Class(class) => Rc::clone(class),
            _ => panic!("BUG: {} is not a Class", self),
        }
    }

    pub fn closure_ref(&self) -> &Closure {
        match &self.datum {
            Datum::Closure(c) => c.borrow(),
            _ => panic!("BUG: {} is not a Closure", self),
        }
    }

    pub fn compiler(&self) -> Rc<Compiler> {
        match &self.datum {
            Datum::Compiler(compiler) => Rc::clone(compiler),
            _ => panic!("BUG: {} is not a Compiler", self),
        }
    }

    pub fn float(&self) -> f64 {
        match self.datum {
            Datum::Float(f) => f,
            _ => panic!("BUG: {} is not a Float", self),
        }
    }

    pub fn input(&self) -> Rc<Input> {
        match &self.datum {
            Datum::Input(input) => Rc::clone(input),
            _ => panic!("BUG: {} is not an Input", self),
        }
    }

    pub fn instance(&self) -> Rc<Instance> {
        match &self.datum {
            Datum::Instance(instance) => Rc::clone(instance),
            _ => panic!("BUG: {} is not an instance", self),
        }
    }

    pub fn integer(&self) -> i64 {
        match self.datum {
            Datum::Integer(i) => i,
            _ => panic!("BUG: {} is not an Integer", self),
        }
    }

    pub fn output(&self) -> Rc<Output> {
        match &self.datum {
            Datum::Output(output) => Rc::clone(output),
            _ => panic!("BUG: {} is not an Output", self),
        }
    }

    pub fn string(&self) -> Rc<String> {
        match &self.datum {
            Datum::String(s) => Rc::clone(s),
            _ => panic!("BUG: {} is not a String", self),
        }
    }

    pub fn string_as_str(&self) -> &str {
        match &self.datum {
            Datum::String(s) => s.as_str(),
            _ => panic!("BUG: {} is not a String", self),
        }
    }

    pub fn send(&self, message: &str, args: &[Object], foo: &Foolang) -> Eval {
        // println!("debug: {} {} {:?}", self, message, args);
        match self.vtable.get(message) {
            Some(Method::Primitive(method)) => method(self, args, foo),
            Some(Method::Interpreter(closure)) => eval::apply(Some(self), closure, args, foo),
            Some(Method::Reader(index)) => read_instance_variable(self, *index),
            None => {
                // println!("debug: available methods: {:?}", &self.vtable.selectors());
                unimplemented!("{:?} doesNotUnderstand {} {:?}", self, message, args);
            }
        }
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.datum {
            Datum::Boolean(true) => write!(f, "True"),
            Datum::Boolean(false) => write!(f, "False"),
            Datum::Class(_) => write!(f, "#<{}>", self.vtable.name),
            Datum::Closure(x) => write!(f, "#<closure {:?}>", x.params),
            Datum::Compiler(_) => write!(f, "#<Compiler>"),
            Datum::Float(x) => {
                if x - x.floor() == 0.0 {
                    write!(f, "{}.0", x)
                } else {
                    write!(f, "{}", x)
                }
            }
            Datum::Input(input) => write!(f, "#<Input {}>", &input.name),
            Datum::Instance(_) => write!(f, "#<instance {}>", self.vtable.name),
            Datum::Integer(x) => write!(f, "{}", x),
            Datum::Output(output) => write!(f, "#<Output {}>", &output.name),
            Datum::String(s) => write!(f, "{}", s),
            Datum::System => write!(f, "#<System>"),
        }
    }
}

impl fmt::Debug for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.datum {
            Datum::Integer(x) => write!(f, "{}", x),
            Datum::Float(x) => {
                if x - x.floor() == 0.0 {
                    write!(f, "{}.0", x)
                } else {
                    write!(f, "{}", x)
                }
            }
            Datum::Closure(x) => write!(f, "Closure({:?})", x.env),
            Datum::Class(_) => write!(f, "{}", self.vtable.name),
            Datum::Instance(_) => write!(f, "{}", self.vtable.name),
            // FIXME: Escape double-quotes
            Datum::String(s) => write!(f, "\"{}\"", s),
            _ => write!(f, "{}", self),
        }
    }
}

fn generic_ctor(receiver: &Object, args: &[Object], _foo: &Foolang) -> Eval {
    let class = receiver.class();
    Ok(Object {
        vtable: Rc::clone(&class.instance_vtable),
        datum: Datum::Instance(Rc::new(Instance {
            instance_variables: RefCell::new(args.iter().map(|x| (*x).to_owned()).collect()),
        })),
    })
}

pub fn read_instance_variable(receiver: &Object, index: usize) -> Eval {
    let instance = receiver.instance();
    let value = instance.instance_variables.borrow()[index].clone();
    Ok(value)
}

pub fn write_instance_variable(receiver: &Object, slot: &Slot, value: Object) -> Eval {
    if let Some(vtable) = &slot.vtable {
        if &value.vtable != vtable {
            return Unwind::type_error(value, vtable.name.clone());
        }
    }
    let instance = receiver.instance();
    instance.instance_variables.borrow_mut()[slot.index] = value.clone();
    Ok(value)
}
