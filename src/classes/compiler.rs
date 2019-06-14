use crate::ast::ProgramElement;
use crate::evaluator::{make_method_result, Eval, GlobalEnv};
use crate::objects::Object;
use crate::parser::try_parse;

pub fn method_evaluate(receiver: Object, args: Vec<Object>, _global: &GlobalEnv) -> Eval {
    assert!(args.len() == 0);
    let compiler = receiver.compiler();
    let mut env = compiler.env.lock().unwrap();
    let ast = compiler.ast.lock().unwrap();
    match *ast {
        None => panic!("Cannot evaluate: no AST available."),
        Some(ref ast) => match ast {
            ProgramElement::Expr(ref expr) => {
                make_method_result(receiver, env.eval(expr.to_owned()))
            }
            ProgramElement::Definition(ref def) => {
                make_method_result(receiver, env.load_definition(def.to_owned()))
            }
        },
    }
}

pub fn method_tryparse(receiver: Object, args: Vec<Object>, _global: &GlobalEnv) -> Eval {
    assert!(args.len() == 1);
    let compiler = receiver.compiler();
    let mut ast = compiler.ast.lock().unwrap();
    let mut ok = false;
    *ast = match args[0].string().with_str(|s| try_parse(s)) {
        Ok(elt) => {
            ok = true;
            Some(elt)
        }
        Err(_) => None,
    };
    make_method_result(receiver, Object::make_boolean(ok))
}
