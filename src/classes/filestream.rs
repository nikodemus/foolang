use std::fmt;
use std::path::{Path, PathBuf};

use std::rc::Rc;

use crate::eval::Env;
use crate::objects::{Datum, Eval, Object, Vtable};
use crate::unwind::Unwind;
use std::io::Read;

use std::cell::RefCell;
use std::hash::{Hash, Hasher};

use std::fs::File;

pub struct FileStream {
    path: PathBuf,
    file: RefCell<Option<File>>,
}

impl PartialEq for FileStream {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Eq for FileStream {}

impl Hash for FileStream {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(self, state);
    }
}

pub trait AsFileStream {
    fn as_filestream<'a>(&'a self, ctx: &str) -> Result<&'a FileStream, Unwind>;
}

impl AsFileStream for Object {
    fn as_filestream<'a>(&'a self, ctx: &str) -> Result<&'a FileStream, Unwind> {
        match &self.datum {
            Datum::FileStream(ref filestream) => Ok(filestream),
            _ => Unwind::error(&format!("{:?} is not a FileStream in {}", self, ctx)),
        }
    }
}

impl fmt::Debug for FileStream {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#<FileStream {:?}>", self.path.to_string_lossy())
    }
}

pub fn class_vtable() -> Vtable {
    Vtable::new("FileStreamClass")
}

pub fn instance_vtable() -> Vtable {
    let vt = Vtable::new("FileStream");
    vt.add_primitive_method_or_panic("close", filestream_close);
    vt.add_primitive_method_or_panic("isClosed", filestream_is_closed);
    vt.add_primitive_method_or_panic("readString", filestream_read_string);
    vt
}

pub fn make_filestream(path: &Path, file: File, env: &Env) -> Object {
    Object {
        vtable: env.foo.filestream_vtable.clone(),
        datum: Datum::FileStream(Rc::new(FileStream {
            path: PathBuf::from(path),
            file: RefCell::new(Some(file)),
        })),
    }
}

fn filestream_close(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_boolean(
        receiver.as_filestream("FileStream#close")?.file.borrow_mut().take().is_some(),
    ))
}

fn filestream_is_closed(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_boolean(
        receiver.as_filestream("FileStream#close")?.file.borrow().is_none(),
    ))
}

fn filestream_read_string(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    let fileref = receiver.as_filestream("FileStream#readString")?.file.borrow_mut();
    let mut file = match &*fileref {
        Some(f) => f,
        None => {
            return Unwind::error(&format!("Cannot read from a closed FileStream: {:?}", receiver))
        }
    };
    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(e) => {
            return Unwind::error(&format!(
                "Could not readString from {:?} ({:?})",
                receiver,
                e.kind()
            ))
        }
        _ => {}
    }
    Ok(env.foo.into_string(s))
}
