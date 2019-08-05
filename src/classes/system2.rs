use crate::objects2::{Eval, Foolang, Object, Vtable};

pub fn vtable() -> Vtable {
    let mut vt = Vtable::new("System");
    vt.def("output", system_output);
    vt
}

fn system_output(_receiver: &Object, _args: &[Object], foo: &Foolang) -> Eval {
    Ok(foo.make_output("stdout", Box::new(std::io::stdout())))
}
