use std::cell::RefCell;
use std::io::Write;

use crate::objects2::{Eval, Foolang, Object, Vtable};

pub fn vtable() -> Vtable {
    let mut vt = Vtable::new("Output");
    vt.def("newline", output_newline);
    vt.def("print:", output_print);
    vt.def("println:", output_println);
    vt
}

fn write(stream: &RefCell<Box<dyn Write>>, string: &str) {
    let end = string.len();
    let mut start = 0;
    let mut out = stream.borrow_mut();
    while start < end {
        match out.write(string[start..].as_bytes()) {
            Ok(n) => start += n,
            Err(e) => panic!("BUG: unhandled write error: {}", e),
        }
    }
}

fn output_newline(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    write(&receiver.output().stream, "\n");
    Ok(receiver.clone())
}

fn output_print(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    write(&receiver.output().stream, args[0].string_as_str());
    Ok(receiver.clone())
}

fn output_println(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let stream = &receiver.output().stream;
    write(stream, args[0].string_as_str());
    write(stream, "\n");
    Ok(receiver.clone())
}
