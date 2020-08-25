use std::cell::{RefCell, RefMut};
use std::fmt;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::eval::Env;
use crate::objects::{Datum, Eval, Object, Vtable};
use crate::unwind::Unwind;

pub struct FileStream {
    path: PathBuf,
    file: RefCell<Option<File>>,
}

impl FileStream {
    fn borrow_open(&self, ctx: &str) -> Result<RefMut<File>, Unwind> {
        let f = self.file.borrow_mut();
        if f.is_none() {
            Unwind::error(&format!("Sent {} to a closed FileStream: {:?}", ctx, self))
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
    Vtable::for_class("FileStream")
}

pub fn instance_vtable() -> Vtable {
    let vt = Vtable::for_instance("FileStream");
    vt.add_primitive_method_or_panic("close", filestream_close);
    vt.add_primitive_method_or_panic("flush", filestream_flush);
    vt.add_primitive_method_or_panic("isClosed", filestream_is_closed);
    vt.add_primitive_method_or_panic("offset", filestream_offset);
    vt.add_primitive_method_or_panic("offset:", filestream_offset_arg);
    vt.add_primitive_method_or_panic("offsetFromEnd:", filestream_offset_from_end);
    vt.add_primitive_method_or_panic("offsetFromHere:", filestream_offset_from_here);
    vt.add_primitive_method_or_panic("readString", filestream_read_string);
    vt.add_primitive_method_or_panic("resize:", filestream_resize);
    vt.add_primitive_method_or_panic(
        "tryReadOnce:bytesInto:at:",
        filestream_try_read_once_bytes_into_at,
    );
    vt.add_primitive_method_or_panic(
        "tryWriteOnce:bytesFrom:at:",
        filestream_try_write_once_bytes_from_at,
    );
    vt.add_primitive_method_or_panic("writeString:", filestream_write_string);
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

fn filestream_flush(receiver: &Object, _args: &[Object], _env: &Env) -> Eval {
    if let Err(e) = receiver.as_filestream("FileStream#close")?.borrow_open("#flush")?.flush() {
        return Unwind::error(&format!("Error flushing {:?} ({:?})", receiver, e.kind()));
    }
    Ok(receiver.clone())
}

fn filestream_is_closed(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_boolean(receiver.as_filestream("FileStream#close")?.file.borrow().is_none()))
}

fn filestream_offset(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    let mut fileref = receiver.as_filestream("FileStream#offset")?.borrow_open("#offset")?;
    let pos = match fileref.seek(SeekFrom::Current(0)) {
        Ok(pos) => pos,
        Err(e) => {
            return Unwind::error(&format!(
                "Could not determine current offset for {:?} ({:?})",
                receiver,
                e.kind()
            ))
        }
    };
    Ok(env.foo.make_integer(pos as i64))
}

fn filestream_offset_arg(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let mut fileref = receiver.as_filestream("FileStream#offset:")?.borrow_open("#offset:")?;
    let pos = match fileref.seek(SeekFrom::Start(args[0].integer() as u64)) {
        Ok(pos) => pos,
        Err(e) => {
            return Unwind::error(&format!(
                "Could not set offset for {:?} ({:?})",
                receiver,
                e.kind()
            ))
        }
    };
    Ok(env.foo.make_integer(pos as i64))
}

fn filestream_offset_from_end(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let mut fileref =
        receiver.as_filestream("FileStream#offsetFromEnd:")?.borrow_open("#offsetFromEnd:")?;
    let pos = match fileref.seek(SeekFrom::End(args[0].integer())) {
        Ok(pos) => pos,
        Err(e) => {
            return Unwind::error(&format!(
                "Could not set offset for {:?} ({:?})",
                receiver,
                e.kind()
            ))
        }
    };
    Ok(env.foo.make_integer(pos as i64))
}

fn filestream_offset_from_here(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let mut fileref =
        receiver.as_filestream("FileStream#offsetFromHere:")?.borrow_open("#offsetFromHere:")?;
    let pos = match fileref.seek(SeekFrom::Current(args[0].integer())) {
        Ok(pos) => pos,
        Err(e) => {
            return Unwind::error(&format!(
                "Could not set offset for {:?} ({:?})",
                receiver,
                e.kind()
            ))
        }
    };
    Ok(env.foo.make_integer(pos as i64))
}

fn filestream_read_string(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    let mut fileref =
        receiver.as_filestream("FileStream#readString")?.borrow_open("#readString")?;
    let mut s = String::new();
    if let Err(e) = fileref.read_to_string(&mut s) {
        return Unwind::error(&format!(
            "Could not readString from {:?} ({:?})",
            receiver,
            e.kind()
        ));
    }
    Ok(env.foo.into_string(s))
}

fn filestream_resize(receiver: &Object, args: &[Object], _env: &Env) -> Eval {
    let fileref = receiver.as_filestream("FileStream#resize:")?.borrow_open("#resize:")?;
    match fileref.set_len(args[0].integer() as u64) {
        Ok(_) => (),
        Err(e) => {
            return Unwind::error(&format!("Could not resize {:?} ({:?})", receiver, e.kind()))
        }
    };
    Ok(receiver.clone())
}

fn filestream_try_read_once_bytes_into_at(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let mut fileref = receiver
        .as_filestream("FileStream#tryReadOnce:bytesInto:at:")?
        .borrow_open("#tryReadOnce:bytesInto:at:")?;

    let want_arg = args[0].integer();
    let want = if want_arg >= 0 {
        want_arg as usize
    } else {
        return Unwind::error(&format!("{} is not a valid number of bytes to read", want_arg));
    };

    let at_arg = args[2].integer();
    let at = if at_arg > 0 {
        (at_arg - 1) as usize
    } else {
        return Unwind::error(&format!("{} in not a valid array index", at_arg));
    };

    let mut byte_array =
        args[1].as_byte_array("FileStream#tryReadOnce:bytesInto:at:")?.borrow_mut();

    if want + at > byte_array.len() {
        return Unwind::error(&format!(
            "ByteArray too short: {} bytes starting at {} specified, size is {}",
            want,
            at_arg,
            byte_array.len()
        ));
    }

    loop {
        match fileref.read(&mut byte_array[at..at + want]) {
            Ok(got) => return Ok(env.foo.make_integer(got as i64)),
            Err(e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => {
                return Unwind::error(&format!(
                    "Error while reading from {:?} ({:?})",
                    receiver,
                    e.kind()
                ))
            }
        };
    }
}

fn filestream_try_write_once_bytes_from_at(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let mut fileref = receiver
        .as_filestream("FileStream#tryWriteOnce:bytesFrom:at:")?
        .borrow_open("#tryWriteOnce:bytesFrom:at:")?;

    let want_arg = args[0].integer();
    let want = if want_arg >= 0 {
        want_arg as usize
    } else {
        return Unwind::error(&format!("{} is not a valid number of bytes to write", want_arg));
    };

    let at_arg = args[2].integer();
    let at = if at_arg > 0 {
        (at_arg - 1) as usize
    } else {
        return Unwind::error(&format!("{} in not a valid array index", at_arg));
    };

    let mut byte_array =
        args[1].as_byte_array("FileStream#tryWriteOnce:bytesFrom:at:")?.borrow_mut();

    if want + at > byte_array.len() {
        return Unwind::error(&format!(
            "ByteArray too short: {} bytes starting at {} specified, size is {}",
            want,
            at_arg,
            byte_array.len()
        ));
    }

    loop {
        match fileref.write(&mut byte_array[at..at + want]) {
            Ok(did) => return Ok(env.foo.make_integer(did as i64)),
            Err(e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => {
                return Unwind::error(&format!(
                    "Error while writing to {:?} ({:?})",
                    receiver,
                    e.kind()
                ))
            }
        }
    }
}

fn filestream_write_string(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let s = args[0].as_str()?;
    let mut fileref =
        receiver.as_filestream("FileStream#writeString:")?.borrow_open("#writeString:")?;
    let end = s.len();
    let mut start = 0;
    while start < end {
        match fileref.write(s[start..].as_bytes()) {
            Ok(n) => start += n,
            Err(e) => {
                return Unwind::error(&format!(
                    "Error during #writeString to {:?} ({:?})",
                    receiver,
                    e.kind()
                ))
            }
        }
    }
    Ok(env.foo.make_integer(end as i64))
}
