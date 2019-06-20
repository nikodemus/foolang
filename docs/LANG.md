# Foo The Language

Semantically Foolang follows in the footsteps of Self, in that
programs consist of nothing objects sending messages to each other.

Except whereas in Self that is really true, Foolang backs away
from the corner a bit: variables and returns are not first class
values explained in terms of objects.

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

### Float Literals

Currently all floats are double-floats.

```
1.123
1.0e6
```

### String Literals

```
$"Simple string literal. Newlines can be embedded as literals.
"Doublequotes" are fine, Escape sequences are ignored. To
embed a doublequote followed by a dollar sign use "$$ instead."$

$"""Block string literal.
   Whitespace upto start column of string proper is stripped.
       Further whitespace is preserved.
   Escape sequences like \n are ignored.
   "Double quotes" can be used without escaping. To embed
   a 3 x doublequote followed by a dollar sign use """$$ instead."""$
```

### String Interpolation

If string interpolation syntax is used without embedding `{}` in the string
the resulting object is a literal string.

```
"Hello {user name}! Escape sequences like \n do work.
Newlines can be embedded too. \" is required to embed a
doublequote, including the interpolated parts."

"""Block string with interpolation: {"doublequotes" append " are fine!"}
   Whitespace upto start column of string proper is stripped.
       Further whitespace is preserved.
   Escape sequences like \n work too."""
```

### Selector Literals

Currently parser translates supported operators into selectors with alphanumeric
names. The plan is to allow user extensions along the same lines.

```
$unarySelector

$key:word:selector:

$threeArgSelector:::
```

### Array Literals

```
$["literal", "constant", "array"]
```

### Array Constructor

```
[array, constructed, at, runtime]
```

### Type Annotations

Annotations can be applied to bindings, values, instance variables,
and method signatures.

Annotations that the compiler cannot prove are asserted at runtime.

```
let x <Int> := 42

foo bar <Int> + 1

@class Foo { bar <Int> }

@method Foo bar: x <Int> -> <Int>
   bar + x
```

### Messages

*Unary Messages*

Unary message is a single word or a prefix symbol. Unary messages have the highest
precedence.

```
object messageOneToObject messageTwoToResultOfMessageOne

-x
```

XXX HERE XXX

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
