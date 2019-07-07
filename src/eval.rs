use std::borrow::ToOwned;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::objects2::{Builtins, Closure, Eval, Object, Unwind};
use crate::parse::{
    parse_str, Assign, ClassDefinition, Expr, Global, Literal, Parser, Return, SyntaxError, Var,
};

struct Env<'a> {
    builtins: &'a Builtins,
    frame: Rc<Frame>,
}

#[derive(PartialEq, Debug)]
pub struct Frame {
    local: RefCell<HashMap<String, Object>>,
    parent: Option<Rc<Frame>>,
}

impl<'a> Env<'a> {
    pub fn new(builtins: &Builtins) -> Env {
        Env::from_parts(builtins, HashMap::new(), None)
    }

    pub fn eval(&self, expr: &Expr) -> Eval {
        use Expr::*;
        match expr {
            Assign(assign) => self.eval_assign(assign),
            Bind(name, value, body) => self.eval_bind(name, value, body),
            Block(_, params, body) => self.eval_block(params, body),
            ClassDefinition(definition) => self.eval_class_definition(definition),
            Const(_, literal) => self.eval_literal(literal),
            Global(global) => self.eval_global(global),
            Return(ret) => self.eval_return(ret),
            Send(_, selector, receiver, args) => self.eval_send(selector, receiver, args),
            Seq(exprs) => self.eval_seq(exprs),
            Var(var) => self.eval_var(var),
        }
    }

    fn from_parts(
        builtins: &'a Builtins,
        local: HashMap<String, Object>,
        parent: Option<Rc<Frame>>,
    ) -> Env<'a> {
        Env {
            builtins,
            frame: Rc::new(Frame {
                local: RefCell::new(local),
                parent,
            }),
        }
    }

    fn bind(&self, name: &String, value: Object) -> Env {
        let mut local = HashMap::new();
        local.insert(name.to_owned(), value);
        Env::from_parts(self.builtins, local, Some(Rc::clone(&self.frame)))
    }

    fn eval_assign(&self, assign: &Assign) -> Eval {
        let name = &assign.name;
        // Value needs to be evaluated before we go looking for the binding,
        // so that the scope of our mutable borrow from the frame is safe.
        let value = self.eval(&assign.value)?;
        let mut frame = &self.frame;
        loop {
            match frame.local.borrow_mut().get_mut(name) {
                Some(place) => {
                    *place = value.clone();
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

    fn eval_bind(&self, name: &String, value: &Expr, body: &Expr) -> Eval {
        self.bind(name, self.eval(value)?).eval(body)
    }

    fn eval_block(&self, params: &Vec<String>, body: &Expr) -> Eval {
        Ok(self.builtins.make_closure(Rc::clone(&self.frame), params.to_owned(), body.to_owned()))
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
        }
    }

    fn eval_return(&self, ret: &Return) -> Eval {
        unimplemented!("eval_return")
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
            result = self.eval(expr)?
        }
        Ok(result)
    }

    fn eval_var(&self, var: &Var) -> Eval {
        let mut frame = &self.frame;
        loop {
            match frame.local.borrow().get(&var.name) {
                Some(value) => return Ok(value.to_owned()),
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
    apply_with_extra_args(&closure.closure(), args, &[], builtins)
}

pub fn apply_with_extra_args(
    closure: &Closure,
    args: &[&Object],
    extra: &[&Object],
    builtins: &Builtins,
) -> Eval {
    // KLUDGE: I'm blind. I would think that iterating over args with IntoIterator
    // would give me an iterator over &Object, but I get &&Object -- so to_owned x 2.
    let locals: HashMap<String, Object> = closure
        .params
        .iter()
        .map(String::clone)
        .zip(args.into_iter().chain(extra.into_iter()).map(|obj| obj.to_owned().to_owned()))
        .collect();
    let myframe = match &closure.env {
        Some(frame) => Some(Rc::clone(&frame)),
        None => None,
    };
    let env = Env::from_parts(builtins, locals, myframe);
    env.eval(&closure.body)
}

fn eval_str(source: &str) -> Eval {
    let builtins = Builtins::new();
    let expr = parse_str(source)?;
    Env::new(&builtins).eval(&expr).map_err(|e| e.add_context(source))
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
            span: 0..5,
            problem: "Malformed number",
            context: concat!("001 1.2.3\n", "    ^^^^^ Malformed number\n").to_string()
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
fn eval_class_not_toplevel() {
    assert_eq!(
        eval_str("let x = 42, @class Point { x, y } @end"),
        Unwind::exception(SyntaxError {
            span: 12..18,
            problem: "Class definition not at toplevel",
            context: concat!(
                "001 let x = 42, @class Point { x, y } @end\n",
                "                ^^^^^^ Class definition not at toplevel\n"
            )
            .to_string()
        })
    );
}

#[test]
fn eval_class1() {
    let class = eval_ok("@class Point { x, y } @end").class();
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
    let (object, builtins) = eval_builtins("@class Point { x, y } @end, Point x: 1 y: 2");
    assert_eq!(object.send("x", &[], &builtins), Ok(builtins.make_integer(1)));
    assert_eq!(object.send("y", &[], &builtins), Ok(builtins.make_integer(2)));
}

#[test]
fn eval_new_instance2() {
    let (object, builtins) = eval_builtins(
        "@class Oh {}
            method no 42
            defaultConstructor noes
         @end,
         Oh noes",
    );
    assert_eq!(object.send("no", &[], &builtins), Ok(builtins.make_integer(42)));
}

#[test]
fn eval_instance_method1() {
    let (object, builtins) = eval_builtins(
        "@class Foo {}
            method bar 311
         @end,
         Foo new",
    );
    assert_eq!(object.send("bar", &[], &builtins), Ok(builtins.make_integer(311)));
}

#[test]
fn eval_instance_method2() {
    let (object, builtins) = eval_builtins(
        "@class Foo {}
            method foo
               self bar
            method bar
               311
         @end,
         Foo new",
    );
    assert_eq!(object.send("bar", &[], &builtins), Ok(builtins.make_integer(311)));
}

#[ignore]
#[test]
fn eval_return_returns() {
    let (obj, builtins) = eval_builtins(
        "@class Foo {}
            method foo
               return 1,
               2
         @end,
         Foo new foo",
    );
    assert_eq!(obj, builtins.make_integer(1));
}
