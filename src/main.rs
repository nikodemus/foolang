struct Context {
  myself: Object,
  receiver: Object,
  method: Method,
  args: Vec<Object>,
  registers: Vec<Object>,
  // Vector of contexts would probably be better, but this works too.
  parent: Box<Context>,
};

impl Context {
  fn call(self, receiver: Object, method: Method, args: Vec<Object>) -> Context {
     Context {
       myself: self.receiver,
       receiver: 
     receiver, method, args, method.registers(), parent: Box::new(self) }
  }
  fn return_register(self, reg: usize) -> Context {
     let return_to = self.parent;
     return_to.receiver = self.registers[reg];
     return_to
  }
}

fn execMethod() {
   let ctx = Contex();
   loop {
      ctx.receiver = 
   loop {
       match ctx.bytecode() {
         Bytecode::sendMessage(message, args) {
            let method = ctx.receiver.method(message);
            ctx = ctx.call(method, args);
            continue;
         }
         Bytecode::returnRegister(reg) {
            ctx = ctx.return_register(reg);
            continue;
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
