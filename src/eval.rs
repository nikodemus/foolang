use std::borrow::ToOwned;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::objects2::{
    read_instance_variable, write_instance_variable, Arg, Builtins, Closure, Eval, Object, Source,
    Vtable,
};
use crate::parse::{Assign, ClassDefinition, Expr, Global, Literal, Parser, Return, Var};
use crate::tokenstream::Span;
use crate::unwind::Unwind;

#[cfg(test)]
use crate::objects2::Slot;
#[cfg(test)]
use crate::unwind::{Error, Location, SimpleError, TypeError};

#[derive(Debug)]
pub struct MethodFrame {
    pub args: RefCell<HashMap<String, Binding>>,
    pub receiver: Object,
}

#[derive(Debug)]
pub struct BlockFrame {
    pub args: RefCell<HashMap<String, Binding>>,
    // Innermost lexically enclosing frame
    pub parent: Option<Frame>,
    // Lexically enclosing method frame
    pub home: Option<Frame>,
}

impl PartialEq for MethodFrame {
    fn eq(&self, other: &Self) -> bool {
        self as *const _ == other as *const _
    }
}

impl PartialEq for BlockFrame {
    fn eq(&self, other: &Self) -> bool {
        self as *const _ == other as *const _
    }
}

// FIXME:
//  Frame {
//    args:
//    context: BlockContext | MethodContext
//  }
#[derive(Debug, Clone, PartialEq)]
pub enum Frame {
    MethodFrame(Rc<MethodFrame>),
    BlockFrame(Rc<BlockFrame>),
}

impl Frame {
    fn new(
        args: HashMap<String, Binding>,
        parent: Option<Frame>,
        receiver: Option<Object>,
    ) -> Frame {
        match receiver {
            None => {
                let home = match &parent {
                    None => None,
                    Some(p) => p.home(),
                };
                Frame::BlockFrame(Rc::new(BlockFrame {
                    args: RefCell::new(args),
                    parent,
                    home,
                }))
            }
            Some(receiver) => {
                assert!(parent.is_none());
                Frame::MethodFrame(Rc::new(MethodFrame {
                    args: RefCell::new(args),
                    receiver,
                }))
            }
        }
    }

    fn args(&self) -> &RefCell<HashMap<String, Binding>> {
        match self {
            Frame::MethodFrame(method_frame) => &method_frame.args,
            Frame::BlockFrame(block_frame) => &block_frame.args,
        }
    }

    fn home(&self) -> Option<Frame> {
        match self {
            Frame::MethodFrame(_) => Some(self.clone()),
            Frame::BlockFrame(block_frame) => block_frame.home.clone(),
        }
    }

    fn receiver(&self) -> Option<&Object> {
        match self {
            Frame::MethodFrame(method_frame) => Some(&method_frame.receiver),
            Frame::BlockFrame(block_frame) => {
                match &block_frame.home {
                    // FIXME: None as span
                    None => None,
                    Some(frame) => frame.receiver(),
                }
            }
        }
    }

    fn parent(&self) -> Option<Frame> {
        match self {
            Frame::MethodFrame(_) => None,
            Frame::BlockFrame(block_frame) => block_frame.parent.clone(),
        }
    }

    fn set(&self, name: &str, value: Object) -> Option<Eval> {
        match self.args().borrow_mut().get_mut(name) {
            Some(binding) => Some(binding.assign(value)),
            None => match self.parent() {
                Some(parent) => parent.set(name, value),
                None => None,
            },
        }
    }

    fn get(&self, name: &str) -> Option<Object> {
        match self.args().borrow().get(name) {
            Some(binding) => return Some(binding.value.clone()),
            None => match self.parent() {
                Some(parent) => parent.get(name),
                None => None,
            },
        }
    }
}

struct Env<'a> {
    builtins: &'a Builtins,
    frame: Frame,
}

#[derive(Debug)]
pub struct Binding {
    vtable: Option<Rc<Vtable>>,
    value: Object,
}

impl Binding {
    fn untyped(init: Object) -> Binding {
        Binding {
            vtable: None,
            value: init,
        }
    }
    fn typed(vtable: Rc<Vtable>, init: Object) -> Binding {
        Binding {
            vtable: Some(vtable),
            value: init,
        }
    }
    fn assign(&mut self, value: Object) -> Eval {
        if let Some(vtable) = &self.vtable {
            if &value.vtable != vtable {
                return Unwind::type_error(value, vtable.name.clone());
            }
        }
        self.value = value.clone();
        Ok(value)
    }
}

