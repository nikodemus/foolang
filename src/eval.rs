use std::borrow::ToOwned;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::objects2::{Arg, Builtins, Closure, Eval, Object, Unwind, Vtable};
use crate::parse::{Assign, ClassDefinition, Expr, Global, Literal, Parser, Return, Var};
use crate::tokenstream::{Span, SyntaxError};

struct Env<'a> {
    builtins: &'a Builtins,
    frame: Rc<Frame>,
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
    fn assign(&mut self, value: Object) -> bool {
        if let Some(vtable) = &self.vtable {
            if &value.vtable != vtable {
                return false;
            }
        }
        self.value = value;
        return true;
    }
}

#[derive(Debug)]
pub struct Frame {
    pub local: RefCell<HashMap<String, Binding>>,
    pub parent: Option<Rc<Frame>>,
    pub method: Option<Rc<Frame>>,
}

impl PartialEq for Frame {
    fn eq(&self, other: &Self) -> bool {
        self as *const _ == other as *const _
    }
}

impl<'a> Env<'a> {
    pub fn new(builtins: &Builtins) -> Env {
        Env::from_parts(builtins, HashMap::new(), None, false)
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
        local: HashMap<String, Binding>,
        parent: Option<Rc<Frame>>,
        method: bool,
    ) -> Env<'a> {
        // KLUDGE: This is as simple as I can make this without making
        // a circular thing requiring weak pointers, which in turn would
        // complex in a different way...
        let parent_method = match &parent {
            None => None,
            Some(frame) => frame.parent.as_ref().map(Rc::clone),
        };
        let base_frame = Rc::new(Frame {
            local: RefCell::new(local),
            parent,
            method: parent_method,
        });
        let frame = if method {
            Rc::new(Frame {
                local: RefCell::new(HashMap::new()),
                parent: Some(Rc::clone(&base_frame)),
                method: Some(Rc::clone(&base_frame)),
            })
        } else {
            base_frame
        };
        Env {
            builtins,
            frame,
        }
    }

    fn eval_assign(&self, assign: &Assign) -> Eval {
        let name = &assign.name;
        // Value needs to be evaluated before we go looking for the binding,
        // so that the scope of our mutable borrow from the frame is safe.
        let value = self.eval(&assign.value)?;
        let mut frame = &self.frame;
        loop {
            match frame.local.borrow_mut().get_mut(name) {
                Some(binding) => {
                    if !binding.assign(value.clone()) {
                        return self.type_error(&assign.value);
                    }
                    return Ok(value);
                }
                None => match &frame.parent {
                    Some(parent_frame) => {
                        frame = parent_frame;
                    }
                    None => {
                        return Unwind::exception(SyntaxError::new(
                            assign.span.clone(),
                            "Cannot assign to an unbound variable",
                        ))
                    }
                },
            }
        }
    }

    fn eval_bind(
        &self,
        name: &String,
        typename: &Option<String>,
        expr: &Expr,
        body: &Expr,
    ) -> Eval {
        let mut local = HashMap::new();
        let binding = match typename {
            None => Binding::untyped(self.eval(expr)?),
            Some(typename) => {
                let class = self.find_class(typename, expr.span())?.class();
                Binding::typed(class.instance_vtable.clone(), self.eval_typecheck(expr, typename)?)
            }
        };
        local.insert(name.to_owned(), binding);
        let env = Env::from_parts(self.builtins, local, Some(Rc::clone(&self.frame)), false);
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
        Ok(self.builtins.make_closure(Rc::clone(&self.frame), args, body.to_owned()))
    }

    fn eval_class_definition(&self, definition: &ClassDefinition) -> Eval {
        if self.frame.parent.is_some() {
            return Unwind::exception(SyntaxError::new(
                definition.span.clone(),
                "Class definition not at toplevel",
            ));
        }
        let name = &definition.name;
        let class = self.builtins.make_class(definition);
        self.builtins.globals.borrow_mut().insert(name.to_string(), class.clone());
        Ok(class)
    }

    fn eval_global(&self, global: &Global) -> Eval {
        match self.builtins.globals.borrow().get(&global.name) {
            Some(obj) => Ok(obj.clone()),
            None => Unwind::exception(SyntaxError::new(global.span.clone(), "Undefined global")),
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
        match &self.frame.method {
            None => {
                Unwind::exception(SyntaxError::new(ret.span.clone(), "No method to return from"))
            }
            Some(frame) => Unwind::return_from(Rc::clone(frame), self.eval(&ret.value)?),
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

    fn type_error(&self, expr: &Expr) -> Eval {
        // FIXME: Get type types into the error message
        // FIXME: wrong span
        Unwind::exception(SyntaxError::new(expr.span(), "TypeError"))
    }

    fn eval_typecheck(&self, expr: &Expr, typename: &str) -> Eval {
        let value = self.eval(expr)?;
        // FIXME: Wrong span.
        let class = self.find_class(typename, expr.span())?.class();
        if class.instance_vtable == value.vtable {
            Ok(value)
        } else {
            self.type_error(expr)
        }
    }

    fn eval_var(&self, var: &Var) -> Eval {
        let mut frame = &self.frame;
        loop {
            match frame.local.borrow().get(&var.name) {
                Some(binding) => return Ok(binding.value.to_owned()),
                None => match &frame.parent {
                    Some(parent_frame) => {
                        frame = parent_frame;
                    }
                    None => {
                        return Unwind::exception(SyntaxError::new(
                            var.span.clone(),
                            "Unbound variable",
                        ))
                    }
                },
            }
        }
    }
}

pub fn apply(closure: &Object, args: &[&Object], builtins: &Builtins) -> Eval {
    apply_with_extra_args(&closure.closure(), args, &[], builtins, false)
}

pub fn apply_with_extra_args(
    closure: &Closure,
    args: &[&Object],
    extra: &[&Object],
    builtins: &Builtins,
    method: bool,
) -> Eval {
    // KLUDGE: I'm blind. I would think that iterating over args with IntoIterator
    // would give me an iterator over &Object, but I get &&Object -- so to_owned x 2.
    let mut locals = HashMap::new();
    for (arg, obj) in
        closure.params.iter().zip(args.into_iter().chain(extra.into_iter()).map(|x| x.to_owned()))
    {
        let binding = match &arg.vtable {
            None => Binding::untyped(obj.to_owned()),
            Some(vtable) => {
                if vtable != &obj.vtable {
                    return Unwind::exception(SyntaxError::new(arg.span.clone(), "TypeError"));
                }
                Binding::typed(vtable.to_owned(), obj.to_owned())
            }
        };
        locals.insert(arg.name.clone(), binding);
    }
    /*
    let locals: HashMap<String, Binding> = closure
       .params
       .iter()
       .map(|p| p.name.clone())
       .zip(
           args.into_iter()
               .chain(extra.into_iter())
               .zip(closure.params)
               .map(|obj, arg| Binding::new(obj.to_owned().to_owned(), arg.vtable.clone())),
       )
       .collect();
       */
    let parent = closure.env.as_ref().map(|x| Rc::clone(x));
    let env = Env::from_parts(builtins, locals, parent, method);
    match env.eval(&closure.body) {
        Ok(value) => Ok(value),
        Err(Unwind::Exception(e)) => Err(Unwind::Exception(e)),
        Err(Unwind::ReturnFrom(frame, value)) => {
            if Some(&frame) == env.frame.parent.as_ref() {
                Ok(value)
            } else {
                Err(Unwind::ReturnFrom(frame, value))
            }
        }
    }
}

fn eval_all(builtins: &Builtins, source: &str) -> Eval {
    let env = Env::new(builtins);
    let mut parser = Parser::new(source);
    loop {
        let expr = match parser.parse() {
            Err(err) => return Err(err.add_context(source)),
            Ok(expr) => expr,
        };
        let object = match env.eval(&expr) {
            Err(err) => return Err(err.add_context(source)),
            Ok(object) => object,
        };
        if parser.at_eof() {
            return Ok(object);
        }
    }
}

fn eval_builtins(source: &str) -> (Object, Builtins) {
    let builtins = Builtins::new();
    match eval_all(&builtins, source) {
        Err(err) => panic!("Unexpected exception:\n{:?}", err),
        Ok(obj) => (obj, builtins),
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
        Unwind::exception(SyntaxError {
            span: 0..3,
            problem: "Malformed number",
            context: concat!("001 1x3\n", "    ^^^ Malformed number\n").to_string()
        })
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
        Unwind::exception(SyntaxError {
            span: 0..5,
            problem: "Malformed hexadecimal number",
            context: concat!("001 0x1x3\n", "    ^^^^^ Malformed hexadecimal number\n").to_string()
        })
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
        Unwind::exception(SyntaxError {
            span: 0..5,
            problem: "Malformed binary number",
            context: concat!("001 0b123\n", "    ^^^^^ Malformed binary number\n").to_string()
        })
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
        Unwind::exception(SyntaxError {
            span: 3..4,
            problem: "Unknown operator",
            context: concat!("001 1.2.3\n", "       ^ Unknown operator\n").to_string()
        })
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
        Unwind::exception(SyntaxError {
            span: 11..12,
            problem: "Cannot assign to an unbound variable",
            context: concat!(
                "001 let x = 1, z = x + 1, let y = x, y\n",
                "               ^ Cannot assign to an unbound variable\n"
            )
            .to_string()
        })
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
        Unwind::exception(SyntaxError {
            span: 20..23,
            problem: "Unbound variable",
            context: concat!(
                "001 let foo = 41, foo + bar\n",
                "                        ^^^ Unbound variable\n"
            )
            .to_string()
        })
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
        Unwind::exception(SyntaxError {
            span: 12..17,
            problem: "Class definition not at toplevel",
            context: concat!(
                "001 let x = 42, class Point { x, y } end\n",
                "                ^^^^^ Class definition not at toplevel\n"
            )
            .to_string()
        })
    );
}

#[test]
fn eval_class1() {
    let class = eval_ok("class Point { x, y } end").class();
    assert_eq!(class.instance_vtable.name, "Point");
    assert_eq!(class.instance_variables, vec!["x".to_string(), "y".to_string()]);
}

#[test]
fn eval_global1() {
    assert_eq!(
        eval_str("DoesNotExist"),
        Unwind::exception(SyntaxError {
            span: 0..12,
            problem: "Undefined global",
            context: concat!("001 DoesNotExist\n", "    ^^^^^^^^^^^^ Undefined global\n")
                .to_string()
        })
    );
}

#[test]
fn eval_global2() {
    let class = eval_ok("Integer").class();
    assert_eq!(class.instance_vtable.name, "Integer");
    assert_eq!(class.instance_variables, Vec::<String>::new());
}

#[test]
fn eval_new_instance1() {
    let (object, builtins) = eval_builtins("class Point { x, y } end, Point x: 1 y: 2");
    assert_eq!(object.send("x", &[], &builtins), Ok(builtins.make_integer(1)));
    assert_eq!(object.send("y", &[], &builtins), Ok(builtins.make_integer(2)));
}

#[test]
fn eval_new_instance2() {
    let (object, builtins) = eval_builtins(
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
    let (object, builtins) = eval_builtins(
        "class Foo {}
            method bar 311
         end,
         Foo new",
    );
    assert_eq!(object.send("bar", &[], &builtins), Ok(builtins.make_integer(311)));
}

#[test]
fn eval_instance_method2() {
    let (object, builtins) = eval_builtins(
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
    let (obj, builtins) = eval_builtins(
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
    let (obj, builtins) = eval_builtins(
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
    let (object, builtins) = eval_builtins(
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
    let (object, builtins) = eval_builtins(
        r#"let a = 1
           let b = 3
           "{a}.{a+1}.{b}.{b+1}"
          "#,
    );
    assert_eq!(object, builtins.make_string("1.2.3.4"));
}

#[test]
fn test_typecheck1() {
    let (object, builtins) = eval_builtins("123::Integer");
    assert_eq!(object, builtins.make_integer(123));
}

#[test]
fn test_typecheck2() {
    assert_eq!(
        eval_str("123::String"),
        Unwind::exception(SyntaxError {
            span: 0..3,
            problem: "TypeError",
            context: concat!("001 123::String\n", "    ^^^ TypeError\n").to_string(),
        }),
    );
}

#[test]
fn test_typecheck3() {
    assert_eq!(
        eval_str("let x::Integer = 42.0, x"),
        Unwind::exception(SyntaxError {
            span: 17..21,
            problem: "TypeError",
            context: concat!(
                "001 let x::Integer = 42.0, x\n",
                "                     ^^^^ TypeError\n"
            )
            .to_string()
        })
    );
}

#[test]
fn test_typecheck4() {
    assert_eq!(
        eval_str("let x::Integer = 42, x = 1.0, x"),
        Unwind::exception(SyntaxError {
            span: 25..28,
            problem: "TypeError",
            context: concat!(
                "001 let x::Integer = 42, x = 1.0, x\n",
                "                             ^^^ TypeError\n"
            )
            .to_string()
        })
    );
}

#[test]
fn test_typecheck5() {
    assert_eq!(eval_ok("{ |x::Integer| x } value: 41").integer(), 41);
}

#[test]
fn test_typecheck6() {
    assert_eq!(
        eval_str("{ |x::Integer| x } value: 41.0"),
        Unwind::exception(SyntaxError {
            span: 3..4,
            problem: "TypeError",
            context: concat!("001 { |x::Integer| x } value: 41.0\n", "       ^ TypeError\n")
                .to_string()
        })
    );
}

#[test]
fn test_typecheck7() {
    assert_eq!(
        eval_str("{ |y x::Integer| x = y } value: 41.0 value: 42"),
        Unwind::exception(SyntaxError {
            span: 21..22,
            problem: "TypeError",
            context: concat!(
                "001 { |y x::Integer| x = y } value: 41.0 value: 42\n",
                "                         ^ TypeError\n"
            )
            .to_string()
        })
    );
}

#[test]
fn test_typecheck8() {
    assert_eq!(
        eval_str(
            "class Foo {}
                defaultConstructor foo
                method zot: x::Integer
                    x
             end
             Foo foo zot: 1.0",
        ),
        Unwind::exception(SyntaxError {
            span: 80..81,
            problem: "TypeError",
            context: concat!(
                "002                 defaultConstructor foo\n",
                "003                 method zot: x::Integer\n",
                "                                ^ TypeError\n",
                "004                     x\n"
            )
            .to_string()
        })
    );
}

#[test]
fn test_instance_variable1() {
    assert_eq!(
        parse_str(
            "class Foo { bar }
               method zot
                  bar
             end
             (Foo bar: 42) zot"
        ),
        Ok(int(0..0, 0))
    );
}
