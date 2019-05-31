/**
 * Let's be crystal clear here: this is not an efficient
 * VM implementation or object representation.

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::rc::Rc;

struct Core {
    classes: Vec<Class>,
    symbols: Vec<Rc<Object>>,
}

static mut CORE: Option<Core> = None;

impl Core {
    fn get() -> &'static Core {
        unsafe match CORE {
            None => {
                let classes: Vec<Rc<Object>> = Vec::new();
                let symbols: Vec<Rc<Object>> = Vec::new();
                let s_symbol = Rc::new(Object {
                    class: 0,
                    value: Value::Str(String::from("Symbol")),
                });
                let
                classes.push(
                    Class {
                        name: s_symbol.clone(),
                        methods: Vec::new(),
                    }
                );
                let s_class = Rc::new(Object {
                    class: 1,
                    value: Value::Slots()
                })
                let o_symbol =
                let c_symbol = Class {
                    name: None,
                    methods: Vec::new(),
                };
                c_symbol.name = Some()
                let symbol = ;
                let s_symbol = Object {
                    class:
                }
                let s_class = Value::Sym(Rc::new(String::from("Class")));
                let c_symbol = Class {
                    name: symval.clone(),
                    methods: Vec::new(),
                };
                let
                CORE = Core::init()
                &CORE
            }
            Some(core) => &core
        }
    }
    fn int(&self, x: i64) -> Rc<Object> {
        Rc::new(Object {
            class: self.INT,
            value: Value::Int(x),
        })
    }
    fn intern(&mut self, name: &str) -> Rc<Object> {
        for sym in self.symbols.iter() {
            if let Value::Sym(_) = sym.value {
                return sym.to_owned();
            } else {
                panic!("Non-symbol in symbol table!");
            }
        }
        let sym = Rc::new(Object {
            class: self.SYMBOL,
            value: Value::Sym(String::from(name)),
        });
        self.symbols.push(sym.clone());
        sym;
    }
    fn class(&mut self, name: Rc<Object>) -> Rc<Class> {
        asert!(name.class == SYMBOL);
        for c in self.classes.iter() {
            if c.name == name {
                panic!("Cannot redefine classes yet...");
            }
        }
        let class = Rc::new(Class {
                name: intern(name),
                methods: Vec::new(),
        });
        self.classes.push(class.clone());
        class
    }

    }
}

struct Object {
    class: usize,
    value: Value,
}

enum Value {
    Str(String),
    Slots(Vec<Rc<Object>>),
}


struct Method {
  name: Rc<Object>,
  constants: Vec<Rc<Object>>,
  code: Vec<u8>,
}

struct Class {
  name: Rc<Object>,
  methods: Vec<Rc<Method>>,
}

struct MethodContext {
    registers: Vec<Rc<Object>>,
    method: Rc<Method>,
    myself: Rc<Object>,
    ip: usize,
}

struct VM {
    stack: Vec<MethodContext>,
    context: MethodContext,
    receiver: Rc<Object>,
    args: Vec<Rc<Object>>,
}

impl VM {
    fn run(&mut self) {
        loop {
            OPCODES[bytecode()](self);
        }
    }
    fn send(&mut self) {
        let m = self.receiver.class.methods[self.constant()];
        self.stack.push(self.context);
        self.context = MethodContext {
            registers: self.args,
            method: m,
            myself: self.receiver,
            ip: 0,
        };
        self.args = Vec::new();
    }
    fn ret(&mut self) {
        self.context = stack.pop();
    }
    fn push(&mut self, obj: Rc<Object>) {
        self.args.push(obj);
    }
    fn set_receiver(&mut self, obj: Rc<Object>) {
        self.receiver = obj;
    }
    fn set_slot(&mut self, obj: Rc<Object>) {
        self.context.myself.slots[self.bytecode()] = obj;
    }
    fn set_register(&mut self, obj: Rc<Object>) {
        self.context.registers[self.bytecode()] = obj;
    }
    fn bytecode(&mut self) -> u8 {
        let ip = self.context.ip;
        self.context.ip = ip + 1;
        self.context.method.code[ip]
    }
    fn receiver(&self) -> Rc<Object> {
        self.receiver.clone()
    }
    fn myself(&self) -> Rc<Object> {
        self.context.myself.clone()
    }
    fn constant(&self) -> Rc<Object> {
        self.context.method.constants[self.bytecode()].clone()
    }
    fn register(&self) -> Rc<Object> {
        self.context.registers[self.bytecode()].clone()
    }
    fn slot(&self) -> Rc<Object> {
        self.context.myself.slots[self.bytecode()].clone()
    }
}

fn op_unimplemented(vm: &mut VM) {
    unimplemented!("unimplemented bytecode");
}

fn op_push_self(vm: &mut VM) {
    vm.push(vm.myself());
}

fn op_self(vm: &mut VM) {
    vm.set_receiver(vm.myself());
}

fn op_return(vm: &mut VM) {
    vm.ret();
}

fn op_push_const(vm: &mut VM) {
    vm.push(vm.constant());
}

fn op_push_reg(vm: &mut VM) {
    vm.push(vm.register());
}

fn op_push_slot(vm: &mut VM) {
    vm.push(vm.slot());
}

fn op_push_receiver(vm: &mut VM) {
    vm.push(vm.receiver());
}

fn op_const(vm: &mut VM) {
    vm.set_receiver(vm.constant());
}

fn op_reg(vm: &mut VM) {
    vm.set_receiver(vm.register());
}

fn op_slot(vm: &mut VM) {
    vm.set_receiver(vm.slot());
}

fn op_send(vm: &mut VM) {
    vm.send();
}

fn op_self_to_reg(vm: &mut VM) {
    vm.set_register(vm.myself());
}

fn op_receiver_to_reg(vm: &mut VM) {
    vm.set_register(vm.receiver());
}

fn op_self_to_slot(vm: &mut VM) {
    vm.set_slot(vm.myself());
}

fn op_receiver_to_slot(vm: &mut VM) {
    vm.set_slot(vm.receiver());
}

fn op_const_to_reg(vm: &mut VM) {
    vm.set_register(vm.constant());
}

fn op_slot_to_reg(vm: &mut VM) {
    vm.set_register()
}

fn op_reg_to_reg(vm: &mut VM) {
    vm.set_register(vm.register());
}

fn op_const_to_slot(vm: &mut VM) {
    vm.set_slot(vm.constant());
}

fn op_slot_to_slot(vm: &mut VM) {
    vm.set_slot(vm.slot());
}

const OPCODES: [(&str,&Fn(&mut VM) -> ()); 20] = [
    ("self", &op_self),
    ("const", &op_const),
    ("reg", &op_reg),
    ("slot", &op_slot),
    ("push_self", &op_push_self),
    ("return", &op_return),
    ("push_const", &op_push_const),
    ("push_reg", &op_push_reg),
    ("push_slot", &op_push_slot),
    ("push_receiver", &op_push_receiver),
    ("send", &op_send),
    ("self_to_reg", &op_self_to_reg),
    ("receiver_to_reg", &op_receiver_to_reg),
    ("self_to_slot", &op_self_to_slot),
    ("receiver_to_slot", &op_receiver_to_slot),
    ("const_to_reg", &op_const_to_reg),
    ("slot_to_reg", &op_slot_to_reg),
    ("reg_to_reg", &op_reg_to_reg),
    ("const_to_slot", &op_const_to_slot),
    ("slot_to_slot", &op_slot_to_slot),
];

fn exec(obj: Rc<Object>, selector: Rc<object>) {
    let vm = VM {
        stack: Vec::new(),
        context: MethodContext {
            registers: Vec::new(),
            method: obj.class.methods[selector],
            myself: obj,
            ip: 0,
        },
        receiver: obj.clone(),
        args: Vec::new(),
    };
    vm.run();
}

#[test]
fn test_easy() {

    let m = Method::new();
    m.emit_constant(Object::int(42));
    m.emit("push_const")
   let r1 = Object.new();
   let m1 = r1.add_method("foo", Method.new());
   m1.emit(Bytecode::sendMessage(
}

*/