impl<'a> Env<'a> {
    pub fn new(builtins: &Builtins) -> Env {
        Env::from_parts(builtins, HashMap::new(), None, None)
    }

    pub fn eval(&self, expr: &Expr) -> Eval {
        use Expr::*;
        match expr {
            Assign(assign) => self.eval_assign(assign),
            Bind(name, typename, value, body) => self.eval_bind(name, typename, value, body),
            Block(_, params, body) => self.eval_block(params, body),
            ClassDefinition(definition) => self.eval_class_definition(definition),
            Const(_, literal) => self.eval_literal(literal),
            Global(global) => self.eval_global(global),
            Return(ret) => self.eval_return(ret),
            Send(_, selector, receiver, args) => self.eval_send(selector, receiver, args),
            Seq(exprs) => self.eval_seq(exprs),
            Typecheck(_, expr, typename) => self.eval_typecheck(expr, typename),
            Var(var) => self.eval_var(var),
        }
    }

    fn from_parts(
        builtins: &'a Builtins,
        args: HashMap<String, Binding>,
        parent: Option<Frame>,
        receiver: Option<Object>,
    ) -> Env<'a> {
        Env {
            builtins,
            frame: Frame::new(args, parent, receiver),
        }
    }

    fn eval_bind(
        &self,
        name: &String,
        typename: &Option<String>,
        expr: &Expr,
        body: &Expr,
    ) -> Eval {
        let mut args = HashMap::new();
        let binding = match typename {
            None => Binding::untyped(self.eval(expr)?),
            Some(typename) => {
                let class = self.find_class(typename, expr.span())?.class();
                Binding::typed(class.instance_vtable.clone(), self.eval_typecheck(expr, typename)?)
            }
        };
        args.insert(name.to_owned(), binding);
        let env = Env::from_parts(self.builtins, args, Some(self.frame.clone()), None);
        env.eval(body)
    }

    fn eval_block(&self, params: &Vec<Var>, body: &Expr) -> Eval {
        let mut args = vec![];
        for p in params {
            let vt = match &p.typename {
                None => None,
                Some(name) => {
                    Some(self.find_class(name, p.span.clone())?.class().instance_vtable.clone())
                }
            };
            args.push(Arg::new(p.span.clone(), p.name.clone(), vt));
        }
        Ok(self.builtins.make_closure(self.frame.clone(), args, body.clone()))
    }

    fn eval_class_definition(&self, definition: &ClassDefinition) -> Eval {
        // FIXME: allow anonymous classes
        if self.frame.parent().is_some() {
            return Unwind::error_at(definition.span.clone(), "Class definition not at toplevel");
        }
        let name = &definition.name;
        let class = self.builtins.make_class(definition)?;
        self.builtins.globals.borrow_mut().insert(name.to_string(), class.clone());
        Ok(class)
    }

    fn eval_global(&self, global: &Global) -> Eval {
        match self.builtins.globals.borrow().get(&global.name) {
            Some(obj) => Ok(obj.clone()),
            None => Unwind::error_at(global.span.clone(), "Undefined global"),
        }
    }

    fn eval_literal(&self, literal: &Literal) -> Eval {
        match literal {
            Literal::Integer(value) => Ok(self.builtins.make_integer(*value)),
            Literal::Float(value) => Ok(self.builtins.make_float(*value)),
            Literal::String(value) => Ok(self.builtins.make_string(value)),
        }
    }

    fn eval_return(&self, ret: &Return) -> Eval {
        match self.frame.home() {
            None => Unwind::error_at(ret.span.clone(), "No method to return from"),
            Some(frame) => Unwind::return_from(frame, self.eval(&ret.value)?),
        }
    }

    fn eval_send(&self, selector: &String, receiver: &Box<Expr>, args: &Vec<Expr>) -> Eval {
        let receiver = self.eval(receiver)?;
        let mut values = Vec::new();
        for arg in args {
            values.push(self.eval(arg)?);
        }
        let args: Vec<&Object> = values.iter().collect();
        receiver.send(selector.as_str(), &args[..], &self.builtins)
    }

    fn eval_seq(&self, exprs: &Vec<Expr>) -> Eval {
        // FIXME: false or nothing
        let mut result = self.builtins.make_integer(0);
        for expr in exprs {
            result = self.eval(expr)?;
        }
        Ok(result)
    }

    fn find_class(&self, name: &str, span: Span) -> Eval {
        self.builtins.find_class(name, span)
    }

    fn eval_typecheck(&self, expr: &Expr, typename: &str) -> Eval {
        let value = self.eval(expr)?;
        // FIXME: Wrong span.
        let class = self.find_class(typename, expr.span())?.class();
        if class.instance_vtable == value.vtable {
            Ok(value)
        } else {
            Unwind::type_error_at(expr.span(), value, class.instance_vtable.name.clone())
        }
    }

    fn eval_assign(&self, assign: &Assign) -> Eval {
        let value = self.eval(&assign.value)?;
        match self.frame.set(&assign.name, value.clone()) {
            Some(res) => res.source(&assign.span),
            None => {
                if let Some(receiver) = self.frame.receiver() {
                    if let Some(slot) = receiver.vtable.slots.get(&assign.name) {
                        return write_instance_variable(receiver, slot, value).source(&assign.span);
                    }
                }
                Unwind::error_at(assign.span.clone(), "Cannot assign to an unbound variable")
            }
        }
    }

    fn eval_var(&self, var: &Var) -> Eval {
        if &var.name == "self" {
            match self.frame.receiver() {
                None => Unwind::error_at(var.span.clone(), "self outside method context"),
                Some(receiver) => Ok(receiver.clone()),
            }
        } else {
            match self.frame.get(&var.name) {
                Some(value) => return Ok(value),
                None => {
                    if let Some(receiver) = self.frame.receiver() {
                        if let Some(slot) = receiver.vtable.slots.get(&var.name) {
                            return read_instance_variable(receiver, slot.index);
                        }
                    }
                }
            }
            Unwind::error_at(var.span.clone(), "Unbound variable")
        }
    }
}

