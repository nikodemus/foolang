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

# Dots are used to sequence expressions.
dbConnection reconnect. Stdout print "reconnected!"

# Commas are used to chain messages.
DB connect: "localhost" user: "me" password: "secret",
  query: "select * from customers",
  save: "customers.csv"

# Semicolons are used to cascade messages.
Stdout
  print: "hello";
  print: " ";
  print: "world!";
  newline
```
