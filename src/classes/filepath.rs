use std::fmt;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::eval::Env;
use crate::objects::{Datum, Eval, Object, Vtable};
use crate::unwind::Unwind;

#[derive(PartialEq, Hash)]
pub struct FilePath {
    path: PathBuf,
}

impl Eq for FilePath {}

impl fmt::Debug for FilePath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.path.as_os_str().is_empty() {
            write!(f, "#<FilePath (root)>")
        } else {
            write!(f, "#<FilePath {}>", self.path.to_string_lossy())
        }
    }
}

pub fn make_current_directory_filepath(env: &Env) -> Eval {
    match std::env::current_dir() {
        Ok(p) => Ok(into_filepath(p, env)),
        Err(e) => Unwind::error(&format!("Could not determine current directory: {}", e)),
    }
}

pub fn make_root_filepath(env: &Env) -> Eval {
    // NOTE: A bit of strangeness. We use relative path as the root
    // to get around Windows vs Unix differences.
    Ok(into_filepath(PathBuf::from(""), env))
}

pub fn as_filepath<'a>(obj: &'a Object, ctx: &str) -> Result<&'a FilePath, Unwind> {
    match &obj.datum {
        Datum::FilePath(ref filepath) => Ok(filepath),
        _ => Unwind::error(&format!("{:?} is not a FilePath in {}", obj, ctx)),
    }
}

pub fn class_vtable() -> Vtable {
    Vtable::for_class("FilePath")
}

pub fn instance_vtable() -> Vtable {
    let vt = Vtable::for_instance("FilePath");
    vt.add_primitive_method_or_panic("deleteFile", filepath_delete_file);
    vt.add_primitive_method_or_panic("exists", filepath_exists);
    vt.add_primitive_method_or_panic("file", filepath_file);
    vt.add_primitive_method_or_panic("isDirectory", filepath_is_directory);
    vt.add_primitive_method_or_panic("isFile", filepath_is_file);
    vt.add_primitive_method_or_panic("/", filepath_slash);
    vt.add_primitive_method_or_panic("pathname", filepath_to_pathname);
    vt
}

fn into_filepath(path: PathBuf, env: &Env) -> Object {
    Object {
        vtable: env.foo.filepath_vtable.clone(),
        datum: Datum::FilePath(Rc::new(FilePath {
            path,
        })),
    }
}

fn filepath_delete_file(receiver: &Object, _args: &[Object], _env: &Env) -> Eval {
    match std::fs::remove_file(&receiver.as_filepath("in FilePath#deleteFile")?.path) {
        Ok(()) => Ok(receiver.clone()),
        Err(e) => Unwind::error(&format!("Could not delete {:?} ({:?})", receiver, e.kind())),
    }
}

fn filepath_to_pathname(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env
        .foo
        .into_string(format!("{}", receiver.as_filepath("in FilePath#toString")?.path.display())))
}

fn filepath_exists(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_boolean(receiver.as_filepath("in FilePath#exists")?.path.exists()))
}

fn filepath_file(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(crate::classes::file::make_file(&receiver.as_filepath("in FilePath#file")?.path, env))
}

fn filepath_is_directory(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_boolean(receiver.as_filepath("in FilePath#isDirectory")?.path.is_dir()))
}

fn filepath_is_file(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_boolean(receiver.as_filepath("in FilePath#isFile")?.path.is_file()))
}

fn filepath_slash(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let filepath = receiver.as_filepath("in FilePath#Path")?;
    let path = filepath.path.as_path();
    let arg = args[0].string_as_str();
    let more = Path::new(arg);
    if arg.find("..").is_none() && (more.is_relative() || more.starts_with(path)) {
        let mut new = PathBuf::from(path);
        #[cfg(target_family = "windows")]
        new.push(arg.replace("/", "\\"));
        #[cfg(target_family = "unix")]
        new.push(arg);
        Ok(into_filepath(new, env))
    } else {
        Unwind::error(&format!("Cannot extend {:?} with {:?}", filepath, more))
    }
}
