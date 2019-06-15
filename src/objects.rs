use crate::ast;
use crate::evaluator::GlobalEnv;
use crate::evaluator::Lexenv;
use crate::time::TimeInfo;
use std::fmt;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(PartialEq, Clone, Debug)]
pub struct ClassId(pub usize);

// NOTE: ALPHABETIC ORDER!
// Matches the order of builtin classes in evaluator.rs
// Steps by two because every second is a metaclass.
pub const CLASS_ARRAY: ClassId = ClassId(1);
pub const CLASS_BOOLEAN: ClassId = ClassId(3);
pub const CLASS_CHARACTER: ClassId = ClassId(5);
pub const CLASS_CLASS: ClassId = ClassId(7);
pub const CLASS_CLOSURE: ClassId = ClassId(9);
pub const CLASS_COMPILER: ClassId = ClassId(11);
pub const CLASS_FLOAT: ClassId = ClassId(13);
pub const CLASS_FOOLANG: ClassId = ClassId(15);
pub const CLASS_INPUT: ClassId = ClassId(17);
pub const CLASS_INTEGER: ClassId = ClassId(19);
pub const CLASS_OUTPUT: ClassId = ClassId(21);
pub const CLASS_STRING: ClassId = ClassId(23);
pub const CLASS_SYMBOL: ClassId = ClassId(25);
pub const CLASS_SYSTEM: ClassId = ClassId(27);
pub const CLASS_TIMEINFO: ClassId = ClassId(29);

#[derive(Debug, PartialEq, Clone)]
pub struct Object {
    pub class: ClassId,
    pub datum: Datum,
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.datum {
            Datum::Integer(i) => write!(f, "{}", i),
            Datum::Float(x) => write!(f, "{}", x),
            Datum::Boolean(x) => write!(f, "{}", x),
            Datum::Character(c) => write!(f, "${}", &c),
            Datum::String(s) => write!(f, r#"'{}'"#, &s.0.lock().unwrap().clone()),
            Datum::Symbol(s) => write!(f, "#{}", &s),
            Datum::Array(vec) => {
                write!(f, "#")?;
                let mut sep = "[";
                for elt in vec.lock().unwrap().iter() {
                    write!(f, "{}{}", sep, elt)?;
                    sep = " ";
                }
                write!(f, "]")
            }
            Datum::Class(class) => write!(f, "#<class {}>", class.name),
            Datum::Instance(_slot) => write!(f, "#<Obj>"),
            Datum::Closure(_closure) => write!(f, "#<Closure>"),
            Datum::Output(_output) => write!(f, "#<Output>"),
            Datum::Input(_input) => write!(f, "#<Input>"),
            Datum::Compiler(_input) => write!(f, "#<Compiler>"),
            Datum::TimeInfo(t) => write!(
                f,
                "#<TimeInfo user: {}, system: {}, real: {}>",
                t.user, t.system, t.real
            ),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ClassObject {
    pub id: ClassId,
    pub name: String,
}

#[derive(Debug)]
pub struct SlotObject {
    pub slots: Mutex<Vec<Object>>,
}

impl PartialEq for SlotObject {
    fn eq(&self, other: &Self) -> bool {
        self as *const _ == other as *const _
    }
}

#[derive(Debug)]
pub struct ClosureObject {
    pub block: ast::Block,
    pub env: Lexenv,
}

impl PartialEq for ClosureObject {
    fn eq(&self, other: &Self) -> bool {
        self as *const _ == other as *const _
    }
}

#[derive(Debug)]
pub struct ArrayObject(Mutex<Vec<Object>>);

impl PartialEq for ArrayObject {
    fn eq(&self, other: &Self) -> bool {
        if self as *const _ == other as *const _ {
            return true;
        }
        return &*self.lock().unwrap() == &*other.lock().unwrap();
    }
}

impl ArrayObject {
    pub fn with_slice<T, F>(&self, f: F) -> T
    where
        F: Fn(&[Object]) -> T,
    {
        let vec = self.lock().unwrap();
        f(&vec[..])
    }
}

impl Deref for ArrayObject {
    type Target = Mutex<Vec<Object>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct StringObject(Mutex<String>);

impl PartialEq for StringObject {
    fn eq(&self, other: &Self) -> bool {
        if self as *const _ == other as *const _ {
            return true;
        }
        self.lock().unwrap().as_str() == other.lock().unwrap().as_str()
    }
}

impl StringObject {
    pub fn to_string(&self) -> String {
        self.lock().unwrap().clone()
    }
    pub fn with_str<T>(&self, f: fn(&str) -> T) -> T {
        let string = self.lock().unwrap();
        f(string.as_str())
    }
}

impl Deref for StringObject {
    type Target = Mutex<String>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct OutputObject(pub Mutex<Box<dyn std::io::Write + Send + Sync>>);

impl PartialEq for OutputObject {
    fn eq(&self, other: &Self) -> bool {
        self as *const _ == other as *const _
    }
}

impl fmt::Debug for OutputObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#<Output>")
    }
}

impl OutputObject {
    pub fn write(&self, bytes: &[u8]) {
        self.0.lock().unwrap().write(bytes).unwrap();
    }
    pub fn flush(&self) {
        self.0.lock().unwrap().flush().unwrap();
    }
}

pub struct InputObject {
    pub stream: Mutex<Box<dyn std::io::Read + Send + Sync>>,
    pub buffer: Mutex<Vec<u8>>,
}

impl PartialEq for InputObject {
    fn eq(&self, other: &Self) -> bool {
        self as *const _ == other as *const _
    }
}

impl fmt::Debug for InputObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#<Input>")
    }
}

impl InputObject {
    pub fn read_line(&self) -> Option<String> {
        let mut buf = self.buffer.lock().unwrap();
        let mut stream = self.stream.lock().unwrap();
        loop {
            if let Some(newline) = buf.iter().position(|x| *x == 10) {
                // Check for preceding carriage return.
                let end = if newline > 0 && buf[newline - 1] == 13 {
                    newline - 1
                } else {
                    newline
                };
                let line = String::from(std::str::from_utf8(&buf[0..end]).unwrap());
                buf.drain(0..newline + 1);
                return Some(line);
            }
            buf.reserve(1024);
            let len = buf.len();
            let capacity = buf.capacity();
            unsafe {
                buf.set_len(capacity);
                let n = stream.read(&mut buf[len..]).unwrap();
                buf.set_len(len + n);
            }
            if len == buf.len() {
                return None; // EOF
            }
        }
    }
}

pub struct CompilerObject {
    pub env: Mutex<GlobalEnv>,
    // FIXME: sort out the naming
    pub ast: Mutex<Option<ast::ProgramElement>>,
}

impl fmt::Debug for CompilerObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#<Compiler>")
    }
}

