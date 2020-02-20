use std::fmt;
use std::path::{Path, PathBuf};

use std::rc::Rc;

use crate::eval::Env;
use crate::objects::{Datum, Eval, Object, Vtable};
use crate::unwind::Unwind;
use std::io::{Read, Seek, SeekFrom};

use std::cell::{RefCell, RefMut};
use std::hash::{Hash, Hasher};

use std::fs::File;

pub struct FileStream {
    path: PathBuf,
    file: RefCell<Option<File>>,
}

impl FileStream {
    fn borrow_open(&self, ctx: &str) -> Result<RefMut<File>, Unwind> {
        let f = self.file.borrow_mut();
        if f.is_none() {
            Unwind::error(&format!("Cannot {} a closed FileStream: {:?}",
                                   ctx, self))
        } else {
            Ok(RefMut::map(f, |opt| opt.as_mut().unwrap()))
        }
    }
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
    vt.add_primitive_method_or_panic("offset", filestream_offset);
    vt.add_primitive_method_or_panic("offset:", filestream_offset_arg);
    vt.add_primitive_method_or_panic("offsetFromEnd:", filestream_offset_from_end);
    vt.add_primitive_method_or_panic("offsetFromHere:", filestream_offset_from_here);
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

fn filestream_offset(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    let mut fileref = receiver.as_filestream("FileStream#offset")?.borrow_open("deterine offset for")?;
    let pos = match fileref.seek(SeekFrom::Current(0)) {
        Ok(pos) => pos,
        Err(e) => return Unwind::error(
            &format!("Could not determine current offset for {:?} ({:?})",
                     receiver,
                     e.kind()))
    };
    Ok(env.foo.make_integer(pos as i64))
}

fn filestream_offset_arg(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let mut fileref = receiver.as_filestream("FileStream#offset:")?.borrow_open("set offset for")?;
    let pos = match fileref.seek(SeekFrom::Start(args[0].integer() as u64)) {
        Ok(pos) => pos,
        Err(e) => return Unwind::error(
            &format!("Could not set offset for {:?} ({:?})",
                     receiver,
                     e.kind()))
    };
    Ok(env.foo.make_integer(pos as i64))
}

fn filestream_offset_from_end(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let mut fileref = receiver.as_filestream("FileStream#offsetFromEnd:")?.borrow_open("set offset for")?;
    let pos = match fileref.seek(SeekFrom::End(args[0].integer())) {
        Ok(pos) => pos,
        Err(e) => return Unwind::error(
            &format!("Could not set offset for {:?} ({:?})",
                     receiver,
                     e.kind()))
    };
    Ok(env.foo.make_integer(pos as i64))
}

fn filestream_offset_from_here(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let mut fileref = receiver.as_filestream("FileStream#offsetFromHere:")?.borrow_open("set offset for")?;
    let pos = match fileref.seek(SeekFrom::Current(args[0].integer())) {
        Ok(pos) => pos,
        Err(e) => return Unwind::error(
            &format!("Could not set offset for {:?} ({:?})",
                     receiver,
                     e.kind()))
    };
    Ok(env.foo.make_integer(pos as i64))
}

fn filestream_read_string(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    let mut fileref = receiver.as_filestream("FileStream#readString")?.borrow_open("read string from")?;
    let mut s = String::new();
    if let Err(e) = fileref.read_to_string(&mut s) {
        return Unwind::error(&format!(
            "Could not readString from {:?} ({:?})",
            receiver,
            e.kind()
        ))
    }
    Ok(env.foo.into_string(s))
}