use lalrpop_util::lalrpop_mod;
use foolang::syntax::Expr;

lalrpop_mod!(pub syntax);

fn parse_expr(s: &str) -> Expr {
    match syntax::ExpressionParser::new().parse(s) {
        Ok(e) => e,
        Err(e) => {
            panic!(format!("Could not parse expression: {}\nError: {}", s, e));
        }
    }
}

#[cfg(test)]
mod tests {
    use foolang::syntax::{Expr, Literal, Identifier};
    use crate::parse_expr;

    // helpers
    fn s(s: &str) -> String {
        s.to_string()
    }
    fn identifier(s: &str) -> Identifier {
        Identifier(s.to_string())
    }
    fn variable(s: &str) -> Expr {
        Expr::Variable(identifier(s))
    }

    #[test]
    fn parse_literals() {
        assert_eq!(parse_expr("42"), Expr::Constant(Literal::Integer(42)));
        assert_eq!(parse_expr("12.23"), Expr::Constant(Literal::Float(12.23)));
        assert_eq!(parse_expr("$x"), Expr::Constant(Literal::Character(s("x"))));
        assert_eq!(parse_expr("#foo:bar:"), Expr::Constant(Literal::Symbol(s("foo:bar:"))));
        assert_eq!(parse_expr("'bleep''bloop'"), Expr::Constant(Literal::String(s("bleep''bloop"))));
        assert_eq!(parse_expr("#(321 34.5 $$ _foobar:quux:zot: 'string' (level2))"),
            Expr::Constant(Literal::Array(vec![
                Literal::Integer(321),
                Literal::Float(34.5),
                Literal::Character("$".to_string()),
                Literal::Symbol(s("_foobar:quux:zot:")),
                Literal::String(s("string")),
                Literal::Array(vec![Literal::Symbol(s("level2"))]),
                ])));
    }
    #[test]
    fn parse_variable() {
        assert_eq!(parse_expr("foo"), variable("foo"));
    }
    #[test]
    fn parse_unary() {
        assert_eq!(parse_expr("foo bar"), Expr::Unary(
            Box::new(variable("foo")),
            identifier("bar")));
    }
    #[test]
    fn parse_binary() {
        assert_eq!(parse_expr("a + b"), Expr::Binary(
            Box::new(variable("a")),
            identifier("+"),
            Box::new(variable("b"))));
        assert_eq!(parse_expr("a + b ** c"), Expr::Binary(
            Box::new(Expr::Binary(
                Box::new(variable("a")),
                identifier("+"),
                Box::new(variable("b")))),
            identifier("**"),
            Box::new(variable("c"))));
    }
    #[test]
    fn parse_keyword() {
        assert_eq!(parse_expr("x foo: y bar: z"), Expr::Keyword(
            Box::new(variable("x")),
            vec![identifier("foo:"), identifier("bar:")],
            vec![variable("y"), variable("z")]));
    }
    #[test]
    fn parse_assign() {
        assert_eq!(parse_expr("foo := foo bar quux"), Expr::Assign(
            Identifier(s("foo")),
            Box::new(Expr::Unary(
                Box::new(Expr::Unary(
                    Box::new(Expr::Variable(Identifier(s("foo")))),
                    Identifier(s("bar")))),
                Identifier(s("quux"))))));
    }
    #[test]
    fn parse_block() {
        assert_eq!(parse_expr("{ foo }"), Expr::Block(
            vec![], vec![variable("foo")]
        ));
        assert_eq!(parse_expr("{ foo bar }"), Expr::Block(
            vec![],
            vec![Expr::Unary(
                    Box::new(variable("foo")),
                    identifier("bar"))
                ]
        ));
        assert_eq!(parse_expr("{ foo bar. quux }"), Expr::Block(
            vec![],
            vec![Expr::Unary(
                    Box::new(variable("foo")),
                    identifier("bar")),
                variable("quux")]
        ));
        assert_eq!(parse_expr("{ :a | foo bar }"), Expr::Block(
            vec![identifier("a")],
            vec![Expr::Unary(
                    Box::new(variable("foo")),
                    identifier("bar"))
                ]
        ));
        assert_eq!(parse_expr("{ :a | foo bar. quux }"), Expr::Block(
            vec![identifier("a")],
            vec![Expr::Unary(
                    Box::new(variable("foo")),
                    identifier("bar")),
                variable("quux")]
        ));
        assert_eq!(parse_expr("{ :a | foo + bar. quux }"), Expr::Block(
            vec![identifier("a")],
            vec![Expr::Binary(
                    Box::new(variable("foo")),
                    identifier("+"),
                    Box::new(variable("bar"))),
                variable("quux")]
        ));
        assert_eq!(parse_expr("{ :a | foo with: bar and: a. quux }"), Expr::Block(
            vec![identifier("a")],
            vec![Expr::Keyword(
                    Box::new(variable("foo")),
                    vec![identifier("with:"), identifier("and:")],
                    vec![variable("bar"), variable("a")]),
                variable("quux")]
        ));
        assert_eq!(parse_expr("{ ^Foo new }"), Expr::Block(
            vec![],
            vec![Expr::Return(Box::new(Expr::Unary(
                    Box::new(variable("Foo")),
                    identifier("new"))))]));
    }
}

fn main() {
    println!("This is foolang 0.1.0");
}
