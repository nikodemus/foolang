use std::fmt;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::eval::Env;
use crate::objects::{Datum, Eval, Object, Vtable};
use crate::unwind::Unwind;

#[derive(PartialEq, Hash, Clone, Copy)]
enum WriteMode {
    None,
    Append,
    Write,
}

#[derive(PartialEq, Hash)]
pub struct File {
    path: PathBuf,
    read: bool,
    truncate: bool,
    write_mode: WriteMode,
}

impl File {
    fn object(self, env: &Env) -> Object {
        Object {
            vtable: env.foo.file_vtable.clone(),
            datum: Datum::File(Rc::new(self)),
        }
    }
}

impl Eq for File {}

impl fmt::Debug for File {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#<File {:?} (", self.path.to_string_lossy())?;
        let mut prefix = "";
        if self.read {
            write!(f, "{}read", prefix)?;
            prefix = "|";
        }
        if WriteMode::Append == self.write_mode {
            write!(f, "{}append", prefix)?;
            prefix = "|";
        }
        if WriteMode::Write == self.write_mode {
            write!(f, "{}write", prefix)?;
            prefix = "|";
            if self.truncate {
                write!(f, "{}truncate", prefix)?;
            }
        }
        if prefix == "" {
            write!(f, "mode unset")?;
        }
        write!(f, ")>")
    }
}

pub fn as_file<'a>(obj: &'a Object, ctx: &str) -> Result<&'a File, Unwind> {
    match &obj.datum {
        Datum::File(ref file) => Ok(file),
        _ => Unwind::error(&format!("{:?} is not a File in {}", obj, ctx)),
    }
}

pub fn class_vtable() -> Vtable {
    Vtable::new("class File")
}

pub fn instance_vtable() -> Vtable {
    let vt = Vtable::new("File");
    vt.add_primitive_method_or_panic("forAppend", file_for_append);
    vt.add_primitive_method_or_panic("forRead", file_for_read);
    vt.add_primitive_method_or_panic("forWrite", file_for_write);
    vt.add_primitive_method_or_panic("isAppend", file_is_append);
    vt.add_primitive_method_or_panic("isRead", file_is_read);
    vt.add_primitive_method_or_panic("isTruncate", file_is_truncate);
    vt.add_primitive_method_or_panic("isWrite", file_is_write);
    vt.add_primitive_method_or_panic("truncateExisting", file_truncate_existing);
    vt
}

pub fn make_file(path: &Path, env: &Env) -> Object {
    File {
        path: PathBuf::from(path),
        read: false,
        truncate: false,
        write_mode: WriteMode::None,
    }
    .object(env)
}

fn file_for_append(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    let file = receiver.as_file("in File#forAppend")?;
    Ok(File {
        path: file.path.clone(),
        read: file.read,
        truncate: file.truncate,
        write_mode: WriteMode::Append,
    }
    .object(env))
}

fn file_for_read(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    let file = receiver.as_file("in File#forRead")?;
    Ok(File {
        path: file.path.clone(),
        read: true,
        truncate: file.truncate,
        write_mode: file.write_mode,
    }
    .object(env))
}

fn file_for_write(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    let file = receiver.as_file("in File#forWrite")?;
    Ok(File {
        path: file.path.clone(),
        read: file.read,
        truncate: file.truncate,
        write_mode: WriteMode::Write,
    }
    .object(env))
}

fn file_is_append(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_boolean(receiver.as_file("in File#isAppend")?.write_mode == WriteMode::Append))
}

fn file_is_read(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_boolean(receiver.as_file("in File#isRead")?.read))
}

fn file_is_truncate(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_boolean(receiver.as_file("in File#isTruncate")?.truncate))
}

fn file_is_write(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_boolean(receiver.as_file("in File#isWrite")?.write_mode == WriteMode::Write))
}

fn file_truncate_existing(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    let file = receiver.as_file("in File#truncateExisting")?;
    Ok(File {
        path: file.path.clone(),
        read: file.read,
        truncate: true,
        write_mode: file.write_mode,
    }
    .object(env))
}
