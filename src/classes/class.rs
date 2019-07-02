use crate::classes;
use crate::evaluator::{make_method_result, Eval, GlobalEnv, MethodImpl};
use crate::objects::{ClassId, Datum, Object};

pub fn init(env: &mut GlobalEnv, class: &ClassId, _meta: &ClassId) {
    env.classes.add_builtin(&class, "nane", classes::class::method_name);
    env.classes.add_builtin(&class, "toString", classes::object::method_tostring);
    env.classes.add_builtin(&class, "==", classes::object::method_eq);
}

pub fn method_createinstance(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 1);
    let class = receiver.class();
    let slots = args[0].vec();
    make_method_result(receiver, Object::make_instance(class.id.to_owned(), slots))
}

pub fn method_help(receiver: Object, args: Vec<Object>, global: &GlobalEnv) -> Eval {
    assert!(args.len() == 1);
    match &args[0].datum {
        Datum::Symbol(name) => {
            if let MethodImpl::Evaluator(m) = &global.find_method(&receiver.class, &name) {
                if let Some(s) = &m.docstring {
                    return make_method_result(receiver, Object::make_string(s));
                }
            }
        }
        _ => panic!("Bad argument to help:!"),
    }
    make_method_result(receiver, Object::make_string("No help available."))
}

pub fn method_name(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 0);
    let class = receiver.class();
    make_method_result(receiver, Object::into_symbol(class.name.clone()))
}
