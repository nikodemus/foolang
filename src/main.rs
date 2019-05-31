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

fn main() {
    println!("This is foolang 0.1.0");
}