impl PartialEq for CompilerObject {
    fn eq(&self, other: &Self) -> bool {
        self as *const _ == other as *const _
    }
}

// FIXME: Should have the contained objects holding the
// Arc so things which are known to receive them could
// receive owned.
#[derive(Debug, PartialEq, Clone)]
pub enum Datum {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Character(Arc<String>),
    String(Arc<StringObject>),
    Symbol(Arc<String>),
    Array(Arc<ArrayObject>),
    Class(Arc<ClassObject>),
    Instance(Arc<SlotObject>),
    Closure(Arc<ClosureObject>),
    Output(Arc<OutputObject>),
    Input(Arc<InputObject>),
    Compiler(Arc<CompilerObject>),
    TimeInfo(Arc<TimeInfo>),
}

impl Object {
    // ARRAY
    pub fn make_array(data: &[Object]) -> Object {
        Object::into_array(data.to_vec())
    }
    pub fn into_array(data: Vec<Object>) -> Object {
        Object {
            class: CLASS_ARRAY,
            datum: Datum::Array(Arc::new(ArrayObject(Mutex::new(data)))),
        }
    }
    pub fn array(&self) -> Arc<ArrayObject> {
        match &self.datum {
            Datum::Array(array) => array.to_owned(),
            _ => panic!("TypeError: {} is not an Array", self),
        }
    }
    pub fn vec(&self) -> Vec<Object> {
        self.array().lock().unwrap().to_owned()
    }
    // BOOLEAN
    pub fn make_boolean(boolean: bool) -> Object {
        Object {
            class: CLASS_BOOLEAN,
            datum: Datum::Boolean(boolean),
        }
    }
    pub fn boolean(&self) -> bool {
        match &self.datum {
            Datum::Boolean(truth) => truth.to_owned(),
            _ => panic!("TypeError: {} is not a Boolean", self),
        }
    }
    // CHARACTER
    pub fn make_character(s: &str) -> Object {
        Object::into_character(String::from(s))
    }
    pub fn into_character(s: String) -> Object {
        assert!(s.len() == 1);
        Object {
            class: CLASS_CHARACTER,
            datum: Datum::Character(Arc::new(s)),
        }
    }
    // CLASS
    pub fn make_class(meta: ClassId, id: ClassId, name: &str) -> Object {
        Object {
            class: meta,
            datum: Datum::Class(Arc::new(ClassObject {
                id,
                name: String::from(name),
            })),
        }
    }
    pub fn class(&self) -> Arc<ClassObject> {
        match &self.datum {
            Datum::Class(class) => class.to_owned(),
            _ => panic!("TypeError: not a Class: {}", self),
        }
    }
    // CLOSURE
    pub fn into_closure(block: ast::Block, env: &Lexenv) -> Object {
        Object {
            class: CLASS_CLOSURE,
            datum: Datum::Closure(Arc::new(ClosureObject {
                block,
                env: env.to_owned(),
            })),
        }
    }
    pub fn closure(&self) -> Arc<ClosureObject> {
        match &self.datum {
            Datum::Closure(closure) => closure.to_owned(),
            _ => panic!("TypeError: not a Closure: {}", self),
        }
    }
    // COMPILER
    pub fn make_compiler() -> Object {
        Object {
            class: CLASS_COMPILER,
            datum: Datum::Compiler(Arc::new(CompilerObject {
                env: Mutex::new(GlobalEnv::new()),
                ast: Mutex::new(None),
            })),
        }
    }
    pub fn compiler(&self) -> Arc<CompilerObject> {
        match &self.datum {
            Datum::Compiler(c) => c.to_owned(),
            _ => panic!("TypeError: not a Compiler: {}", self),
        }
    }
    // FLOAT
    pub fn make_float(x: f64) -> Object {
        Object {
            class: CLASS_FLOAT,
            datum: Datum::Float(x),
        }
    }
    pub fn float(&self) -> f64 {
        match &self.datum {
            Datum::Float(f) => f.to_owned(),
            _ => panic!("TypeError: {} is not a Float", self),
        }
    }
    // INPUT
    pub fn make_input(input: Box<dyn std::io::Read + Send + Sync>) -> Object {
        Object {
            class: CLASS_INPUT,
            datum: Datum::Input(Arc::new(InputObject {
                stream: Mutex::new(input),
                buffer: Mutex::new(Vec::new()),
            })),
        }
    }
    // INSTANCE
    pub fn make_instance(class: ClassId, slots: Vec<Object>) -> Object {
        Object {
            class,
            datum: Datum::Instance(Arc::new(SlotObject {
                slots: Mutex::new(slots),
            })),
        }
    }
    pub fn slot(&self, idx: usize) -> Object {
        if let Datum::Instance(obj) = &self.datum {
            obj.slots.lock().unwrap()[idx].clone()
        } else {
            panic!("Cannot access slot of a non-slot object.");
        }
    }
    pub fn set_slot(&self, idx: usize, val: Object) {
        if let Datum::Instance(obj) = &self.datum {
            obj.slots.lock().unwrap()[idx] = val;
        } else {
            panic!("Cannot access slot of a non-slot object.");
        }
    }
    // INTEGER
    pub fn make_integer(x: i64) -> Object {
        Object {
            class: CLASS_INTEGER,
            datum: Datum::Integer(x),
        }
    }
    pub fn integer(&self) -> i64 {
        match &self.datum {
            Datum::Integer(i) => i.to_owned(),
            _ => panic!("TypeError: {} is not an Integer", self),
        }
    }
    // OUTPUT
    pub fn make_output(output: Box<dyn std::io::Write + Send + Sync>) -> Object {
        Object {
            class: CLASS_OUTPUT,
            datum: Datum::Output(Arc::new(OutputObject(Mutex::new(output)))),
        }
    }
    // STRING
    pub fn make_string(s: &str) -> Object {
        Object::into_string(String::from(s))
    }
    pub fn into_string(s: String) -> Object {
        Object {
            class: CLASS_STRING,
            datum: Datum::String(Arc::new(StringObject(Mutex::new(s)))),
        }
    }
    pub fn string(&self) -> Arc<StringObject> {
        match &self.datum {
            Datum::String(s) => s.to_owned(),
            _ => panic!("TypeError: not a String: {}", self),
        }
    }
    // SYMBOL
    pub fn make_symbol(s: &str) -> Object {
        Object::into_symbol(String::from(s))
    }
    pub fn into_symbol(s: String) -> Object {
        Object {
            class: CLASS_SYMBOL,
            datum: Datum::Symbol(Arc::new(s)),
        }
    }
    // TIMEINFO
    pub fn into_timeinfo(t: TimeInfo) -> Object {
        Object {
            class: CLASS_TIMEINFO,
            datum: Datum::TimeInfo(Arc::new(t)),
        }
    }
    pub fn timeinfo(&self) -> Arc<TimeInfo> {
        match &self.datum {
            Datum::TimeInfo(t) => t.to_owned(),
            _ => panic!("TypeError: not a TimeInfo: {}", self),
        }
    }
}
