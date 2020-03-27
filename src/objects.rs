use std::borrow::Borrow;
use std::cell::{Ref, RefCell, RefMut};
use std::cmp::Eq;
use std::collections::{HashMap, HashSet};
use std::convert::AsRef;
use std::fmt;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::def::*;
use crate::eval::{Binding, Env, EnvRef};
use crate::expr::*;

use crate::span::Span;
use crate::time::TimeInfo;
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

#[derive(PartialEq, Clone, Debug)]
pub struct Signature {
    parameter_types: Vec<Option<Rc<Vtable>>>,
    return_type: Option<Rc<Vtable>>,
}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(")?;
        let mut first = true;
        for t in &self.parameter_types {
            if first {
                first = false;
            } else {
                write!(f, ", ")?;
            }
            match t {
                Some(vt) => write!(f, "{}", &vt.name)?,
                None => write!(f, "Any")?,
            }
        }
        write!(f, ") -> ")?;
        match &self.return_type {
            Some(vt) => write!(f, "{}", &vt.name)?,
            None => write!(f, "Any")?,
        }
        Ok(())
    }
}

#[derive(Clone)]
pub enum Method {
    Primitive(MethodFunction),
    Interpreter(Rc<Closure>),
    Reader(usize),
    // FIXME: split Interface from Class, give Interface a vec
    // of required signature.
    Required(Signature),
}

