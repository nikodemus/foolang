use crate::classes;
use crate::evaluator::{closure_apply, make_method_result, Eval, GlobalEnv};
use crate::objects::{ClassId, Datum, Object};
pub fn init(env: &mut GlobalEnv, class: &ClassId, _meta: &ClassId) {
    env.classes
        .add_builtin(&class, "==", classes::object::method_eq);
    env.classes
        .add_builtin(&class, "do:", classes::array::method_do);
    env.classes
        .add_builtin(&class, "inject:into:", classes::array::method_inject_into);
    env.classes
        .add_builtin(&class, "push:", classes::array::method_push);
    env.classes
        .add_builtin(&class, "size", classes::array::method_size);
    env.classes
        .add_builtin(&class, "toString", classes::object::method_tostring);
}

pub fn method_do(receiver: Object, args: Vec<Object>, global: &GlobalEnv) -> Eval {
    assert!(args.len() == 1);
    match &receiver.datum {
        Datum::Array(v) => {
            let closure = args[0].closure();
            for each in v.lock().unwrap().iter() {
                let res = closure_apply(receiver.clone(), &closure, &vec![each.to_owned()], global);
                if res.is_return() {
                    return res;
                }
            }
        }
        _ => panic!("TypeError: {} is not an Array", receiver),
    }
    make_method_result(receiver.clone(), receiver)
}

pub fn method_inject_into(receiver: Object, args: Vec<Object>, global: &GlobalEnv) -> Eval {
    assert!(args.len() == 2);
    match &receiver.datum {
        Datum::Array(v) => {
            let mut value = args[0].to_owned();
            let closure = args[1].closure();
            for each in v.lock().unwrap().iter() {
                let res = closure_apply(
                    receiver.clone(),
                    &closure,
                    &vec![value, each.to_owned()],
                    global,
                );
                if res.is_return() {
                    return res;
                } else {
                    value = res.value()
                }
            }
            make_method_result(receiver, value)
        }
        _ => panic!("TypeError: {} is not an Array", receiver),
    }
}

pub fn method_push(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 1);
    let array = receiver.array();
    let mut v = array.lock().unwrap();
    v.push(args[0].to_owned());
    make_method_result(receiver.clone(), receiver)
}

pub fn method_size(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 0);
    let size = receiver.array().with_slice(|slice| slice.len());
    make_method_result(receiver, Object::make_integer(size as i64))
}
