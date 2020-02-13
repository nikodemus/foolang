use crate::eval::Env;
use crate::objects::{Eval, Object, Source, Vtable};
use crate::parse::Parser;
use crate::unwind::{Error, Unwind};

pub fn class_vtable() -> Vtable {
    let vt = Vtable::new("class Compiler");
    vt.add_primitive_method_or_panic("new", class_compiler_new);
    vt
}

pub fn instance_vtable() -> Vtable {
    let vt = Vtable::new("Compiler");
    vt.add_primitive_method_or_panic("define:as:", compiler_define_as);
    vt.add_primitive_method_or_panic("evaluate", compiler_evaluate);
    vt.add_primitive_method_or_panic("parse:", compiler_parse);
    vt.add_primitive_method_or_panic("parse:onEof:", compiler_parse_on_eof);
    vt
}

fn class_compiler_new(_receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    Ok(env.foo.make_compiler())
}

fn compiler_evaluate(receiver: &Object, _args: &[Object], _env: &Env) -> Eval {
    let compiler = receiver.compiler();
    let expr = compiler.expr.borrow();
    let source = compiler.source.borrow();
    compiler.env.eval(&expr).context(&source)
}

fn compiler_define_as(receiver: &Object, args: &[Object], _env: &Env) -> Eval {
    let name = args[0].string_as_str();
    let value = args[1].clone();
    // FIXME: I used to have explicit workspaces, is this as good?
    receiver.compiler().env.define(name, value);
    Ok(receiver.clone())
}

fn parse_aux(receiver: &Object, source: &Object, handler: Option<&Object>, env: &Env) -> Eval {
    let source = source.string_as_str();
    let mut parser = Parser::new(source, env.foo.root());
    let compiler = receiver.compiler();
    let expr = match parser.parse() {
        Ok(expr) => expr,
        Err(Unwind::Exception(Error::EofError(ref e), ..)) if handler.is_some() => {
            return handler.unwrap().send("value:", &[env.foo.into_string(e.what())], env)
        }
        Err(unwind) => return Err(unwind).context(source),
    };
    compiler.source.replace(source.to_string());
    compiler.expr.replace(expr);
    Ok(receiver.clone())
}

fn compiler_parse(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    // FIXME: This will panic if it doesn't get a string.
    parse_aux(receiver, &args[0], None, env)
}

fn compiler_parse_on_eof(receiver: &Object, args: &[Object], env: &Env) -> Eval {
    // FIXME: This will panic if it doesn't get a string.
    parse_aux(receiver, &args[0], Some(&args[1]), env)
}