pub fn apply(
    receiver: Option<&Object>,
    closure: &Closure,
    call_args: &[&Object],
    builtins: &Builtins,
) -> Eval {
    let mut args = HashMap::new();
    for (arg, obj) in closure.params.iter().zip(call_args.into_iter().map(|x| (*x).clone())) {
        let binding = match &arg.vtable {
            None => Binding::untyped(obj),
            Some(vtable) => {
                if vtable != &obj.vtable {
                    return Unwind::type_error_at(arg.span.clone(), obj, vtable.name.clone());
                }
                Binding::typed(vtable.to_owned(), obj.to_owned())
            }
        };
        args.insert(arg.name.clone(), binding);
    }
    let env = Env::from_parts(builtins, args, closure.env(), receiver.map(|x| x.clone()));
    match env.eval(&closure.body) {
        Err(Unwind::ReturnFrom(ref frame, ref value)) if frame == &env.frame => Ok(value.clone()),
        Ok(value) => Ok(value),
        Err(unwind) => Err(unwind),
    }
}

pub fn eval_all(builtins: &Builtins, source: &str) -> Eval {
    let env = Env::new(builtins);
    let mut parser = Parser::new(source);
    loop {
        let expr = match parser.parse() {
            Err(unwind) => {
                return Err(unwind.with_context(source));
            }
            Ok(expr) => expr,
        };
        let object = match env.eval(&expr) {
            Err(unwind) => {
                return Err(unwind.with_context(source));
            }
            Ok(object) => object,
        };
        if parser.at_eof() {
            return Ok(object);
        }
    }
}

fn eval_obj(source: &str) -> (Object, Builtins) {
    let builtins = Builtins::new();
    match eval_all(&builtins, source) {
        Err(unwind) => panic!("Unexpected unwind:\n{:?}", unwind),
        Ok(obj) => (obj, builtins),
    }
}

fn eval_exception(source: &str) -> (Unwind, Builtins) {
    let builtins = Builtins::new();
    match eval_all(&builtins, source) {
        Err(unwind) => match &unwind {
            Unwind::Exception(..) => (unwind, builtins),
            _ => panic!("Expected exception, got: {:?}", unwind),
        },
        Ok(value) => panic!("Expected exception, got: {:?}", value),
    }
}

