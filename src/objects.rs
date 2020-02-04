use std::borrow::Borrow;
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::convert::AsRef;
use std::fmt;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::eval::{Binding, Env};
use crate::parse::{ClassDefinition, ClassExtension, Const, Expr, Literal, Parser, Var};
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

impl PartialEq for Method {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

#[derive(Debug, PartialEq, Eq)]
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
        std::ptr::eq(self, other)
    }
}

impl Eq for Vtable {}

impl Hash for Vtable {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(self, state);
    }
}

#[derive(PartialEq, Clone, Eq, Hash)]
pub struct Object {
    pub vtable: Rc<Vtable>,
    pub datum: Datum,
}

pub struct System {
    pub output: Option<Object>,
}

impl PartialEq for System {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Eq for System {}

impl Hash for System {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(self, state);
    }
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

pub struct Class {
    pub instance_vtable: Rc<Vtable>,
}

impl PartialEq for Class {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Eq for Class {}

impl Hash for Class {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(self, state);
    }
}

impl Class {
    fn object(class_vtable: &Rc<Vtable>, instance_vtable: &Rc<Vtable>) -> Object {
        Object {
            vtable: Rc::clone(class_vtable),
            datum: Datum::Class(Rc::new(Class {
                instance_vtable: Rc::clone(instance_vtable),
            })),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Closure {
    pub name: String,
    pub env: Env,
    pub params: Vec<Arg>,
    pub body: Expr,
    pub return_vtable: Option<Rc<Vtable>>,
}

impl PartialEq for Closure {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Eq for Closure {}

impl Hash for Closure {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(self, state);
    }
}

impl Closure {
    pub fn apply(&self, receiver: Option<&Object>, args: &[Object]) -> Eval {
        let mut symbols = HashMap::new();
        if self.params.len() != args.len() {
            return Unwind::error_at(
                self.body.span(), // FIXME: call-site would be 1000 x better...
                &format!(
                    "Argument count mismatch, {} wanted {}, got {}: {:?}",
                    &self.name,
                    self.params.len(),
                    args.len(),
                    args,
                ),
            );
        }
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

pub struct Compiler {
    pub env: Env,
    pub source: RefCell<String>,
    pub expr: RefCell<Expr>,
}

impl PartialEq for Compiler {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Eq for Compiler {}

impl Hash for Compiler {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(self, state);
    }
}

pub struct Input {
    pub name: String,
    stream: RefCell<Box<dyn Read>>,
    buffer: RefCell<Vec<u8>>,
}

impl PartialEq for Input {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Eq for Input {}

impl Hash for Input {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(self, state);
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

pub struct Instance {
    pub instance_variables: RefCell<Vec<Object>>,
}

impl PartialEq for Instance {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Eq for Instance {}

impl Hash for Instance {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(self, state);
    }
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

impl Eq for Output {}

impl Hash for Output {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(self, state);
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

impl Eq for StringOutput {}

impl Hash for StringOutput {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(self, state);
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
        std::ptr::eq(self, other)
    }
}

impl Eq for Window {}

impl Hash for Window {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(self, state);
    }
}

pub struct SceneNode {
    pub node: RefCell<kiss3d::scene::SceneNode>,
}

impl PartialEq for SceneNode {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Eq for SceneNode {}

impl Hash for SceneNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(self, state);
    }
}

#[derive(PartialEq, Clone)]
pub enum Datum {
    Array(Rc<classes::array::Array>),
    Boolean(bool),
    ByteArray(Rc<classes::byte_array::ByteArray>),
    Class(Rc<Class>),
    Clock,
    Closure(Rc<Closure>),
    Compiler(Rc<Compiler>),
    Dictionary(Rc<classes::dictionary::Dictionary>),
    Float(f64),
    Input(Rc<Input>),
    Instance(Rc<Instance>),
    Integer(i64),
    Output(Rc<Output>),
    Random(Rc<classes::random::Random>),
    Record(Rc<classes::record::Record>),
    String(Rc<String>),
    StringOutput(Rc<StringOutput>),
    // XXX: Null?
    System(Rc<System>),
    Time(Rc<TimeInfo>),
    // Kiss3D stuff
    Window(Rc<Window>),
    SceneNode(Rc<SceneNode>),
}

impl Eq for Datum {}

impl Hash for Datum {
    fn hash<H: Hasher>(&self, state: &mut H) {
        use Datum::*;
        match self {
            Array(x) => x.hash(state),
            Boolean(x) => x.hash(state),
            ByteArray(x) => x.hash(state),
            Class(x) => x.hash(state),
            Clock => 42.hash(state),
            Closure(x) => x.hash(state),
            Compiler(x) => x.hash(state),
            Dictionary(x) => x.hash(state),
            Float(x) => x.to_bits().hash(state),
            Input(x) => x.hash(state),
            Instance(x) => x.hash(state),
            Integer(x) => x.hash(state),
            Output(x) => x.hash(state),
            Random(x) => x.hash(state),
            Record(x) => x.hash(state),
            String(x) => x.hash(state),
            StringOutput(x) => x.hash(state),
            // XXX: Null?
            System(x) => x.hash(state),
            Time(x) => x.hash(state),
            // Kiss3D stuff
            Window(x) => x.hash(state),
            SceneNode(x) => x.hash(state),
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct Foolang {
    pub array_class_vtable: Rc<Vtable>,
    pub array_vtable: Rc<Vtable>,
    pub boolean_class_vtable: Rc<Vtable>,
    pub boolean_vtable: Rc<Vtable>,
    pub byte_array_vtable: Rc<Vtable>,
    pub byte_array_class_vtable: Rc<Vtable>,
    pub clock_class_vtable: Rc<Vtable>,
    pub clock_vtable: Rc<Vtable>,
    pub closure_class_vtable: Rc<Vtable>,
    pub closure_vtable: Rc<Vtable>,
    pub compiler_class_vtable: Rc<Vtable>,
    pub compiler_vtable: Rc<Vtable>,
    pub dictionary_class_vtable: Rc<Vtable>,
    pub dictionary_vtable: Rc<Vtable>,
    pub float_class_vtable: Rc<Vtable>,
    pub float_vtable: Rc<Vtable>,
    pub input_class_vtable: Rc<Vtable>,
    pub input_vtable: Rc<Vtable>,
    pub integer_class_vtable: Rc<Vtable>,
    pub integer_vtable: Rc<Vtable>,
    pub output_class_vtable: Rc<Vtable>,
    pub output_vtable: Rc<Vtable>,
    pub random_class_vtable: Rc<Vtable>,
    pub random_vtable: Rc<Vtable>,
    pub record_class_vtable: Rc<Vtable>,
    pub record_vtable: Rc<Vtable>,
    pub string_class_vtable: Rc<Vtable>,
    pub string_output_class_vtable: Rc<Vtable>,
    pub string_output_vtable: Rc<Vtable>,
    pub string_vtable: Rc<Vtable>,
    pub time_class_vtable: Rc<Vtable>,
    pub time_vtable: Rc<Vtable>,
    // Kiss3D stuff
    pub window_vtable: Rc<Vtable>,
    pub window_class_vtable: Rc<Vtable>,
    pub scene_node_vtable: Rc<Vtable>,
    pub scene_node_class_vtable: Rc<Vtable>,
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
        env.define("Array", Class::object(&self.array_class_vtable, &self.array_vtable));
        env.define("Boolean", Class::object(&self.boolean_class_vtable, &self.boolean_vtable));
        env.define(
            "ByteArray",
            Class::object(&self.byte_array_class_vtable, &self.byte_array_vtable),
        );
        env.define("Clock", Class::object(&self.clock_class_vtable, &self.clock_vtable));
        env.define("Closure", Class::object(&self.closure_class_vtable, &self.closure_vtable));
        env.define("Compiler", Class::object(&self.compiler_class_vtable, &self.compiler_vtable));
        env.define(
            "Dictionary",
            Class::object(&self.dictionary_class_vtable, &self.dictionary_vtable),
        );
        env.define("Float", Class::object(&self.float_class_vtable, &self.float_vtable));
        env.define("Input", Class::object(&self.input_class_vtable, &self.input_vtable));
        env.define("Integer", Class::object(&self.integer_class_vtable, &self.integer_vtable));
        env.define("Output", Class::object(&self.output_class_vtable, &self.output_vtable));
        env.define("Random", Class::object(&self.random_class_vtable, &self.random_vtable));
        env.define("Record", Class::object(&self.record_class_vtable, &self.record_vtable));
        env.define("String", Class::object(&self.string_class_vtable, &self.string_vtable));
        env.define(
            "StringOutput",
            Class::object(&self.string_output_class_vtable, &self.string_output_vtable),
        );
        env.define("Time", Class::object(&self.time_class_vtable, &self.time_vtable));
        // Kiss3D stuff
        env.define("Window", Class::object(&self.window_class_vtable, &self.window_vtable));
        env.define(
            "SceneNode",
            Class::object(&self.scene_node_class_vtable, &self.scene_node_vtable),
        );
    }

    pub fn new(prelude: &Path, roots: HashMap<String, PathBuf>) -> Result<Foolang, Unwind> {
        Foolang {
            array_class_vtable: Rc::new(classes::array::class_vtable()),
            array_vtable: Rc::new(classes::array::instance_vtable()),
            boolean_class_vtable: Rc::new(classes::boolean::class_vtable()),
            boolean_vtable: Rc::new(classes::boolean::instance_vtable()),
            byte_array_class_vtable: Rc::new(classes::byte_array::class_vtable()),
            byte_array_vtable: Rc::new(classes::byte_array::instance_vtable()),
            clock_class_vtable: Rc::new(classes::clock::class_vtable()),
            clock_vtable: Rc::new(classes::clock::instance_vtable()),
            closure_class_vtable: Rc::new(Vtable::new("class Closure")),
            closure_vtable: Rc::new(classes::closure::vtable()),
            compiler_class_vtable: Rc::new(classes::compiler::class_vtable()),
            compiler_vtable: Rc::new(classes::compiler::instance_vtable()),
            dictionary_class_vtable: Rc::new(classes::dictionary::class_vtable()),
            dictionary_vtable: Rc::new(classes::dictionary::instance_vtable()),
            float_class_vtable: Rc::new(Vtable::new("class Float")),
            float_vtable: Rc::new(classes::float::vtable()),
            input_class_vtable: Rc::new(Vtable::new("class Input")),
            input_vtable: Rc::new(classes::input::vtable()),
            integer_class_vtable: Rc::new(Vtable::new("class Integer")),
            integer_vtable: Rc::new(classes::integer::vtable()),
            output_class_vtable: Rc::new(Vtable::new("class Output")),
            output_vtable: Rc::new(classes::output::vtable()),
            random_class_vtable: Rc::new(classes::random::class_vtable()),
            random_vtable: Rc::new(classes::random::instance_vtable()),
            record_class_vtable: Rc::new(classes::record::class_vtable()),
            record_vtable: Rc::new(classes::record::instance_vtable()),
            string_class_vtable: Rc::new(classes::string::class_vtable()),
            string_output_class_vtable: Rc::new(classes::string_output::class_vtable()),
            string_output_vtable: Rc::new(classes::string_output::instance_vtable()),
            string_vtable: Rc::new(classes::string::instance_vtable()),
            time_class_vtable: Rc::new(classes::time::class_vtable()),
            time_vtable: Rc::new(classes::time::instance_vtable()),
            // Kiss3D stuff
            window_vtable: Rc::new(classes::window::instance_vtable()),
            window_class_vtable: Rc::new(classes::window::class_vtable()),
            scene_node_vtable: Rc::new(classes::scene_node::instance_vtable()),
            scene_node_class_vtable: Rc::new(classes::scene_node::class_vtable()),
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
        Foolang::new(Path::new("foo/prelude.foo"), roots).unwrap()
    }

    pub fn root(&self) -> &Path {
        &self.roots["."]
    }

    pub fn run(self, program: &str, command: Object) -> Eval {
        let system = self.make_system(None);
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
        Ok(main.send("run:in:", &[command, system], &env).context(&program)?)
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

    fn load_prelude(mut self, path: &Path) -> Result<Foolang, Unwind> {
        let prelude = self.load_module_into(path, Env::from(self.clone()))?;
        self.prelude = Some(prelude);
        Ok(self)
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
        // println!("load: {:?}", file);
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
            // println!("expr: {:?}", &expr);
            env.eval(&expr).context(&code)?;
        }
        Ok(env)
    }

    pub fn make_array(&self, data: &[Object]) -> Object {
        self.into_array(data.to_vec())
    }

    pub fn into_array(&self, data: Vec<Object>) -> Object {
        classes::array::into_array(self, data)
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
                make_method_function(
                    env,
                    format!("{}#{}", &class_vtable.name, &method.selector),
                    &method.parameters,
                    &method.body,
                    &method.return_type,
                )?,
            )?;
        }
        for method in &classdef.instance_methods {
            instance_vtable.add_method(
                &method.selector,
                make_method_function(
                    env,
                    format!("{}#{}", &instance_vtable.name, &method.selector),
                    &method.parameters,
                    &method.body,
                    &method.return_type,
                )?,
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
                name: "block".to_string(),
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
                expr: RefCell::new(Const::expr(0..0, Literal::Boolean(false))),
            })),
        }
    }

    pub fn into_dictionary(&self, data: HashMap<Object, Object>) -> Object {
        classes::dictionary::into_dictionary(self, data)
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

    pub fn make_system(&self, output: Option<Object>) -> Object {
        Object {
            vtable: Rc::new(classes::system::vtable()),
            datum: Datum::System(Rc::new(System {
                output,
            })),
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
                        format!("{}##{}", &class_vtable.name, &method.selector),
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
                        format!("{}##{}", &instance_vtable.name, &method.selector),
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

    pub fn as_i64(&self, ctx: &str) -> Result<i64, Unwind> {
        match self.datum {
            Datum::Integer(i) => Ok(i),
            _ => Unwind::error(&format!("{:?} is not an Integer ({})", &self, ctx)),
        }
    }

    pub fn as_u8(&self, ctx: &str) -> Result<u8, Unwind> {
        match self.datum {
            Datum::Integer(i) => {
                if 0 <= i && i <= 255 {
                    Ok(i as u8)
                } else {
                    Unwind::error(&format!(
                        "{:?} is not an Integer in range 0-255 ({})",
                        &self, &ctx
                    ))
                }
            }
            _ => Unwind::error(&format!("{:?} is not an Integer ({})", &self, ctx)),
        }
    }

    pub fn as_u64(&self, ctx: &str) -> Result<u64, Unwind> {
        match self.datum {
            Datum::Integer(i) => {
                if 0 <= i {
                    Ok(i as u64)
                } else {
                    Unwind::error(&format!("{:?} is not an unsigned Integer ({})", &self, &ctx))
                }
            }
            _ => Unwind::error(&format!("{:?} is not an Integer ({})", &self, ctx)),
        }
    }

    pub fn as_array(&self, ctx: &str) -> Result<&classes::array::Array, Unwind> {
        classes::array::as_array(self, ctx)
    }

    pub fn as_byte_array(&self, ctx: &str) -> Result<&classes::byte_array::ByteArray, Unwind> {
        classes::byte_array::as_byte_array(self, ctx)
    }

    pub fn as_dictionary(&self, ctx: &str) -> Result<&classes::dictionary::Dictionary, Unwind> {
        classes::dictionary::as_dictionary(self, ctx)
    }

    pub fn as_record(&self, ctx: &str) -> Result<&classes::record::Record, Unwind> {
        classes::record::as_record(self, ctx)
    }

    pub fn as_random(&self, ctx: &str) -> Result<&classes::random::Random, Unwind> {
        classes::random::as_random(self, ctx)
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

    pub fn system(&self) -> Rc<System> {
        match &self.datum {
            Datum::System(s) => Rc::clone(s),
            _ => panic!("BUG: {:?} is not a System", self),
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
                // println!("known: {:?}", self.vtable.selectors());
                let not_understood = vec![env.foo.make_string(selector), env.foo.make_array(args)];
                match self.vtable.get("perform:with:") {
                    Some(m) => match &*m {
                        Method::Primitive(method) => method(self, &not_understood, env),
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
    name: String,
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
        name,
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
            Datum::ByteArray(byte_array) => write!(f, "{:?}", byte_array),
            Datum::Class(_) => write!(f, "#<{}>", self.vtable.name),
            Datum::Clock => write!(f, "#<Clock>"),
            Datum::Closure(x) => write!(f, "#<closure {:?}>", x.params),
            Datum::Compiler(_) => write!(f, "#<Compiler>"),
            Datum::Dictionary(_) => write!(f, "#<Dictionary>"),
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
            Datum::Random(_) => write!(f, "#<Random>"),
            Datum::Record(r) => write!(f, "{:?}", r),
            Datum::StringOutput(_output) => write!(f, "#<StringOutput>"),
            Datum::String(s) => write!(f, "{}", s),
            Datum::System(_) => write!(f, "#<System>"),
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
            Datum::Array(array) => write!(f, "{:?}", array),
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
            Datum::String(s) => write!(f, "{:?}", s),
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
        _ => Ok(env.foo.into_string(format!("{}", receiver))),
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
