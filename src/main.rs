

struct Method {
  name: String,
  code: Vec<u8>,
  constants: Vec<Rc<Object>>,
}

struct Class {
  name: String,
  methods: HashMap<String,Rc<Method>>,
}

struct Object {
  class: Class,
  slots: Vec<Rc<Object>>,
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

const OPCODES: [(&str,OpImpl), 20] = [
    ("self", op_self),
    ("const", op_const),
    ("reg", op_reg),
    ("slot", op_slot),
    ("push_self", op_push_self),
    ("return", op_return),
    ("push_const", op_push_const),
    ("push_reg", op_push_reg),
    ("push_slot", op_push_slot),
    ("push_receiver", op_push_receiver),
    ("send", op_send),
    ("self_to_reg", op_self_to_reg),
    ("receiver_to_reg", op_receiver_to_reg),
    ("self_to_slot", op_self_to_slot),
    ("receiver_to_slot", op_receiver_to_slot),
    ("const_to_reg", op_const_to_reg),
    ("slot_to_reg", op_slot_to_reg),
    ("reg_to_reg", op_reg_to_reg),
    ("const_to_slot", op_const_to_slot),
    ("slot_to_slot", op_slot_to_slot),
];

fn exec() {
   // XXX: If everything is 256 in size I can just use arrays! (Just need an arg pointer?)
   let context = MethodContext {
       registers: Vec::new(),
       myself: myself,
       method: method,
       ip: 0,
   };
   let stack: Vec<MethodContext> = Vec::new();
   let receiver = NONE;
   let args: Vec<Object> = Vec::new();

   loop {
       let opcode = contex.method.bytecode[context.ip()];

       if single_byte_opcode(opcode) {

       }

         // Passing arguments
         Bytecode::ConstArg(i)    => args.push(method.constants[i]),
         Bytecode::RegisterArg(i) => args.push(registers[i]),
         Bytecode::SelfArg()      => args.push(myself),
         Bytecode::SlotArg(i)     => args.push(myself.slots[i]),

         // Setting receiver
         Bytecode::ConstReceiver(i)    => { receiver = method.constants[i]; },
         Bytecode::RegisterReceiver(i) => { receiver = registers[i]; }
         Bytecode::SelfReceiver        => { receiver = myself; }
         Bytecode::SlotReceiver(i)     => { receiver = myself.slots[i]; }

         // Sending the message
         Bytecode::SendMessage(i) => {
           let m = myself.methods[method.constants[i]],
           stack.push(context);
           context = MethodContext {
                   registers: args,
                   myself: receiver,
                   method: m,
                   ip: 0,
           };
           receiver = NONE;
           args = Vec::new();
         },

         // Returning a value
         Bytecode::ReturnReceiver => {
            context = stack.pop();
         }
     }
  }
}

#[test]
fn test_send1() {
   let r1 = Object.new();
   let m1 = r1.add_method("foo", Method.new());
   m1.emit(Bytecode::sendMessage(
}

fn main() {
    println!("Hello, world!");
}