fn eval_str(source: &str) -> Eval {
    let builtins = Builtins::new();
    eval_all(&builtins, source)
}

fn eval_ok(source: &str) -> Object {
    eval_str(source).unwrap()
}

#[test]
fn eval_decimal() {
    assert_eq!(eval_ok("123").integer(), 123);
}

#[test]
fn eval_bad_decimal() {
    assert_eq!(
        eval_str("1x3"),
        Err(Unwind::Exception(
            Error::SimpleError(SimpleError {
                what: "Malformed number",
            }),
            Location {
                span: Some(0..3),
                context: Some(concat!("001 1x3\n", "    ^^^ Malformed number\n").to_string())
            }
        ))
    );
}

#[test]
fn eval_hex() {
    assert_eq!(eval_ok("0xFFFF").integer(), 0xFFFF);
}

#[test]
fn eval_bad_hex() {
    assert_eq!(
        eval_str("0x1x3"),
        Err(Unwind::Exception(
            Error::SimpleError(SimpleError {
                what: "Malformed hexadecimal number",
            }),
            Location {
                span: Some(0..5),
                context: Some(
                    concat!("001 0x1x3\n", "    ^^^^^ Malformed hexadecimal number\n").to_string()
                )
            }
        ))
    );
}

#[test]
fn eval_binary() {
    assert_eq!(eval_ok("0b101").integer(), 0b101);
}

#[test]
fn eval_bad_binary() {
    assert_eq!(
        eval_str("0b123"),
        Err(Unwind::Exception(
            Error::SimpleError(SimpleError {
                what: "Malformed binary number",
            }),
            Location {
                span: Some(0..5),
                context: Some(
                    concat!("001 0b123\n", "    ^^^^^ Malformed binary number\n").to_string()
                )
            }
        ))
    );
}

#[test]
fn eval_float() {
    assert_eq!(eval_ok("1.2").float(), 1.2);
}

#[test]
fn eval_bad_float() {
    assert_eq!(
        eval_str("1.2.3"),
        Err(Unwind::Exception(
            Error::SimpleError(SimpleError {
                what: "Unknown operator",
            }),
            Location {
                span: Some(3..4),
                context: Some(concat!("001 1.2.3\n", "       ^ Unknown operator\n").to_string())
            }
        ))
    );
}

#[test]
fn eval_let1() {
    assert_eq!(eval_ok("let x = 42, x").integer(), 42);
}

#[test]
fn eval_let2() {
    assert_eq!(eval_ok("let x = 1, let x = 42, x").integer(), 42);
}

#[test]
fn eval_let3() {
    assert_eq!(eval_ok("let x = 42, let y = 1, x").integer(), 42);
}

#[test]
fn eval_arith1() {
    assert_eq!(eval_ok("1 + 1").integer(), 2);
}

#[test]
fn eval_arith2() {
    assert_eq!(eval_ok("1 + 1 * 2").integer(), 3);
}

#[test]
fn eval_div1() {
    assert_eq!(eval_ok("10 / 5").integer(), 2);
}

#[test]
fn eval_div2() {
    assert_eq!(eval_ok("10.0 / 5.0").float(), 2.0);
}

#[test]
fn eval_div3() {
    assert_eq!(eval_ok("10.0 / 5").float(), 2.0);
}

#[test]
fn eval_div4() {
    assert_eq!(eval_ok("10 / 5.0").float(), 2.0);
}

#[test]
fn eval_sub1() {
    assert_eq!(eval_ok("10 - 5").integer(), 5);
}

#[test]
fn eval_sub2() {
    assert_eq!(eval_ok("10.0 - 5.0").float(), 5.0);
}

#[test]
fn eval_sub3() {
    assert_eq!(eval_ok("10.0 - 5").float(), 5.0);
}

#[test]
fn eval_sub4() {
    assert_eq!(eval_ok("10 - 5.0").float(), 5.0);
}

#[test]
fn eval_add1() {
    assert_eq!(eval_ok("10 + 5").integer(), 15);
}

#[test]
fn eval_add2() {
    assert_eq!(eval_ok("10.0 + 5.0").float(), 15.0);
}

#[test]
fn eval_add3() {
    assert_eq!(eval_ok("10.0 + 5").float(), 15.0);
}

