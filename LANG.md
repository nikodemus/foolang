# Foo The Language

Everything is an expression that evaluates to an object.

Everything happens by sending messages to objects.

## Quick Tour

```
# Comments are prefixed with hash-sign

# Numbers
123
123.123
0b10101010
0xffffffff

# Strings
"Simple string"
"String with interpolation {user}!"

# Arrays
[evaluated . at . runtime]
$["literal" "constant" "array"]

# Symbols
$literalSymbol

# Variables
let x = 42 # lexical binding
x          # reference
x = 13     # assignment

# Unary message
x sign

# Binary message
x + x

# Keyword message
DB connect: "localhost" user: "me" password: "secret"

# Dots are used to sequence expressions: value of the expression
# before the dot is discarded.
dbConnection reconnect. System stdout print: "reconnected!"

# Commas are used to chain messages: value of the expression
# before the comma becomes the receiver of the message following
# the comma, and so on.
DB connect: "localhost" user: "me" password: "secret",
  query: "select * from customers",
  save: "customers.csv"

# Semicolons are used to cascade messages. Receiver of the first
# message in the cascade (print: "hello" in the example below)
# is used for all messages in the cascade.
System stdout
  print: "hello";
  print: " ";
  print: "world!";
  newline

@class Foo
   "Class for holding three values."
   [_slot1 _slot2 _slot3]

@class-method Foo new
   "Create an instance of Foo with default values.""
   ^self createInstance ["value for slot 1" . "value for slot 2" . "value for slot 2"]

@method Foo slot1: newval
    "Change the value of the first slot."
    _slot1 := newval

@method Foo slot1
    "Read the current value of the first slot."
    ^_slot1
```
