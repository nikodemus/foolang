use crate::eval;
use crate::objects2::Vtable;

pub fn vtable() -> Vtable {
    let mut vt = Vtable::new("Closure");
    // FUNDAMENTAL
    vt.def("value", eval::apply);
    vt.def("value:", eval::apply);
    vt.def("value:value:", eval::apply);
    vt.def("value:value:value:", eval::apply);
    vt
}

// FUNDAMENTAL METHODS