#[test]
fn eval_add4() {
    assert_eq!(eval_ok("10 + 5.0").float(), 15.0);
}

#[test]
fn eval_mul1() {
    assert_eq!(eval_ok("10 * 5").integer(), 50);
}

#[test]
fn eval_mul2() {
    assert_eq!(eval_ok("10.0 * 5.0").float(), 50.0);
}

#[test]
fn eval_mul3() {
    assert_eq!(eval_ok("10.0 * 5").float(), 50.0);
}

#[test]
fn eval_mul4() {
    assert_eq!(eval_ok("10 * 5.0").float(), 50.0);
}

#[test]
fn eval_assign1() {
    assert_eq!(eval_ok("let x = 1, x = x + 1, let y = x, y").integer(), 2);
}

#[test]
fn eval_assign_unbound() {
    assert_eq!(
        eval_str("let x = 1, z = x + 1, let y = x, y"),
        Err(Unwind::Exception(
            Error::SimpleError(SimpleError {
                what: "Cannot assign to an unbound variable",
            }),
            Location {
                span: Some(11..12),
                context: Some(
                    concat!(
                        "001 let x = 1, z = x + 1, let y = x, y\n",
                        "               ^ Cannot assign to an unbound variable\n"
                    )
                    .to_string()
                )
            }
        ))
    );
}

#[test]
fn eval_int_as_int() {
    assert_eq!(eval_ok("42 asInteger").integer(), 42);
}

#[test]
fn eval_float_as_float() {
    assert_eq!(eval_ok("42.3 asFloat").float(), 42.3);
}

#[test]
fn eval_int_as_float() {
    assert_eq!(eval_ok("42 asFloat").float(), 42.0);
}

#[test]
fn eval_float_as_int() {
    assert_eq!(eval_ok("42.1 asInteger").integer(), 42);
    assert_eq!(eval_ok("41.9 asInteger").integer(), 42);
}

#[test]
fn eval_unary() {
    assert_eq!(eval_ok("42 asFloat asInteger").integer(), 42);
}

#[test]
fn eval_keyword() {
    assert_eq!(eval_ok("15 gcd: 100").integer(), 5);
}

#[test]
fn eval_unbound() {
    assert_eq!(
        eval_str("let foo = 41, foo + bar"),
        Err(Unwind::Exception(
            Error::SimpleError(SimpleError {
                what: "Unbound variable",
            }),
            Location {
                span: Some(20..23),
                context: Some(
                    concat!(
                        "001 let foo = 41, foo + bar\n",
                        "                        ^^^ Unbound variable\n"
                    )
                    .to_string()
                )
            }
        ))
    );
}

#[test]
fn eval_closure1() {
    assert_eq!(eval_ok("let x = 41, { x + 1 } value").integer(), 42);
}

#[test]
fn eval_closure2() {
    assert_eq!(eval_ok("let x = 41, { |y| x + y } value: 1").integer(), 42);
}

#[test]
fn eval_closure3() {
    assert_eq!(
        eval_ok(
            "let thunk = { let counter = 0, { counter = counter + 1, counter } } value,
                        thunk value + thunk value"
        )
        .integer(),
        3
    );
}

#[test]
fn test_string_append() {
    assert_eq!(
        eval_ok(
            r#"
                 "foo" append: "bar"
             "#
        )
        .string_as_str(),
        "foobar"
    );
}

#[test]
fn eval_class_not_toplevel() {
    assert_eq!(
        eval_str("let x = 42, class Point { x, y } end"),
        Err(Unwind::Exception(
            Error::SimpleError(SimpleError {
                what: "Class definition not at toplevel",
            }),
            Location {
                span: Some(12..17),
                context: Some(
                    concat!(
                        "001 let x = 42, class Point { x, y } end\n",
                        "                ^^^^^ Class definition not at toplevel\n"
                    )
                    .to_string()
                )
            }
        ))
    );
}

#[test]
fn eval_class1() {
    let class = eval_ok("class Point { x, y } end").class();
    assert_eq!(class.instance_vtable.name, "Point");
    assert_eq!(
        class.instance_vtable.slots["x"],
        Slot {
            index: 0,
            vtable: None,
        }
    );
    assert_eq!(
        class.instance_vtable.slots["y"],
        Slot {
            index: 1,
            vtable: None,
        }
    );
}

