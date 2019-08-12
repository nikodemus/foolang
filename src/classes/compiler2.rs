use crate::eval::Env;
use crate::objects2::{Eval, Foolang, Object, Source, Vtable};
use crate::parse::Parser;
use crate::unwind::{Error, Unwind};

pub fn class_vtable() -> Vtable {
    let mut vt = Vtable::new("class Compiler");
    vt.def("new", class_compiler_new);
    vt
}

pub fn instance_vtable() -> Vtable {
    let mut vt = Vtable::new("Compiler");
    vt.def("define:as:", compiler_define_as);
    vt.def("evaluate", compiler_evaluate);
    vt.def("parse:", compiler_parse);
    vt.def("parse:onEof:", compiler_parse_on_eof);
    vt
}

fn class_compiler_new(_receiver: &Object, _args: &[Object], foo: &Foolang) -> Eval {
    Ok(foo.make_compiler())
}

fn compiler_evaluate(receiver: &Object, _args: &[Object], _foo: &Foolang) -> Eval {
    let compiler = receiver.compiler();
    let expr = compiler.expr.borrow();
    // This is the part that constrains the effects inside the compiler.
    let env = Env::new(&compiler.foolang);
    let source = compiler.source.borrow();
    env.eval(&expr).context(&source)
}

fn compiler_define_as(receiver: &Object, args: &[Object], _foo: &Foolang) -> Eval {
    let compiler = receiver.compiler();
    let name = args[0].string_as_str();
    let value = args[1].clone();
    let mut workspace = compiler.foolang.workspace.borrow_mut();
    workspace.insert(name.to_string(), value);
    Ok(receiver.clone())
}

fn parse_aux(receiver: &Object, source: &Object, handler: Option<&Object>, foo: &Foolang) -> Eval {
    let source = source.string_as_str();
    let mut parser = Parser::new(source);
    let compiler = receiver.compiler();
    let expr = match parser.parse() {
        Ok(expr) => expr,
        Err(Unwind::Exception(Error::EofError(ref e), ..)) if handler.is_some() => {
            return handler.unwrap().send("value:", &[foo.into_string(e.what())], foo)
        }
        Err(unwind) => return Err(unwind).context(source),
    };
    compiler.source.replace(source.to_string());
    compiler.expr.replace(expr);
    Ok(receiver.clone())
}

fn compiler_parse(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    // FIXME: This will panic if it doesn't get a string.
    parse_aux(receiver, &args[0], None, foo)
}

fn compiler_parse_on_eof(receiver: &Object, args: &[Object], foo: &Foolang) -> Eval {
    // FIXME: This will panic if it doesn't get a string.
    parse_aux(receiver, &args[0], Some(&args[1]), foo)
}
