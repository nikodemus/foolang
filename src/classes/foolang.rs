use crate::classes::object;
use crate::evaluator::{make_method_result, Eval, GlobalEnv};
use crate::objects::{ClassId, Object};

// classes::foolang::init(&env, &class, &meta);

pub fn init(env: &mut GlobalEnv, class: &ClassId, meta: &ClassId) {
    env.classes.add_builtin(&meta, "classes", class_method_classes);
    env.classes.add_builtin(&meta, "compiler", class_method_compiler);
    env.classes.add_builtin(&class, "toString", object::method_tostring);
    env.classes.add_builtin(&class, "==", object::method_eq);
}

fn class_method_compiler(receiver: Object, args: Vec<Object>, _: &GlobalEnv) -> Eval {
    assert!(args.len() == 0);
    make_method_result(receiver, Object::make_compiler())
}

pub fn class_method_classes(receiver: Object, args: Vec<Object>, env: &GlobalEnv) -> Eval {
    assert!(args.len() == 0);
    make_method_result(
        receiver,
        Object::into_array(
            env.classes.names.iter().map(|x| Object::make_string(x.0.as_str())).collect(),
        ),
    )
}