#[test]
fn eval_global1() {
    assert_eq!(
        eval_str("DoesNotExist"),
        Err(Unwind::Exception(
            Error::SimpleError(SimpleError {
                what: "Undefined global",
            }),
            Location {
                span: Some(0..12),
                context: Some(
                    concat!("001 DoesNotExist\n", "    ^^^^^^^^^^^^ Undefined global\n")
                        .to_string()
                )
            }
        ))
    );
}

#[test]
fn eval_global2() {
    let class = eval_ok("Integer").class();
    assert_eq!(class.instance_vtable.name, "Integer");
    assert!(class.instance_vtable.slots.is_empty());
}

#[test]
fn eval_new_instance1() {
    let (object, builtins) = eval_obj("class Point { x, y } end, Point x: 1 y: 2");
    assert_eq!(object.send("x", &[], &builtins), Ok(builtins.make_integer(1)));
    assert_eq!(object.send("y", &[], &builtins), Ok(builtins.make_integer(2)));
}

#[test]
fn eval_new_instance2() {
    let (object, builtins) = eval_obj(
        "class Oh {}
            method no 42
            defaultConstructor noes
         end,
         Oh noes",
    );
    assert_eq!(object.send("no", &[], &builtins), Ok(builtins.make_integer(42)));
}

#[test]
fn eval_instance_method1() {
    let (object, builtins) = eval_obj(
        "class Foo {}
            method bar 311
         end,
         Foo new",
    );
    assert_eq!(object.send("bar", &[], &builtins), Ok(builtins.make_integer(311)));
}

#[test]
fn eval_instance_method2() {
    let (object, builtins) = eval_obj(
        "class Foo {}
            method foo
               self bar
            method bar
               311
         end,
         Foo new",
    );
    assert_eq!(object.send("bar", &[], &builtins), Ok(builtins.make_integer(311)));
}

#[test]
fn test_return_returns() {
    let (obj, builtins) = eval_obj(
        "class Foo {}
            method foo
               return 1,
               2
         end,
         Foo new foo",
    );
    assert_eq!(obj, builtins.make_integer(1));
}

#[test]
fn test_return_from_method_block() {
    let (obj, builtins) = eval_obj(
        "class Foo {}
            method test
                self boo: { return 42 },
                31
            method boo: blk
                blk value
         end,
         Foo new test
        ",
    );
    assert_eq!(obj, builtins.make_integer(42));
}

#[test]
fn test_return_from_deep_block_to_middle() {
    let (object, builtins) = eval_obj(
        "class Foo {}
            method test
               return 1 + let x = 41, self test0: x
            method test0: x
               self test1: { return x },
               return 100
            method test1: blk
               self test2: blk,
               return 1000
            method test2: blk
               blk value,
               return 10000
         end,
         Foo new test
        ",
    );
    assert_eq!(object, builtins.make_integer(42));
}

#[test]
fn test_string_interpolation1() {
    let (object, builtins) = eval_obj(
        r#"let a = 1
           let b = 3
           "{a}.{a+1}.{b}.{b+1}"
          "#,
    );
    assert_eq!(object, builtins.make_string("1.2.3.4"));
}

#[test]
fn test_typecheck1() {
    let (object, builtins) = eval_obj("123::Integer");
    assert_eq!(object, builtins.make_integer(123));
}

#[test]
fn test_typecheck2() {
    let (exception, foo) = eval_exception("123::String");
    assert_eq!(
        exception,
        Unwind::Exception(
            Error::TypeError(TypeError {
                value: foo.make_integer(123),
                expected: "String".to_string()
            }),
            Location {
                span: Some(0..3),
                context: Some(
                    concat!("001 123::String\n", "    ^^^ String expected, got: Integer 123\n")
                        .to_string()
                ),
            }
        )
    );
}

#[test]
fn test_typecheck3() {
    let (exception, foo) = eval_exception("let x::Integer = 42.0, x");
    assert_eq!(
        exception,
        Unwind::Exception(
            Error::TypeError(TypeError {
                value: foo.make_float(42.0),
                expected: "Integer".to_string()
            }),
            Location {
                span: Some(17..21),
                context: Some(
                    concat!(
                        "001 let x::Integer = 42.0, x\n",
                        "                     ^^^^ Integer expected, got: Float 42.0\n"
                    )
                    .to_string()
                )
            }
        )
    );
}