impl PartialEq for Method {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Method {
    fn reader(index: usize) -> Method {
        Method::Reader(index)
    }
    fn primitive(method: MethodFunction) -> Method {
        Method::Primitive(method)
    }
    fn closure(method: &MethodDefinition, env: &Env) -> Result<Method, Unwind> {
        Ok(Method::Interpreter(Rc::new(make_method_closure(
            env,
            &method.selector,
            &method.parameters,
            method.required_body()?,
            &method.return_type,
        )?)))
    }
    fn required(signature: Signature) -> Method {
        Method::Required(signature)
    }
    fn is_required(&self) -> bool {
        match self {
            Method::Required(_) => true,
            _ => false,
        }
    }
    fn signature(&self) -> Result<&Signature, Unwind> {
        match self {
            Method::Required(ref s) => Ok(s),
            Method::Interpreter(ref c) => Ok(&c.signature),
            // FIXME: Both should
            Method::Primitive(_) => Unwind::error("Primitive method has no signature"),
            Method::Reader(_) => Unwind::error("Reader method has no signature"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Slot {
    pub index: usize,
    pub vtable: Option<Rc<Vtable>>,
}

pub struct Vtable {
    pub name: String,
    pub methods: RefCell<HashMap<String, Method>>,
    pub slots: RefCell<HashMap<String, Slot>>,
    pub interfaces: RefCell<HashSet<Rc<Vtable>>>,
}

impl Vtable {
    pub fn new(class: &str) -> Vtable {
        Vtable {
            name: class.to_string(),
            methods: RefCell::new(HashMap::new()),
            slots: RefCell::new(HashMap::new()),
            interfaces: RefCell::new(HashSet::new()),
        }
    }

    pub fn add_interface(&self, vt: &Rc<Vtable>) {
        let mut interfaces = self.interfaces.borrow_mut();
        for inherited in vt.interfaces.borrow().iter() {
            interfaces.insert(inherited.clone());
        }
        interfaces.insert(vt.clone());
    }

    pub fn add_method(&self, selector: &str, method: Method) -> Result<(), Unwind> {
        if self.has(selector) {
            return Unwind::error(&format!(
                "Cannot override method {} in {}",
                selector, &self.name
            ));
        }
        self.methods.borrow_mut().insert(selector.to_string(), method);
        Ok(())
    }

    pub fn add_primitive_method_or_panic(&self, selector: &str, method: MethodFunction) {
        self.add_method(selector, Method::primitive(method))
            .expect(&format!("Could not add primitive method: {:?} to {:?}", selector, self));
    }

    pub fn add_slot(&self, name: &str, index: usize, vtable: Option<Rc<Vtable>>) {
        self.slots.borrow_mut().insert(
            name.to_string(),
            Slot {
                index,
                vtable,
            },
        );
    }

    pub fn slots(&self) -> Ref<HashMap<String, Slot>> {
        self.slots.borrow()
    }

    pub fn methods(&self) -> Ref<HashMap<String, Method>> {
        self.methods.borrow()
    }

    pub fn interfaces(&self) -> Ref<HashSet<Rc<Vtable>>> {
        self.interfaces.borrow()
    }

    pub fn selectors(&self) -> Vec<String> {
        let mut selectors = vec![];
        for key in self.methods.borrow().keys() {
            selectors.push(key.clone());
        }
        selectors
    }

    // FIXME: Could I return a reference instead?
    pub fn get(&self, name: &str) -> Option<Method> {
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
}

impl Arg {
    pub fn new(span: Span, name: String) -> Arg {
        Arg {
            span,
            name,
        }
    }
}

pub struct Class {
    pub instance_vtable: Rc<Vtable>,
    pub interface: bool,
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
    fn new_class(name: &str) -> Object {
        Object {
            vtable: Rc::new(Vtable::new(&format!("class {}", name))),
            datum: Datum::Class(Rc::new(Class {
                instance_vtable: Rc::new(Vtable::new(name)),
                interface: false,
            })),
        }
    }
    fn new_interface(name: &str) -> Object {
        Object {
            vtable: Rc::new(Vtable::new(&format!("interface {}", name))),
            datum: Datum::Class(Rc::new(Class {
                instance_vtable: Rc::new(Vtable::new(name)),
                interface: true,
            })),
        }
    }
    fn object(class_vtable: &Rc<Vtable>, instance_vtable: &Rc<Vtable>) -> Object {
        Object {
            vtable: Rc::clone(class_vtable),
            datum: Datum::Class(Rc::new(Class {
                instance_vtable: Rc::clone(instance_vtable),
                interface: false,
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
    pub signature: Signature,
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
        for ((arg, vt), obj) in self
            .params
            .iter()
            .zip(&self.signature.parameter_types)
            .zip(args.into_iter().map(|x| (*x).clone()))
        {
            let binding = match vt {
                None => Binding::untyped(obj),
                Some(ref vtable) => {
                    let value = obj.typecheck(vtable).source(&arg.span)?;
                    Binding::typed(vtable.to_owned(), value)
                }
            };
            symbols.insert(arg.name.clone(), binding);
        }
        let env = self.env.extend(symbols, receiver);
        let ret = env.eval(&self.body);
        // println!("apply return: {:?}", &ret);
        let result = match ret {
            Ok(value) => value,
            Err(Unwind::ReturnFrom(ref ret_env, ref value)) if ret_env == &env.env_ref => {
                value.clone()
            }
            Err(unwind) => {
                return Err(unwind);
            }
        };
        if let Some(vtable) = &self.signature.return_type {
            result.typecheck(vtable).source(&self.body.span())?;
        }
        Ok(result)
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
    Compiler(Rc<classes::compiler::Compiler>),
    Dictionary(Rc<classes::dictionary::Dictionary>),
    File(Rc<classes::file::File>),
    FilePath(Rc<classes::filepath::FilePath>),
    FileStream(Rc<classes::filestream::FileStream>),
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
            File(x) => x.hash(state),
            FilePath(x) => x.hash(state),
            FileStream(x) => x.hash(state),
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
    pub file_class_vtable: Rc<Vtable>,
    pub file_vtable: Rc<Vtable>,
    pub filepath_class_vtable: Rc<Vtable>,
    pub filepath_vtable: Rc<Vtable>,
    pub filestream_class_vtable: Rc<Vtable>,
    pub filestream_vtable: Rc<Vtable>,
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
    /// Holds the toplevel builtin environment, including prelude.
    builtin_env_ref: EnvRef,
    /// Used to ensure we load each module only once.
    pub modules: Rc<RefCell<HashMap<PathBuf, Env>>>,
    /// Map from toplevel module names to their paths
    pub roots: HashMap<String, PathBuf>,
}

impl Foolang {
    /// Used to initialize a builtin environment.
    pub fn init_builtins(self) -> Self {
        // println!("INIT_ENV");
        let env = &self.builtin_env_ref;
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
        env.define("File", Class::object(&self.file_class_vtable, &self.file_vtable));
        env.define("FilePath", Class::object(&self.filepath_class_vtable, &self.filepath_vtable));
        env.define(
            "FileStream",
            Class::object(&self.filestream_class_vtable, &self.filestream_vtable),
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
        // println!("INIT OK");
        self
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
            file_class_vtable: Rc::new(classes::file::class_vtable()),
            file_vtable: Rc::new(classes::file::instance_vtable()),
            filepath_class_vtable: Rc::new(classes::filepath::class_vtable()),
            filepath_vtable: Rc::new(classes::filepath::instance_vtable()),
            filestream_class_vtable: Rc::new(classes::filestream::class_vtable()),
            filestream_vtable: Rc::new(classes::filestream::instance_vtable()),
            float_class_vtable: Rc::new(Vtable::new("class Float")),
            float_vtable: Rc::new(classes::float::vtable()),
            input_class_vtable: Rc::new(Vtable::new("class Input")),
            input_vtable: Rc::new(classes::input::vtable()),
            integer_class_vtable: Rc::new(Vtable::new("class Integer")),
            integer_vtable: Rc::new(classes::integer::vtable()),
            output_class_vtable: Rc::new(classes::output::class_vtable()),
            output_vtable: Rc::new(classes::output::instance_vtable()),
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
            builtin_env_ref: EnvRef::new(),
            modules: Rc::new(RefCell::new(HashMap::new())),
            roots,
        }
        .init_builtins()
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
        let env = self.builtin_env().load_code(program, self.root())?;
        let main = env.find_global_or_unwind("Main")?;
        Ok(main.send("run:in:", &[command, self.make_system(None)], &env).context(&program)?)
    }

    fn load_prelude(self, path: &Path) -> Result<Foolang, Unwind> {
        let env = self.builtin_env();
        self.load_module_into(path, env)?;
        Ok(self)
    }

    pub fn toplevel_env(&self) -> Env {
        Env {
            env_ref: self.builtin_env_ref.enclose(),
            foo: Rc::new(self.clone()),
        }
    }

    fn builtin_env(&self) -> Env {
        Env {
            env_ref: self.builtin_env_ref.clone(),
            foo: Rc::new(self.clone()),
        }
    }

    pub fn load_module_into(&self, file: &Path, env: Env) -> Result<Env, Unwind> {
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
        env.load_code(&code, fs::canonicalize(file).unwrap().parent().unwrap())
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
    pub fn make_class(&self, def: &ClassDef, env: &Env) -> Eval {
        let mut class = Class::new_class(&def.name);
        for (i, var) in def.instance_variables.iter().enumerate() {
            class.add_slot(&var.name, i, env.maybe_type(&var.typename)?)?;
        }
        class.add_primitive_class_method(&def.constructor(), generic_ctor)?;
        for method in &def.class_methods {
            class.add_interpreted_class_method(env, method)?;
        }
        for method in &def.instance_methods {
            class.add_interpreted_instance_method(env, method)?;
        }
        for name in &def.interfaces {
            class.add_interface(env, name)?;
        }
        Ok(class)
    }

    pub fn make_interface(&self, def: &InterfaceDef, env: &Env) -> Eval {
        let interface = Class::new_interface(&def.name);
        for method in &def.class_methods {
            interface.add_interpreted_class_method(env, method)?;
        }
        for method in &def.instance_methods {
            interface.add_interpreted_instance_method(env, method)?;
        }
        for method in &def.required_methods {
            interface.add_required_method(env, method)?;
        }
        for name in &def.interfaces {
            interface.add_interface(env, name)?;
        }
        Ok(interface)
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
        parameter_type_names: Vec<&Option<String>>,
        return_type_name: &Option<String>,
    ) -> Eval {
        let mut parameter_types = vec![];
        for name in parameter_type_names {
            parameter_types.push(env.maybe_type(name)?);
        }
        let return_type = env.maybe_type(return_type_name)?;
        Ok(Object {
            vtable: Rc::clone(&self.closure_vtable),
            datum: Datum::Closure(Rc::new(Closure {
                name: "block".to_string(),
                env,
                params,
                body,
                signature: Signature {
                    parameter_types,
                    return_type,
                },
            })),
        })
    }

    pub fn make_compiler(&self) -> Object {
        classes::compiler::make_compiler(self)
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

    pub fn as_class_ref(&self) -> Result<&Class, Unwind> {
        match &self.datum {
            Datum::Class(class) => Ok(class.as_ref()),
            _ => Unwind::error(&format!("{} is not a Class or Interface", self)),
        }
    }

    pub fn typecheck(&self, typevt: &Rc<Vtable>) -> Eval {
        if typevt == &self.vtable {
            return Ok(self.clone());
        }
        for vt in self.vtable.interfaces().iter() {
            if typevt == vt {
                return Ok(self.clone());
            }
        }
        return Unwind::type_error(self.clone(), typevt.name.clone());
    }

    pub fn extend_class(&self, ext: &ExtensionDef, env: &Env) -> Eval {
        if !ext.class_methods.is_empty() && self.vtable.has("perform:with:") {
            return Unwind::error(&format!(
                "Cannot extend {}: class method 'perform:with:' defined",
                &self.vtable.name
            ));
        }
        for method in &ext.class_methods {
            self.add_interpreted_class_method(env, method)?;
        }
        let class = self.as_class_ref()?;
        if !ext.instance_methods.is_empty() && class.instance_vtable.has("perform:with:") {
            return Unwind::error(&format!(
                "Cannot extend {}: instance method 'perform:with:' defined",
                class.instance_vtable.name
            ));
        }
        for method in &ext.instance_methods {
            self.add_interpreted_instance_method(env, method)?;
        }
        for name in &ext.interfaces {
            self.add_interface(env, name)?;
        }
        Ok(self.clone())
    }

    pub fn slots(&self) -> Ref<HashMap<String, Slot>> {
        self.vtable.slots()
    }

    pub fn methods(&self) -> Ref<HashMap<String, Slot>> {
        self.vtable.slots()
    }

    pub fn closure_ref(&self) -> &Closure {
        match &self.datum {
            Datum::Closure(c) => c.borrow(),
            _ => panic!("BUG: {:?} is not a Closure", self),
        }
    }

    pub fn compiler(&self) -> Rc<classes::compiler::Compiler> {
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

    fn add_slot(
        &mut self,
        name: &str,
        index: usize,
        vtable: Option<Rc<Vtable>>,
    ) -> Result<(), Unwind> {
        let class = self.as_class_ref()?;
        if class.interface {
            return Unwind::error("BUG: Cannot add slot to an interface");
        }
        class.instance_vtable.add_slot(name, index, vtable);
        if !name.starts_with("_") {
            class.instance_vtable.add_method(name, Method::reader(index))?;
        }
        Ok(())
    }

    fn add_primitive_class_method(
        &self,
        selector: &str,
        method: MethodFunction,
    ) -> Result<(), Unwind> {
        self.as_class_ref()?;
        self.vtable.add_method(selector, Method::primitive(method))?;
        Ok(())
    }

    fn add_interface(&self, env: &Env, name: &str) -> Result<(), Unwind> {
        let class = self.as_class_ref()?;
        let class_name = &class.instance_vtable.name;
        // Add interface class methods
        let interface_obj = env.find_global_or_unwind(name)?;
        let interface = interface_obj.as_class_ref()?;
        for (selector, method) in interface_obj.vtable.methods().iter() {
            if !self.vtable.has(selector) {
                self.vtable.add_method(selector, method.clone())?;
            }
        }
        // Add interface to instance vtable
        let instance_vt = &class.instance_vtable;
        instance_vt.add_interface(&interface.instance_vtable);
        // Add interface instance methods
        for (selector, method) in interface.instance_vtable.methods().iter() {
            let signature = method.signature()?;
            let required = method.is_required();
            match instance_vt.get(selector) {
                Some(Method::Interpreter(ref closure)) => {
                    if &closure.signature != signature {
                        return Unwind::error(&format!(
                            "{}#{} is {}, interface {} specifies {}",
                            class_name, selector, &closure.signature, name, signature
                        ));
                    }
                }
                Some(_) => {
                    return Unwind::error(&format!(
                    "{}#{} is an interface method, non-vanilla implementations not supporte yet",
                    class_name, selector
                ))
                }
                None if required => {
                    return Unwind::error(&format!(
                        "{}#{} unimplemented, required by interface {}",
                        class_name, selector, name
                    ))
                }
                None => {
                    instance_vt.add_method(selector, method.clone())?;
                }
            }
        }
        Ok(())
    }

    fn add_interpreted_class_method(
        &self,
        env: &Env,
        method: &MethodDefinition,
    ) -> Result<(), Unwind> {
        let class = self.as_class_ref()?;
        let env = env.bind(&class.instance_vtable.name, Binding::untyped(self.clone()));
        self.vtable.add_method(&method.selector, Method::closure(method, &env)?)?;
        Ok(())
    }

    fn add_interpreted_instance_method(
        &self,
        env: &Env,
        method: &MethodDefinition,
    ) -> Result<(), Unwind> {
        let class = self.as_class_ref()?;
        let env = env.bind(&class.instance_vtable.name, Binding::untyped(self.clone()));
        class.instance_vtable.add_method(&method.selector, Method::closure(method, &env)?)?;
        Ok(())
    }

    fn add_required_method(&self, env: &Env, method: &MethodDefinition) -> Result<(), Unwind> {
        let class = self.as_class_ref()?;
        let mut parameter_types = vec![];
        for p in &method.parameters {
            parameter_types.push(env.maybe_type(&p.typename)?);
        }
        class.instance_vtable.add_method(
            &method.selector,
            Method::required(Signature {
                parameter_types,
                return_type: env.maybe_type(&method.return_type)?,
            }),
        )?;
        Ok(())
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

    pub fn as_file(&self, ctx: &str) -> Result<&classes::file::File, Unwind> {
        classes::file::as_file(self, ctx)
    }

    pub fn as_filepath(&self, ctx: &str) -> Result<&classes::filepath::FilePath, Unwind> {
        classes::filepath::as_filepath(self, ctx)
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

    pub fn as_str(&self) -> Result<&str, Unwind> {
        match &self.datum {
            Datum::String(s) => Ok(s.as_str()),
            _ => Unwind::error(&format!("{:?} is not a String", self)),
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
            Some(m) => match &m {
                Method::Primitive(method) => method(self, args, env),
                Method::Interpreter(closure) => closure.apply(Some(self), args),
                Method::Reader(index) => read_instance_variable(self, *index),
                Method::Required(_) => {
                    Unwind::error(&format!("Required method '{}' unimplemented", selector))
                }
            },
            None if selector == "toString" => generic_to_string(self, args, env),
            None => {
                // println!("known: {:?}", self.vtable.selectors());
                let not_understood = vec![env.foo.make_string(selector), env.foo.make_array(args)];
                match self.vtable.get("perform:with:") {
                    Some(m) => match &m {
                        Method::Primitive(method) => method(self, &not_understood, env),
                        Method::Interpreter(closure) => closure.apply(Some(self), &not_understood),
                        Method::Reader(index) => read_instance_variable(self, *index),
                        Method::Required(_) => {
                            Unwind::error(&format!("Required method '{}' unimplemented", selector))
                        }
                    },
                    None => Unwind::message_error(self, selector, args),
                }
            }
        }
    }
}

pub fn make_method_closure(
    env: &Env,
    name: &str,
    params: &[Var],
    body: &Expr,
    return_type: &Option<String>,
) -> Result<Closure, Unwind> {
    let mut args = vec![];
    let mut parameter_types = vec![];
    for param in params {
        args.push(Arg::new(param.span.clone(), param.name.clone()));
        match &param.typename {
            Some(name) => parameter_types.push(Some(env.find_type(name)?)),
            None => parameter_types.push(None),
        }
    }
    Ok(Closure {
        name: name.to_string(),
        env: env.clone(),
        params: args,
        body: body.to_owned(),
        signature: Signature {
            parameter_types,
            return_type: env.maybe_type(&return_type)?,
        },
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
            Datum::File(x) => std::fmt::Debug::fmt(x, f),
            Datum::FilePath(x) => write!(f, "{:?}", x),
            Datum::FileStream(x) => std::fmt::Debug::fmt(x, f),
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
            Datum::Closure(x) => write!(f, "#<Closure {:?}>", x.params),
            Datum::Class(_) => write!(f, "{}", self.vtable.name),
            Datum::Instance(_) => write!(f, "{}", self.vtable.name),
            Datum::String(s) => write!(f, "{:?}", s),
            _ => write!(f, "{}", self),
        }
    }
}

fn generic_ctor(receiver: &Object, args: &[Object], _env: &Env) -> Eval {
    let class = receiver.as_class_ref()?;
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
        value.typecheck(vtable)?;
    }
    let instance = receiver.instance();
    instance.instance_variables.borrow_mut()[slot.index] = value.clone();
    Ok(value)
}
