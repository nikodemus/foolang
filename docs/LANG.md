# Foo The Language

Programs consist of definitions including a main program.

Definitions are:
- Constants
- Classes
- Class and instance methods

Main program consists of a binding for the system object
and expressions to evaluate.

Expressions are either:
- Literals
- Constant references
- Variable bindings
- Variable references
- Variable assignments
- Message sends
- Blocks
- Returns

## Syntax

### Comments

```
# This is a comment.
```

### Integer Literals

_Decimal numbers_
```
123
```

_Hexadecimal numbers_
```
0xFFFFFFFF
```

_Binary numbers_
```
0b01010101
```

All integer literals additionally allow interleaving of underscore
characters to make magnitude of large numbers easier to see. 

```
100_000_000

0xFFFF_FFFF_FFFF_FFFF

0b0101_0101_0101_0101
```

# Float literals

```
1.123
123123
```
$"This is a literal string: {} does nothing here."

$"""This is very literal block string."""

\#['literal' 'constant' 'array']
\#literalSymbol
```

```
"This is a string that could be interpolated but is not."
"""This is a block string that could be interpolated but
   is not.

   Leading whitespace upto the indentation of the first
   line is removed.

   "Double quotes" can be embedded without escaping."""
```

### Array constructors

Where every evaluation of an array literal returns the same array
object, the array constructor expression creates a new array every
time.

```
[ constructed . at . runtime ]
```

### Messages

In the following examples both the parenthesized and unparenthesized
expressions mean the same thing.

*Unary Messages*

Unary message is a single word. Unary messages have the highest
precedence.

```
object messageOneToObject messageTwoToResultOfMessageOne
```

*Binary Messages*

Binary messages are a symbol followed by an expression. Binary
messages have the second highest precedence, but have no internal
precedence ordering.

```
x + y + z * multipliesXplusYplusZ
resultOfSinTOBeAdded + x sin
```

*Keyword Messages*

Keyword messages are a sequence of ``keyword: value`` pairs. One such sequence
is a single message. Keyword messages have the lowest precedence.

```
DB connect: "localhost" user: "me" password: secrets decodePassword
```

### Blocks

First significant divergence from Smalltalk syntax: blocks are written using
curly braces.

```
{ foo + 1 } value
```

WIP MARKER

### Variables

Variable bindings



{ | x y y | ...
let x = 42 # lexical binding
x          # reference
x = 13     # assignment

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
