use std::borrow::Borrow;
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::convert::AsRef;
use std::fmt;
use std::fs;
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::eval::{Binding, Env};
use crate::parse::{ClassDefinition, ClassExtension, Expr, Literal, Parser, Var};
use crate::time::TimeInfo;
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

type MethodFunction = fn(&Object, &[Object], &Env) -> Eval;

pub enum Method {
    Primitive(MethodFunction),
    Interpreter(Rc<Closure>),
    Reader(usize),
}

#[derive(Debug, PartialEq)]
pub struct Slot {
    pub index: usize,
    pub vtable: Option<Rc<Vtable>>,
}

pub struct Vtable {
    pub name: String,
    pub methods: RefCell<HashMap<String, Rc<Method>>>,
    pub slots: HashMap<String, Slot>,
}

impl Vtable {
    pub fn new(class: &str) -> Vtable {
        Vtable {
            name: class.to_string(),
            methods: RefCell::new(HashMap::new()),
            slots: HashMap::new(),
        }
    }

    pub fn def(&mut self, name: &str, method: MethodFunction) {
        self.methods.borrow_mut().insert(name.to_string(), Rc::new(Method::Primitive(method)));
    }

    pub fn add_method(&self, selector: &str, method: Closure) -> Result<(), Unwind> {
        if self.has(selector) {
            Unwind::error(&format!("Cannot override method {} in {}", selector, self.name))
        } else {
            self.methods
                .borrow_mut()
                .insert(selector.to_string(), Rc::new(Method::Interpreter(Rc::new(method))));
            Ok(())
        }
    }

    pub fn add_reader(&mut self, selector: &str, index: usize) {
        self.methods.borrow_mut().insert(selector.to_string(), Rc::new(Method::Reader(index)));
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
        for key in self.methods.borrow().keys() {
            selectors.push(key.clone());
        }
        selectors
    }

    pub fn get(&self, name: &str) -> Option<Rc<Method>> {
        match self.methods.borrow().get(name) {
            Some(m) => Some(m.clone()),
            None => None,
        }
    }

    pub fn has(&self, name: &str) -> bool {
        self.methods.borrow().contains_key(name)
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

pub struct Array {
    pub data: RefCell<Vec<Object>>,
}

impl PartialEq for Array {
    fn eq(&self, other: &Self) -> bool {
        self as *const _ == other as *const _
    }
}

impl fmt::Debug for Array {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let data = self.data.borrow();
        let mut buf = String::from("[");
        if !data.is_empty() {
            buf.push_str(format!("{:?}", &data[0]).as_str());
            if data.len() > 1 {
                for elt in &data[1..] {
                    buf.push_str(format!(", {:?}", elt).as_str());
                }
            }
        }
        buf.push_str("]");
        write!(f, "{}", buf)
    }
}

impl fmt::Display for Array {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let data = self.data.borrow();
        let mut buf = String::from("[");
        if !data.is_empty() {
            buf.push_str(format!("{}", &data[0]).as_str());
            if data.len() > 1 {
                for elt in &data[1..] {
                    buf.push_str(format!(", {}", elt).as_str());
                }
            }
        }
        buf.push_str("]");
        write!(f, "{}", buf)
    }
}

#[derive(PartialEq)]
pub struct Class {
    pub instance_vtable: Rc<Vtable>,
}

