use crate::classes;
use crate::evaluator::{closure_apply, make_method_result, Eval, GlobalEnv};
use crate::objects::{ClassId, Object};

pub fn init(env: &mut GlobalEnv, class: &ClassId, _meta: &ClassId) {
    let classes = &mut env.classes;
    classes.add_builtin(&class, "==", classes::object::method_eq);
    classes.add_builtin(&class, "detect:", classes::array::method_detect);
    classes.add_builtin(&class, "do:", classes::array::method_do);
    classes.add_builtin(&class, "inject:into:", classes::array::method_inject_into);
    classes.add_builtin(&class, "push:", classes::array::method_push);
    classes.add_builtin(&class, "size", classes::array::method_size);
    classes.add_builtin(&class, "toString", classes::object::method_tostring);
}

pub fn method_detect(receiver: Object, args: Vec<Object>, global: &GlobalEnv) -> Eval {
    assert!(args.len() == 1);
    let array = receiver.array();
    let closure = args[0].closure();
    let truth = Object::make_boolean(true);
    array.with_slice(|slice| {
        for elt in slice.iter() {
            let tmp = closure_apply(receiver.clone(), &closure, &vec![elt.to_owned()], global);
            if tmp.is_return() {
                return tmp;
            }
            if tmp.value() == truth {
                return make_method_result(receiver.clone(), elt.to_owned());
            }
        }
        make_method_result(receiver.clone(), Object::make_boolean(false))
    })
}

pub fn method_do(receiver: Object, args: Vec<Object>, global: &GlobalEnv) -> Eval {
    assert!(args.len() == 1);
    let array = receiver.array();
    let closure = args[0].closure();
    array.with_slice(|slice| {
        for elt in slice.iter() {
            let res = closure_apply(receiver.clone(), &closure, &vec![elt.to_owned()], global);
            if res.is_return() {
                return res;
            }
        }
        make_method_result(receiver.clone(), receiver.clone())
    })
}

pub fn method_inject_into(receiver: Object, args: Vec<Object>, global: &GlobalEnv) -> Eval {
    assert!(args.len() == 2);
    let array = receiver.array();
    let closure = args[1].closure();
    array.with_slice(|slice| {
        let mut value = args[0].to_owned();
        for elt in slice.iter() {
            let res = closure_apply(
                receiver.clone(),
                &closure,
                &vec![value, elt.to_owned()],
                global,
            );
            if res.is_return() {
                return res;
            } else {
                value = res.value()
            }
        }
        make_method_result(receiver.clone(), value)
    })
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