#[test]
fn test_typecheck4() {
    let (exception, foo) = eval_exception("let x::Integer = 42, x = 1.0, x");
    assert_eq!(
        exception,
        Unwind::Exception(
            Error::TypeError(TypeError {
                value: foo.make_float(1.0),
                expected: "Integer".to_string()
            }),
            Location {
                span: Some(21..22),
                context: Some(
                    concat!(
                        "001 let x::Integer = 42, x = 1.0, x\n",
                        "                         ^ Integer expected, got: Float 1.0\n"
                    )
                    .to_string()
                )
            }
        )
    );
}

#[test]
fn test_typecheck5() {
    assert_eq!(eval_ok("{ |x::Integer| x } value: 41").integer(), 41);
}

#[test]
fn test_typecheck6() {
    let (exception, foo) = eval_exception("{ |x::Integer| x } value: 41.0");
    assert_eq!(
        exception,
        Unwind::Exception(
            Error::TypeError(TypeError {
                value: foo.make_float(41.0),
                expected: "Integer".to_string()
            }),
            Location {
                span: Some(3..4),
                context: Some(
                    concat!(
                        "001 { |x::Integer| x } value: 41.0\n",
                        "       ^ Integer expected, got: Float 41.0\n"
                    )
                    .to_string()
                )
            }
        )
    );
}

#[test]
fn test_typecheck7() {
    let (exception, foo) = eval_exception("{ |y x::Integer| x = y } value: 41.0 value: 42");
    assert_eq!(
        exception,
        Unwind::Exception(
            Error::TypeError(TypeError {
                value: foo.make_float(41.0),
                expected: "Integer".to_string()
            }),
            Location {
                span: Some(17..18),
                context: Some(
                    concat!(
                        "001 { |y x::Integer| x = y } value: 41.0 value: 42\n",
                        "                     ^ Integer expected, got: Float 41.0\n"
                    )
                    .to_string()
                )
            }
        )
    );
}

#[test]
fn test_typecheck8() {
    let (exception, foo) = eval_exception(
        "class Foo {}
            defaultConstructor foo
            method zot: x::Integer
                x
         end
         Foo foo zot: 1.0",
    );
    assert_eq!(
        exception,
        Unwind::Exception(
            Error::TypeError(TypeError {
                value: foo.make_float(1.0),
                expected: "Integer".to_string()
            }),
            Location {
                span: Some(72..73),
                context: Some(
                    concat!(
                        "002             defaultConstructor foo\n",
                        "003             method zot: x::Integer\n",
                        "                            ^ Integer expected, got: Float 1.0\n",
                        "004                 x\n"
                    )
                    .to_string()
                )
            }
        )
    );
}

#[test]
fn test_instance_variable1() {
    assert_eq!(
        eval_ok(
            "class Foo { bar }
               method zot
                  bar
             end
             (Foo bar: 42) zot"
        )
        .integer(),
        42
    );
}

#[test]
fn test_instance_variable2() {
    assert_eq!(
        eval_ok(
            "class Foo { bar }
               method zit
                  bar = bar + 1
                  self
               method zot
                  bar
             end
             (Foo bar: 41) zit zot"
        )
        .integer(),
        42
    );
}

#[test]
fn test_instance_variable3() {
    assert_eq!(
        eval_ok(
            "class Foo { bar::Integer }
               method foo: x
                  bar = bar + x
                  self
             end
             ((Foo bar: 41) foo: 1) bar"
        )
        .integer(),
        42
    );
}

#[test]
fn test_instance_variable4() {
    let (exception, foo) = eval_exception(
        "class Foo { bar::Integer }
           method foo: x
              bar = bar + x
              self
         end
         ((Foo bar: 41) foo: 1.0) bar",
    );
    assert_eq!(
        exception,
        Unwind::Exception(
            Error::TypeError(TypeError {
                value: foo.make_float(42.0),
                expected: "Integer".to_string()
            }),
            Location {
                span: Some(66..69),
                context: Some(
                    concat!(
                        "002            method foo: x\n",
                        "003               bar = bar + x\n",
                        "                  ^^^ Integer expected, got: Float 42.0\n",
                        "004               self\n"
                    )
                    .to_string()
                )
            }
        )
    );
}