impl Class {
    fn object(class_vtable: Vtable, instance_vtable: &Rc<Vtable>) -> Object {
        Object {
            vtable: Rc::new(class_vtable),
            datum: Datum::Class(Rc::new(Class {
                instance_vtable: Rc::clone(instance_vtable),
            })),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Closure {
    pub env: Env,
    pub params: Vec<Arg>,
    pub body: Expr,
    pub return_vtable: Option<Rc<Vtable>>,
}

impl Closure {
    pub fn apply(&self, receiver: Option<&Object>, args: &[Object]) -> Eval {
        let mut symbols = HashMap::new();
        for (arg, obj) in self.params.iter().zip(args.into_iter().map(|x| (*x).clone())) {
            let binding = match &arg.vtable {
                None => Binding::untyped(obj),
                Some(vtable) => {
                    if vtable != &obj.vtable {
                        return Unwind::type_error_at(arg.span.clone(), obj, vtable.name.clone());
                    }
                    Binding::typed(vtable.to_owned(), obj.to_owned())
                }
            };
            symbols.insert(arg.name.clone(), binding);
        }
        let env = self.env.extend(symbols, receiver);
        let ret = env.eval(&self.body);
        // println!("apply return: {:?}", &ret);
        let result = match ret {
            Ok(value) => value,
            Err(Unwind::ReturnFrom(ref ret_env, ref value)) if ret_env == &env => value.clone(),
            Err(unwind) => {
                return Err(unwind);
            }
        };
        if let Some(vtable) = &self.return_vtable {
            if &result.vtable != vtable {
                return Unwind::type_error(result, vtable.name.clone()).source(&self.body.span());
            }
        }
        Ok(result)
    }
}

// XXX: The interesting thing about this is that it doesn't give
// access to the global environment... but I actually like that.
#[derive(PartialEq)]
pub struct Compiler {
    pub env: Env,
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

#[derive(PartialEq)]
pub struct Interval {
    pub start: i64,
    pub end: i64,
}

pub struct Output {
    pub name: String,
    stream: RefCell<Box<dyn Write>>,
}

impl PartialEq for Output {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
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
    pub fn flush(&self) {
        let mut out = self.stream.borrow_mut();
        out.flush().expect("BUG: unhandled flush error");
    }
}

pub struct StringOutput {
    pub contents: RefCell<String>,
}

impl PartialEq for StringOutput {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
impl StringOutput {
    pub fn write(&self, text: &str) {
        self.contents.borrow_mut().push_str(text);
    }
    pub fn content(&self) -> String {
        self.contents.replace(String::new())
    }
}

pub struct Window {
    pub window: RefCell<kiss3d::window::Window>,
}

impl PartialEq for Window {
    fn eq(&self, other: &Self) -> bool {
        self as *const _ == other as *const _
    }
}

pub struct SceneNode {
    pub node: RefCell<kiss3d::scene::SceneNode>,
}

impl PartialEq for SceneNode {
    fn eq(&self, other: &Self) -> bool {
        self as *const _ == other as *const _
    }
}

#[derive(PartialEq, Clone)]
pub enum Datum {
    Array(Rc<Array>),
    Boolean(bool),
    Class(Rc<Class>),
    Clock,
    Closure(Rc<Closure>),
    Compiler(Rc<Compiler>),
    Float(f64),
    Input(Rc<Input>),
    Instance(Rc<Instance>),
    Integer(i64),
    Interval(Rc<Interval>),
    Output(Rc<Output>),
    String(Rc<String>),
    StringOutput(Rc<StringOutput>),
    // XXX: Null?
    System,
    Time(Rc<TimeInfo>),
    // Kiss3D stuff
    Window(Rc<Window>),
    SceneNode(Rc<SceneNode>),
}

#[derive(PartialEq, Debug, Clone)]
pub struct Foolang {
    array_vtable: Rc<Vtable>,
    boolean_vtable: Rc<Vtable>,
    clock_vtable: Rc<Vtable>,
    closure_vtable: Rc<Vtable>,
    compiler_vtable: Rc<Vtable>,
    float_vtable: Rc<Vtable>,
    input_vtable: Rc<Vtable>,
    integer_vtable: Rc<Vtable>,
    interval_vtable: Rc<Vtable>,
    output_vtable: Rc<Vtable>,
    string_vtable: Rc<Vtable>,
    string_output_vtable: Rc<Vtable>,
    time_vtable: Rc<Vtable>,
    // Kiss3D stuff
    window_vtable: Rc<Vtable>,
    scene_node_vtable: Rc<Vtable>,
    /// Holds the environment constructed by the prelude.
    prelude: Option<Env>,
    /// Used to ensure we load each module only once.
    modules: Rc<RefCell<HashMap<PathBuf, Env>>>,
    /// Map from toplevel module names to their paths
    roots: HashMap<String, PathBuf>,
}

impl Foolang {
    /// Used to initialize a builtin environment.
    pub fn init_env(&self, env: &mut Env) {
        env.define("Array", Class::object(classes::array::class_vtable(), &self.array_vtable));
        env.define("Boolean", Class::object(Vtable::new("class Boolean"), &self.boolean_vtable));
        env.define("Clock", Class::object(classes::clock::class_vtable(), &self.clock_vtable));
        env.define("Closure", Class::object(Vtable::new("class Closure"), &self.closure_vtable));
        env.define(
            "Compiler",
            Class::object(classes::compiler::class_vtable(), &self.compiler_vtable),
        );
        env.define("Float", Class::object(Vtable::new("class Float"), &self.float_vtable));
        env.define("Input", Class::object(Vtable::new("class Input"), &self.input_vtable));
        env.define("Integer", Class::object(Vtable::new("class Integer"), &self.integer_vtable));
        env.define("Interval", Class::object(Vtable::new("class Interval"), &self.interval_vtable));
        env.define("Output", Class::object(Vtable::new("class Output"), &self.output_vtable));
        env.define(
            "StringOutput",
            Class::object(classes::string_output::class_vtable(), &self.string_output_vtable),
        );
        env.define("String", Class::object(classes::string::class_vtable(), &self.string_vtable));
        env.define("Time", Class::object(classes::time::class_vtable(), &self.time_vtable));
        // Kiss3D stuff
        env.define("Window", Class::object(classes::window::class_vtable(), &self.window_vtable));
        env.define(
            "SceneNode",
            Class::object(classes::scene_node::class_vtable(), &self.scene_node_vtable),
        );
    }

    pub fn new(prelude: &Path, roots: HashMap<String, PathBuf>) -> Foolang {
        Foolang {
            array_vtable: Rc::new(classes::array::instance_vtable()),
            boolean_vtable: Rc::new(classes::boolean::vtable()),
            clock_vtable: Rc::new(classes::clock::instance_vtable()),
            closure_vtable: Rc::new(classes::closure::vtable()),
            compiler_vtable: Rc::new(classes::compiler::instance_vtable()),
            float_vtable: Rc::new(classes::float::vtable()),
            input_vtable: Rc::new(classes::input::vtable()),
            integer_vtable: Rc::new(classes::integer::vtable()),
            interval_vtable: Rc::new(classes::interval::vtable()),
            output_vtable: Rc::new(classes::output::vtable()),
            string_output_vtable: Rc::new(classes::string_output::instance_vtable()),
            string_vtable: Rc::new(classes::string::instance_vtable()),
            time_vtable: Rc::new(classes::time::instance_vtable()),
            // Kiss3D stuff
            window_vtable: Rc::new(classes::window::instance_vtable()),
            scene_node_vtable: Rc::new(classes::scene_node::instance_vtable()),
            // Other
            prelude: None,
            modules: Rc::new(RefCell::new(HashMap::new())),
            roots,
        }
        .load_prelude(prelude)
    }

    #[cfg(test)]
    pub fn here() -> Foolang {
        let mut roots = HashMap::new();
        roots.insert(".".to_string(), std::env::current_dir().unwrap());
        Foolang::new(Path::new("foo/prelude.foo"), roots)
    }

    pub fn root(&self) -> &Path {
        &self.roots["."]
    }

    pub fn run(self, program: &str) -> Eval {
        let system = self.make_system();
        let env = self.prelude_env()?;
        let mut parser = Parser::new(&program, env.foo.root());
        while !parser.at_eof() {
            let expr = match parser.parse() {
                Ok(expr) => expr,
                Err(unwind) => return Err(unwind.with_context(&program)),
            };
            env.eval(&expr).context(&program)?;
        }
        // FIXME: Bad error "Unknown class" with bogus span.
        let main = env.find_class("Main", 0..0)?;
        let instance = main.send("system:", &[system], &env).context(&program)?;
        Ok(instance.send("run", &[], &env).context(&program)?)
    }

    pub fn load_module<P: AsRef<Path>>(&self, path: P) -> Result<Env, Unwind> {
        let mut file = path.as_ref().to_path_buf();
        if file.is_relative() {
            let name = match path.as_ref().components().next() {
                Some(std::path::Component::Normal(p)) => AsRef::<Path>::as_ref(p).to_str().unwrap(),
                _ => panic!("Bad module path! {}", path.as_ref().display()),
            };
            file = match self.roots.get(name) {
                Some(p) => p.join(path),
                None => {
                    return Unwind::error(&format!(
                        "Unknown module: {}, --use /path/to/{} missing from command-line?",
                        name, name
                    ))
                }
            };
        }
        {
            // For some reason on 1.40 at least borrow() fails to infer type.
            let modules = self.modules.borrow_mut();
            if let Some(module) = modules.get(&file) {
                return Ok(module.clone());
            }
        }
        let env = self.load_module_into(&file, self.prelude_env()?)?;
        self.modules.borrow_mut().insert(file.clone(), env.clone());
        Ok(env)
    }

    fn load_prelude(mut self, path: &Path) -> Self {
        let prelude = self.load_module_into(path, Env::from(self.clone())).unwrap();
        self.prelude = Some(prelude);
        self
    }

    fn prelude_env(&self) -> Result<Env, Unwind> {
        match &self.prelude {
            Some(env) => Ok(env.clone()),
            // This seems a bit messy and not 100% obviously correct. The case is when loading the
            // prelude itself. Load prelude should maybe instead take a mutable ref to self?
            None => Ok(Env::from(self.clone())),
        }
    }

    fn load_module_into(&self, file: &Path, env: Env) -> Result<Env, Unwind> {
        let code = match std::fs::read_to_string(file) {
            Ok(code) => code,
            Err(_err) => {
                return Unwind::error(&format!(
                    "Could not load module from {}",
                    file.to_string_lossy()
                ))
            }
        };
        let mut parser = Parser::new(&code, fs::canonicalize(file).unwrap().parent().unwrap());
        while !parser.at_eof() {
            let expr = match parser.parse() {
                Ok(expr) => expr,
                Err(unwind) => return Err(unwind.with_context(&code)),
            };
            env.eval(&expr).context(&code)?;
        }
        Ok(env)
    }

    pub fn make_array(&self, data: &[Object]) -> Object {
        self.into_array(data.to_vec())
    }

    pub fn into_array(&self, data: Vec<Object>) -> Object {
        Object {
            vtable: Rc::clone(&self.array_vtable),
            datum: Datum::Array(Rc::new(Array {
                data: RefCell::new(data),
            })),
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
    pub fn make_class(&self, classdef: &ClassDefinition, env: &Env) -> Eval {
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
                    let slotclass = env.find_class(typename, var.span.clone())?.class();
                    Some(slotclass.instance_vtable.clone())
                }
            };
            instance_vtable.add_slot(&var.name, index - 1, vtable);
            if &var.name[0..1] == "_" {
                continue;
            }
            instance_vtable.add_reader(&var.name, index - 1);
        }
        for method in &classdef.class_methods {
            class_vtable.add_method(
                &method.selector,
                make_method_function(env, &method.parameters, &method.body, &method.return_type)?,
            )?;
        }
        for method in &classdef.instance_methods {
            instance_vtable.add_method(
                &method.selector,
                make_method_function(env, &method.parameters, &method.body, &method.return_type)?,
            )?;
        }
        Ok(Object {
            vtable: Rc::new(class_vtable),
            datum: Datum::Class(Rc::new(Class {
                instance_vtable: Rc::new(instance_vtable),
            })),
        })
    }

    pub fn make_clock(&self) -> Object {
        Object {
            vtable: Rc::clone(&self.clock_vtable),
            datum: Datum::Clock,
        }
    }

    pub fn make_closure(
        &self,
        env: Env,
        params: Vec<Arg>,
        body: Expr,
        rtype: &Option<String>,
    ) -> Eval {
        let rtype = env.find_vtable_if_name(rtype, body.span())?;
        Ok(Object {
            vtable: Rc::clone(&self.closure_vtable),
            datum: Datum::Closure(Rc::new(Closure {
                env,
                params,
                body,
                // FIXME: questionable span
                return_vtable: rtype,
            })),
        })
    }

    pub fn make_compiler(&self) -> Object {
        Object {
            vtable: Rc::clone(&self.compiler_vtable),
            datum: Datum::Compiler(Rc::new(Compiler {
                // This makes the objects resulting from Compiler eval
                // share same vtable instances as the parent, which
                // seems like the right thing -- but it would be nice to
                // be able to specify a different prelude. Meh.
                env: self.prelude_env().unwrap(),
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

    pub fn make_interval(&self, start: i64, end: i64) -> Object {
        Object {
            vtable: Rc::clone(&self.interval_vtable),
            datum: Datum::Interval(Rc::new(Interval {
                start,
                end,
            })),
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

    pub fn make_string_output(&self) -> Object {
        Object {
            vtable: Rc::clone(&self.string_output_vtable),
            datum: Datum::StringOutput(Rc::new(StringOutput {
                contents: RefCell::new(String::new()),
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
            vtable: Rc::new(classes::system::vtable()),
            datum: Datum::System,
        }
    }

    pub fn make_time(&self, timeinfo: TimeInfo) -> Object {
        Object {
            vtable: Rc::clone(&self.time_vtable),
            datum: Datum::Time(Rc::new(timeinfo)),
        }
    }

    // Kiss3D stuff

    pub fn make_window(&self, window: kiss3d::window::Window) -> Object {
        Object {
            vtable: Rc::clone(&self.window_vtable),
            datum: Datum::Window(Rc::new(Window {
                window: RefCell::new(window),
            })),
        }
    }

    pub fn make_scene_node(&self, node: kiss3d::scene::SceneNode) -> Object {
        Object {
            vtable: Rc::clone(&self.scene_node_vtable),
            datum: Datum::SceneNode(Rc::new(SceneNode {
                node: RefCell::new(node),
            })),
        }
    }
}

impl Object {
    pub fn as_mut_vec<T>(
        &self,
        fun: impl FnOnce(RefMut<Vec<Object>>) -> Result<T, Unwind>,
    ) -> Result<T, Unwind> {
        match &self.datum {
            Datum::Array(array) => fun(array.data.borrow_mut()),
            _ => panic!("BUG: {:?} is not an Array", self),
        }
    }

    pub fn as_vec<T>(
        &self,
        fun: impl FnOnce(Ref<Vec<Object>>) -> Result<T, Unwind>,
    ) -> Result<T, Unwind> {
        match &self.datum {
            Datum::Array(array) => fun(array.data.borrow()),
            _ => panic!("BUG: {:?} is not an Array", self),
        }
    }

    pub fn boolean(&self) -> bool {
        match self.datum {
            Datum::Boolean(value) => value,
            _ => panic!("BUG: {:?} is not a Boolean", self),
        }
    }

    pub fn is_boolean(&self) -> bool {
        match self.datum {
            Datum::Boolean(_) => true,
            _ => false,
        }
    }

    pub fn class(&self) -> Rc<Class> {
        match &self.datum {
            Datum::Class(class) => Rc::clone(class),
            _ => panic!("BUG: {:?} is not a Class", self),
        }
    }

    pub fn extend_class(&self, ext: &ClassExtension, env: &Env) -> Eval {
        let class_vtable = &self.vtable;
        if !ext.class_methods.is_empty() && class_vtable.has("perform:with:") {
            return Unwind::error(&format!(
                "Cannot extend {}: class method 'perform:with:' defined",
                &class_vtable.name
            ));
        } else {
            for method in &ext.class_methods {
                class_vtable.add_method(
                    &method.selector,
                    make_method_function(
                        env,
                        &method.parameters,
                        &method.body,
                        &method.return_type,
                    )?,
                )?;
            }
        }
        let instance_vtable = &(self.class().instance_vtable);
        if !ext.instance_methods.is_empty() && instance_vtable.has("perform:with:") {
            return Unwind::error(&format!(
                "Cannot extend {}: instance method 'perform:with:' defined",
                &instance_vtable.name
            ));
        } else {
            for method in &ext.instance_methods {
                instance_vtable.add_method(
                    &method.selector,
                    make_method_function(
                        env,
                        &method.parameters,
                        &method.body,
                        &method.return_type,
                    )?,
                )?;
            }
        }
        Ok(self.clone())
    }

    pub fn closure_ref(&self) -> &Closure {
        match &self.datum {
            Datum::Closure(c) => c.borrow(),
            _ => panic!("BUG: {:?} is not a Closure", self),
        }
    }

    pub fn compiler(&self) -> Rc<Compiler> {
        match &self.datum {
            Datum::Compiler(compiler) => Rc::clone(compiler),
            _ => panic!("BUG: {:?} is not a Compiler", self),
        }
    }

    pub fn float(&self) -> f64 {
        match self.datum {
            Datum::Float(f) => f,
            _ => panic!("BUG: {:?} is not a Float", self),
        }
    }

    pub fn input(&self) -> Rc<Input> {
        match &self.datum {
            Datum::Input(input) => Rc::clone(input),
            _ => panic!("BUG: {:?} is not an Input", self),
        }
    }

    pub fn instance(&self) -> Rc<Instance> {
        match &self.datum {
            Datum::Instance(instance) => Rc::clone(instance),
            _ => panic!("BUG: {:?} is not an instance", self),
        }
    }

    pub fn integer(&self) -> i64 {
        match self.datum {
            Datum::Integer(i) => i,
            _ => panic!("BUG: {:?} is not an Integer", self),
        }
    }

    pub fn interval(&self) -> Rc<Interval> {
        match &self.datum {
            Datum::Interval(interval) => Rc::clone(interval),
            _ => panic!("BUG: {:?} is not an Interval", self),
        }
    }

    pub fn output(&self) -> Rc<Output> {
        match &self.datum {
            Datum::Output(output) => Rc::clone(output),
            _ => panic!("BUG: {:?} is not an Output", self),
        }
    }

    pub fn string_output(&self) -> Rc<StringOutput> {
        match &self.datum {
            Datum::StringOutput(output) => Rc::clone(output),
            _ => panic!("BUG: {:?} is not a StringOutput", self),
        }
    }

    pub fn string(&self) -> Rc<String> {
        match &self.datum {
            Datum::String(s) => Rc::clone(s),
            _ => panic!("BUG: {:?} is not a String", self),
        }
    }

    pub fn string_as_str(&self) -> &str {
        match &self.datum {
            Datum::String(s) => s.as_str(),
            _ => panic!("BUG: {:?} is not a String", self),
        }
    }

    pub fn time(&self) -> &Rc<TimeInfo> {
        match &self.datum {
            Datum::Time(info) => info,
            _ => panic!("BUG: {:?} is not a Time", self),
        }
    }

    // Kiss3D stuff

    pub fn window(&self) -> &Rc<Window> {
        match &self.datum {
            Datum::Window(win) => win,
            _ => panic!("BUG: {:?} is not a Window", self),
        }
    }

    pub fn scene_node(&self) -> &Rc<SceneNode> {
        match &self.datum {
            Datum::SceneNode(node) => node,
            _ => panic!("BUG: {:?} is not a Window", self),
        }
    }

    // SEND

    pub fn send(&self, selector: &str, args: &[Object], env: &Env) -> Eval {
        // println!("debug: {} {} {:?}", self, selector, args);
        match self.vtable.get(selector) {
            Some(m) => match &*m {
                Method::Primitive(method) => method(self, args, env),
                Method::Interpreter(closure) => closure.apply(Some(self), args),
                Method::Reader(index) => read_instance_variable(self, *index),
            },
            None if selector == "toString" => generic_to_string(self, args, env),
            None => {
                let not_understood = vec![env.foo.make_string(selector), env.foo.make_array(args)];
                match self.vtable.get("perform:with:") {
                    Some(m) => match &*m {
                        Method::Primitive(_method) => unimplemented!(
                            "Dispatching to primitive perform:with: {:?} {} {:?}",
                            self,
                            selector,
                            args
                        ),
                        Method::Interpreter(closure) => closure.apply(Some(self), &not_understood),
                        Method::Reader(index) => read_instance_variable(self, *index),
                    },
                    None => Unwind::message_error(self, selector, args),
                }
            }
        }
    }
}

pub fn make_method_function(
    env: &Env,
    params: &[Var],
    body: &Expr,
    return_type: &Option<String>,
) -> Result<Closure, Unwind> {
    let mut args = vec![];
    for param in params {
        let vtable = env.find_vtable_if_name(&param.typename, param.span.clone())?;
        args.push(Arg::new(param.span.clone(), param.name.clone(), vtable));
    }
    Ok(Closure {
        env: env.clone(),
        params: args,
        body: body.to_owned(),
        // FIXME: questionable span
        return_vtable: env.find_vtable_if_name(&return_type, body.span())?,
    })
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.datum {
            Datum::Array(array) => write!(f, "{:?}", array),
            Datum::Boolean(true) => write!(f, "True"),
            Datum::Boolean(false) => write!(f, "False"),
            Datum::Class(_) => write!(f, "#<{}>", self.vtable.name),
            Datum::Clock => write!(f, "#<Clock>"),
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
            Datum::Interval(interval) => {
                write!(f, "#<Interval {} to {}>", interval.start, interval.end)
            }
            Datum::Output(output) => write!(f, "#<Output {}>", &output.name),
            Datum::StringOutput(_output) => write!(f, "#<StringOutput>"),
            Datum::String(s) => write!(f, "{}", s),
            Datum::System => write!(f, "#<System>"),
            Datum::Time(time) => write!(
                f,
                "#<Time real: {}, system: {}, user: {}>",
                time.real, time.system, time.user
            ),
            // Kiss3D stuff
            Datum::Window(_) => write!(f, "#<Window>"),
            Datum::SceneNode(_) => write!(f, "#<SceneNode>"),
        }
    }
}

impl fmt::Debug for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.datum {
            Datum::Array(array) => write!(f, "{}", array),
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

fn generic_ctor(receiver: &Object, args: &[Object], _env: &Env) -> Eval {
    let class = receiver.class();
    Ok(Object {
        vtable: Rc::clone(&class.instance_vtable),
        datum: Datum::Instance(Rc::new(Instance {
            instance_variables: RefCell::new(args.iter().map(|x| (*x).to_owned()).collect()),
        })),
    })
}

fn generic_to_string(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    match &receiver.datum {
        Datum::Class(class) => {
            Ok(env.foo.into_string(format!("#<class {}>", &class.instance_vtable.name)))
        }
        Datum::Instance(instance) => {
            let mut info = String::new();
            for var in instance.instance_variables.borrow().iter() {
                if info.len() > 50 {
                    info.push_str("...");
                    break;
                }
                if info.is_empty() {
                    info.push_str(" ");
                } else {
                    info.push_str(",");
                }
                info.push_str(format!("{:?}", var).as_str());
            }
            Ok(env.foo.into_string(format!("#<{}{}>", &receiver.vtable.name, info)))
        }
        _ => panic!("INTERNAL ERROR: unexpected object in generic_to_string: {:?}", receiver),
    }
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
