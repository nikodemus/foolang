use std::cell::RefCell;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use crate::eval::Env;
use crate::objects::{Datum, Eval, Foolang, Object, Source, Vtable};
use crate::parse::Parser;
use crate::syntax::Syntax;
use crate::unwind::{Error, Unwind};

pub struct Compiler {
    env: Env,
    source: RefCell<String>,
    parsed: RefCell<Vec<Syntax>>,
}

impl PartialEq for Compiler {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Eq for Compiler {}

impl Hash for Compiler {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(self, state);
    }
}

pub fn make_compiler(foo: &Foolang) -> Object {
    Object {
        vtable: Rc::clone(&foo.compiler_vtable),
        datum: Datum::Compiler(Rc::new(Compiler {
            // This makes the objects resulting from Compiler eval share same
            // vtable instances as the parent, which seems like the right thing
            // -- but it would be nice to be able to specify a different
            // prelude. Meh.
            env: foo.toplevel_env(),
            source: RefCell::new(String::new()),
            parsed: RefCell::new(Vec::new()),
        })),
    }
}

pub fn class_vtable() -> Vtable {
    let vt = Vtable::new("Compiler");
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

fn compiler_evaluate(receiver: &Object, _args: &[Object], env: &Env) -> Eval {
    let compiler = receiver.compiler();
    let source = compiler.source.borrow();
    let mut res = env.foo.make_boolean(false);
    for s in compiler.parsed.borrow().iter() {
        res = match s {
            Syntax::Def(ref def) => compiler.env.augment(def).context(&source)?,
            Syntax::Expr(ref expr) => compiler.env.eval(expr).context(&source)?,
        }
    }
    Ok(res)
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
    let mut parsed = Vec::new();
    while !parser.at_eof() {
        match parser.parse() {
            Ok(syntax) => parsed.push(syntax),
            Err(Unwind::Panic(Error::EofError(ref e), ..)) if handler.is_some() => {
                return handler.unwrap().send("value:", &[env.foo.into_string(e.what())], env)
            }
            Err(unwind) => return Err(unwind).context(source),
        };
    }
    compiler.source.replace(source.to_string());
    compiler.parsed.replace(parsed);
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
