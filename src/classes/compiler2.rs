use crate::eval::Env;
use crate::objects2::{Eval, Foolang, Object, Source, Vtable};
use crate::parse::Parser;

pub fn class_vtable() -> Vtable {
    let mut vt = Vtable::new("class Compiler");
    vt.def("new", class_compiler_new);
    vt
}

pub fn instance_vtable() -> Vtable {
    let mut vt = Vtable::new("Compiler");
    vt.def("parse:", compiler_parse);
    vt.def("evaluate", compiler_evaluate);
    vt
}

fn class_compiler_new(_receiver: &Object, _args: &[Object], foo: &Foolang) -> Eval {
    Ok(foo.make_compiler())
}

fn compiler_parse(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    // FIXME: This will panic if it doesn't get a string.
    let compiler = receiver.compiler();
    let source = args[0].string_as_str();
    compiler.source.replace(source.to_string());
    let mut parser = Parser::new(&source);
    compiler.expr.replace(parser.parse()?);
    Ok(receiver.clone())
}

fn compiler_evaluate(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    let compiler = receiver.compiler();
    let expr = compiler.expr.borrow();
    // This is the part that constrains the effects inside the compiler.
    let env = Env::new(&compiler.foolang);
    let source = compiler.source.borrow();
    env.eval(&expr).context(&source)
}
