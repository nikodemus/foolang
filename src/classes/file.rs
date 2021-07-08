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
    fn open_options(&self) -> OpenOptions {
        let mut opts = OpenOptions::new();
        opts.read(self.read)
            .truncate(self.truncate)
            .write(self.write_mode == WriteMode::Write)
            .append(self.write_mode == WriteMode::Append);
        opts
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
    Vtable::for_class("File")
}

pub fn instance_vtable() -> Vtable {
    let vt = Vtable::for_instance("File");
    vt.add_primitive_method_or_panic("pathname", file_pathname);
    vt.add_primitive_method_or_panic("create", file_create);
    vt.add_primitive_method_or_panic("createOrOpen", file_create_or_open);
    vt.add_primitive_method_or_panic("forAppend", file_for_append);
    vt.add_primitive_method_or_panic("forRead", file_for_read);
    vt.add_primitive_method_or_panic("forWrite", file_for_write);
    vt.add_primitive_method_or_panic("isAppend", file_is_append);
    vt.add_primitive_method_or_panic("isRead", file_is_read);
    vt.add_primitive_method_or_panic("isTruncate", file_is_truncate);
    vt.add_primitive_method_or_panic("isWrite", file_is_write);
    vt.add_primitive_method_or_panic("open", file_open);
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

fn file_pathname(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.into_string(format!("{}", receiver.as_file("File#pathname")?.path.display())))
}

fn file_create(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    let file = receiver.as_file("File#create")?;
    let open_file = match file.open_options().create_new(true).open(&file.path) {
        Ok(f) => f,
        Err(e) => {
            return Unwind::error(&format!(
                "Could not create file: {:?} ({:?}), did you want createOrOpen?",
                &file.path,
                e.kind()
            ))
        }
    };
    Ok(crate::classes::filestream::make_filestream(&file.path, open_file, env))
}

fn file_create_or_open(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    let file = receiver.as_file("File#createOrOpen")?;
    let open_file = match file.open_options().create(true).open(&file.path) {
        Ok(f) => f,
        Err(e) => {
            return Unwind::error(&format!(
                "Could not createOrOpen file: {:?} ({:?})",
                &file.path,
                e.kind()
            ))
        }
    };
    Ok(crate::classes::filestream::make_filestream(&file.path, open_file, env))
}

fn file_for_append(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    let file = receiver.as_file("File#forAppend")?;
    Ok(File {
        path: file.path.clone(),
        read: file.read,
        truncate: file.truncate,
        write_mode: WriteMode::Append,
    }
    .object(env))
}

fn file_for_read(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    let file = receiver.as_file("File#forRead")?;
    Ok(File {
        path: file.path.clone(),
        read: true,
        truncate: file.truncate,
        write_mode: file.write_mode,
    }
    .object(env))
}

fn file_for_write(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    let file = receiver.as_file("File#forWrite")?;
    Ok(File {
        path: file.path.clone(),
        read: file.read,
        truncate: file.truncate,
        write_mode: WriteMode::Write,
    }
    .object(env))
}

fn file_is_append(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_boolean(receiver.as_file("File#isAppend")?.write_mode == WriteMode::Append))
}

fn file_is_read(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_boolean(receiver.as_file("File#isRead")?.read))
}

fn file_is_truncate(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_boolean(receiver.as_file("File#isTruncate")?.truncate))
}

fn file_is_write(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_boolean(receiver.as_file("File#isWrite")?.write_mode == WriteMode::Write))
}

fn file_open(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    let file = receiver.as_file("File#open")?;
    let open_file = match file.open_options().open(&file.path) {
        Ok(f) => f,
        Err(e) => {
            return Unwind::error(&format!(
                "Could not open file: {:?} ({:?}), did you want #createOrOpen?",
                &file.path,
                e.kind()
            ))
        }
    };
    Ok(crate::classes::filestream::make_filestream(&file.path, open_file, env))
}

fn file_truncate_existing(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    let file = receiver.as_file("File#truncateExisting")?;
    Ok(File {
        path: file.path.clone(),
        read: file.read,
        truncate: true,
        write_mode: file.write_mode,
    }
    .object(env))
}
