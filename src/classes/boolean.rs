use crate::classes;
use crate::evaluator::{closure_apply, make_result, Eval, GlobalEnv};
use crate::objects::{ClassId, Object};

pub fn init(env: &mut GlobalEnv, class: &ClassId, _meta: &ClassId) {
    env.classes.add_builtin(&class, "ifTrue:", classes::boolean::method_iftrue);
    env.classes.add_builtin(&class, "ifFalse:", classes::boolean::method_iffalse);
    env.classes.add_builtin(&class, "toString", classes::object::method_tostring);
    env.classes.add_builtin(&class, "==", classes::object::method_eq);
}

fn method_iffalse(receiver: Object, args: Vec<Object>, global: &GlobalEnv) -> Eval {
    assert!(args.len() == 1);
    if !receiver.boolean() {
        let closure = args[0].closure();
        closure_apply(receiver, &closure, &vec![], global)
    } else {
        make_result(receiver)
    }
}

fn method_iftrue(receiver: Object, args: Vec<Object>, global: &GlobalEnv) -> Eval {
    assert!(args.len() == 1);
    if receiver.boolean() {
        let closure = args[0].closure();
        closure_apply(receiver, &closure, &vec![], global)
    } else {
        make_result(receiver)
    }
}
