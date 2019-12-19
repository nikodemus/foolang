use crate::eval::Env;
use crate::objects::{Eval, Object, Vtable};

pub fn vtable() -> Vtable {
    let mut vt = Vtable::new("Interval");
    vt.def("do:", interval_do);
    vt
}

fn interval_do(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    let interval = receiver.interval();
    let block = args[0].clone();
    let mut i = interval.start;
    let end = interval.end;
    let step = if i < end {
        1
    } else {
        -1
    };
    loop {
        block.send("value:", &[env.foo.make_integer(i)], env)?;
        if i == end {
            break;
        }
        i += step;
    }
    Ok(receiver.clone())
}
