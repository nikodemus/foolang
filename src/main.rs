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

struct Method {
  name: String,
  code: Vec<Bytecode>,
  constants: Vec<Object>,
}

struct Class {
  name: String,
  methods: HashMap<String,Method>,
}

struct Object {
  class: Class,
  slots: Vec<Object>,
}

// Opcode formats:
//
// [opcode6]00
// [opcode6]01[arg8]
// [opcode6]10[arg8][arg8]
const OPCODES: [OpImpl, 256] = [
    // [opcode8]
    op_self_arg,        // Opcode 0
    op_self_receiver,   // Opcode 1
    op_return,          // Opcode 2
    op_receiver_arg,    // Opcode 3
    op_unimplemented,   // Opcode 4
    op_unimplemented,   // Opcode 5
    // [opcode8][arg8]
    op_const_arg,        // Opcode 6
    op_reg_arg,          // Opcode 7
    op_slot_arg,         // Opcode 8
    op_const_receiver,   // Opcode 9
    op_reg_receiver,     // Opcode 10
    op_slot_receiver,    // Opcode 11
    op_send_message,     // Opcode 12
    op_self_to_reg,      // Opcode 13
    op_receiver_to_reg,  // Opcode 14
    op_self_to_slot,     // Opcode 15
    op_receiver_to_slot, // Opcode 16
    op_unimplemented,    // Opcode 17
    op_unimplemented,    // Opcode 18
    op_unimplemented,    // Opcode 19
    // [opcode8][arg8][arg8]
    op_const_to_reg,     // Opcode 20
    op_slot_to_reg,      // Opcode 21
    op_reg_to_reg,       // Opcode 22
    op_const_to_slot,    // Opcode 23
    op_slot_to_slot,     // Opcode 24
    op_reg_to_slot,      // Opcode 25
    op_unimplemented, // Opcode 26
    op_unimplemented, // Opcode 27
    op_unimplemented, // Opcode 28
    op_unimplemented, // Opcode 29
    op_unimplemented, // Opcode 30
    op_unimplemented, // Opcode 31
    op_unimplemented, // Opcode 32
    op_unimplemented, // Opcode 33
    op_unimplemented, // Opcode 34
    op_unimplemented, // Opcode 35
    op_unimplemented, // Opcode 36
    op_unimplemented, // Opcode 37
    op_unimplemented, // Opcode 38
    op_unimplemented, // Opcode 39
    op_unimplemented, // Opcode 40
    op_unimplemented, // Opcode 41
    op_unimplemented, // Opcode 42
    op_unimplemented, // Opcode 43
    op_unimplemented, // Opcode 44
    op_unimplemented, // Opcode 45
    op_unimplemented, // Opcode 46
    op_unimplemented, // Opcode 47
    op_unimplemented, // Opcode 48
    op_unimplemented, // Opcode 49
    op_unimplemented, // Opcode 50
    op_unimplemented, // Opcode 51
    op_unimplemented, // Opcode 52
    op_unimplemented, // Opcode 53
    op_unimplemented, // Opcode 54
    op_unimplemented, // Opcode 55
    op_unimplemented, // Opcode 56
    op_unimplemented, // Opcode 57
    op_unimplemented, // Opcode 58
    op_unimplemented, // Opcode 59
    op_unimplemented, // Opcode 60
    op_unimplemented, // Opcode 61
    op_unimplemented, // Opcode 62
    op_unimplemented, // Opcode 63
    op_unimplemented, // Opcode 64
    op_unimplemented, // Opcode 65
    op_unimplemented, // Opcode 66
    op_unimplemented, // Opcode 67
    op_unimplemented, // Opcode 68
    op_unimplemented, // Opcode 69
    op_unimplemented, // Opcode 70
    op_unimplemented, // Opcode 71
    op_unimplemented, // Opcode 72
    op_unimplemented, // Opcode 73
    op_unimplemented, // Opcode 74
    op_unimplemented, // Opcode 75
    op_unimplemented, // Opcode 76
    op_unimplemented, // Opcode 77
    op_unimplemented, // Opcode 78
    op_unimplemented, // Opcode 79
    op_unimplemented, // Opcode 80
    op_unimplemented, // Opcode 81
    op_unimplemented, // Opcode 82
    op_unimplemented, // Opcode 83
    op_unimplemented, // Opcode 84
    op_unimplemented, // Opcode 85
    op_unimplemented, // Opcode 86
    op_unimplemented, // Opcode 87
    op_unimplemented, // Opcode 88
    op_unimplemented, // Opcode 89
    op_unimplemented, // Opcode 90
    op_unimplemented, // Opcode 91
    op_unimplemented, // Opcode 92
    op_unimplemented, // Opcode 93
    op_unimplemented, // Opcode 94
    op_unimplemented, // Opcode 95
    op_unimplemented, // Opcode 96
    op_unimplemented, // Opcode 97
    op_unimplemented, // Opcode 98
    op_unimplemented, // Opcode 99
    op_unimplemented, // Opcode 100
    op_unimplemented, // Opcode 101
    op_unimplemented, // Opcode 102
    op_unimplemented, // Opcode 103
    op_unimplemented, // Opcode 104
    op_unimplemented, // Opcode 105
    op_unimplemented, // Opcode 106
    op_unimplemented, // Opcode 107
    op_unimplemented, // Opcode 108
    op_unimplemented, // Opcode 109
    op_unimplemented, // Opcode 110
    op_unimplemented, // Opcode 111
    op_unimplemented, // Opcode 112
    op_unimplemented, // Opcode 113
    op_unimplemented, // Opcode 114
    op_unimplemented, // Opcode 115
    op_unimplemented, // Opcode 116
    op_unimplemented, // Opcode 117
    op_unimplemented, // Opcode 118
    op_unimplemented, // Opcode 119
    op_unimplemented, // Opcode 120
    op_unimplemented, // Opcode 121
    op_unimplemented, // Opcode 122
    op_unimplemented, // Opcode 123
    op_unimplemented, // Opcode 124
    op_unimplemented, // Opcode 125
    op_unimplemented, // Opcode 126
    op_unimplemented, // Opcode 127
    op_unimplemented, // Opcode 128
    op_unimplemented, // Opcode 129
    op_unimplemented, // Opcode 130
    op_unimplemented, // Opcode 131
    op_unimplemented, // Opcode 132
    op_unimplemented, // Opcode 133
    op_unimplemented, // Opcode 134
    op_unimplemented, // Opcode 135
    op_unimplemented, // Opcode 136
    op_unimplemented, // Opcode 137
    op_unimplemented, // Opcode 138
    op_unimplemented, // Opcode 139
    op_unimplemented, // Opcode 140
    op_unimplemented, // Opcode 141
    op_unimplemented, // Opcode 142
    op_unimplemented, // Opcode 143
    op_unimplemented, // Opcode 144
    op_unimplemented, // Opcode 145
    op_unimplemented, // Opcode 146
    op_unimplemented, // Opcode 147
    op_unimplemented, // Opcode 148
    op_unimplemented, // Opcode 149
    op_unimplemented, // Opcode 150
    op_unimplemented, // Opcode 151
    op_unimplemented, // Opcode 152
    op_unimplemented, // Opcode 153
    op_unimplemented, // Opcode 154
    op_unimplemented, // Opcode 155
    op_unimplemented, // Opcode 156
    op_unimplemented, // Opcode 157
    op_unimplemented, // Opcode 158
    op_unimplemented, // Opcode 159
    op_unimplemented, // Opcode 160
    op_unimplemented, // Opcode 161
    op_unimplemented, // Opcode 162
    op_unimplemented, // Opcode 163
    op_unimplemented, // Opcode 164
    op_unimplemented, // Opcode 165
    op_unimplemented, // Opcode 166
    op_unimplemented, // Opcode 167
    op_unimplemented, // Opcode 168
    op_unimplemented, // Opcode 169
    op_unimplemented, // Opcode 170
    op_unimplemented, // Opcode 171
    op_unimplemented, // Opcode 172
    op_unimplemented, // Opcode 173
    op_unimplemented, // Opcode 174
    op_unimplemented, // Opcode 175
    op_unimplemented, // Opcode 176
    op_unimplemented, // Opcode 177
    op_unimplemented, // Opcode 178
    op_unimplemented, // Opcode 179
    op_unimplemented, // Opcode 180
    op_unimplemented, // Opcode 181
    op_unimplemented, // Opcode 182
    op_unimplemented, // Opcode 183
    op_unimplemented, // Opcode 184
    op_unimplemented, // Opcode 185
    op_unimplemented, // Opcode 186
    op_unimplemented, // Opcode 187
    op_unimplemented, // Opcode 188
    op_unimplemented, // Opcode 189
    op_unimplemented, // Opcode 190
    op_unimplemented, // Opcode 191
    op_unimplemented, // Opcode 192
    op_unimplemented, // Opcode 193
    op_unimplemented, // Opcode 194
    op_unimplemented, // Opcode 195
    op_unimplemented, // Opcode 196
    op_unimplemented, // Opcode 197
    op_unimplemented, // Opcode 198
    op_unimplemented, // Opcode 199
    op_unimplemented, // Opcode 200
    op_unimplemented, // Opcode 201
    op_unimplemented, // Opcode 202
    op_unimplemented, // Opcode 203
    op_unimplemented, // Opcode 204
    op_unimplemented, // Opcode 205
    op_unimplemented, // Opcode 206
    op_unimplemented, // Opcode 207
    op_unimplemented, // Opcode 208
    op_unimplemented, // Opcode 209
    op_unimplemented, // Opcode 210
    op_unimplemented, // Opcode 211
    op_unimplemented, // Opcode 212
    op_unimplemented, // Opcode 213
    op_unimplemented, // Opcode 214
    op_unimplemented, // Opcode 215
    op_unimplemented, // Opcode 216
    op_unimplemented, // Opcode 217
    op_unimplemented, // Opcode 218
    op_unimplemented, // Opcode 219
    op_unimplemented, // Opcode 220
    op_unimplemented, // Opcode 221
    op_unimplemented, // Opcode 222
    op_unimplemented, // Opcode 223
    op_unimplemented, // Opcode 224
    op_unimplemented, // Opcode 225
    op_unimplemented, // Opcode 226
    op_unimplemented, // Opcode 227
    op_unimplemented, // Opcode 228
    op_unimplemented, // Opcode 229
    op_unimplemented, // Opcode 230
    op_unimplemented, // Opcode 231
    op_unimplemented, // Opcode 232
    op_unimplemented, // Opcode 233
    op_unimplemented, // Opcode 234
    op_unimplemented, // Opcode 235
    op_unimplemented, // Opcode 236
    op_unimplemented, // Opcode 237
    op_unimplemented, // Opcode 238
    op_unimplemented, // Opcode 239
    op_unimplemented, // Opcode 240
    op_unimplemented, // Opcode 241
    op_unimplemented, // Opcode 242
    op_unimplemented, // Opcode 243
    op_unimplemented, // Opcode 244
    op_unimplemented, // Opcode 245
    op_unimplemented, // Opcode 246
    op_unimplemented, // Opcode 247
    op_unimplemented, // Opcode 248
    op_unimplemented, // Opcode 249
    op_unimplemented, // Opcode 250
    op_unimplemented, // Opcode 251
    op_unimplemented, // Opcode 252
    op_unimplemented, // Opcode 253
    op_unimplemented, // Opcode 254
    op_unimplemented, // Opcode 255
];




enum Bytecode {
   // Passing arguments
   // Setting up receiver
   // Sending the message
   // Returning current receiver
   // Moves
   ConstRegister(u8)
   // Primitives
   PrintObject
   Add
   
}

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
