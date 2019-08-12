use crate::objects2::{Eval, Foolang, Object, Vtable};

pub fn vtable() -> Vtable {
    let mut vt = Vtable::new("Interval");
    vt.def("do:", interval_do);
    vt
}

fn interval_do(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
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
        block.send("value:", &[foo.make_integer(i)], foo)?;
        if i == end {
            break;
        }
        i += step;
    }
    Ok(receiver.clone())
}
