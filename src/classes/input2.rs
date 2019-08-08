use crate::objects2::{Eval, Foolang, Object, Vtable};

pub fn vtable() -> Vtable {
    let mut vt = Vtable::new("Input");
    vt.def("readline", input_readline);
    vt
}

fn input_readline(receiver: &Object, _args: &[Object], foo: &Foolang) -> Eval {
    match receiver.input().readline() {
        Some(line) => Ok(foo.into_string(line)),
        // FIXME: Nil would make more sense, or a specific EOF object
        None => Ok(foo.make_boolean(false)),
    }
}
