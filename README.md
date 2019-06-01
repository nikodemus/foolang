# foolang

[x] AST
[x] Expression parser
[x] Expression evaluator
[X] Tests
[ ] # Comments
[ ] Change array syntax to #[] and add [] for runtime constructed arrays
[ ] Change local variable syntax to let x := y
[ ] Reserved words: method, class-method, class, constant, let
[ ] Program parser: Class parser
[ ] Program parser: class-method parser
[ ] Method evaluator (handles ^)
[ ] Program parser: Method parser
[ ] Formatter
[ ] Blocks are closures
[ ] Backtrace for errors
[ ] Source locations for expressions
[ ] Self hosting compiler with abstract backend
[ ] Bootstrap backend: compiles methods to rust code (or C?)
[ ] VM: built using the bootstrap compiler
[ ] Bytecode backend: targets the VM

Planned divergences from Smalltalk
- Local type inference (and runtime assertions)
- Conflict-safe extensions to third-party classes
- Package system
- Methods implemented in terms of blocks
- Blocks implement: value, value::::, and apply:
  b value            # unary
  b value: 1         # keyword
  b value: 1 : 2 : 3 # keyword
  b apply: array

## Declarative Syntax

### Example

```
class Bar [x y z] # This is actually not consistent with using [x . y . z]
                  # for runtime allocated arrays... but I don't think
                  # I care.

class-method Bar x: xval y: yval
    ^self create-instance: [x . y] # This is nice because the vector can just be wrapped in the class.

method Bar foo: change
    x := x + change

phrase ring!
    ding ding ding

constant PI := 3.14

import Quux
import foopkg.Foo as: Foofoo

```

### Grammar

```
program := program-element*

program-element := class | instance-method | class-method | phrase

class := "class" Identifier "[" identifier* "]"

instance-method := "method" Identifier method-pattern method-body

class-method := "class-method" Identifier method-pattern method-body

phrase := "phrase" identifier! message-chain

constant := "constant" IDENTIFIER ":=" expression
```

## Parts


Syntax work:
[ ] rework string and character syntax
[ ] => {}
[ ] Positional/variable arguments:
    Array of: 1 : 2 : 3
      Array addMethod: #of: { :(args*) | ^args toArray }
    { :(a b) | a + b } : 1 : 2
[ ] String interpolation #"This is {self name}!"
[ ] Message chaining with ,
[ ] Unary minus and negation  -foo ~foo
[ ] dict syntax { foo: x signum.
                  quux: y.
                  zot: z. }
[ ] Indexing foo[x] and foo[x][y]
[ ] Indexing assignment foo[x] = 42
[ ] Operator precedence
    0. Attached (unary) - and ~
    1. Attached ^
    2. * / // %
    3. + -
    4. << >>
    5. & |
    6. == < > =< >=
    7. &&
    8. ||

### Bootstrap Compiler

Written in Rust.

Goal: specify VM core in foolang:

    "newContext: newMethod on: newSelf
        stack push: context
        context = Context new: newMethod on: newSelf with: context args"

    "invoke: method
        self newContext: method on: receiver"

- Generates code for methods given class description, method source,
  and backend.
- Initial backend is C.
- Supports:
  - self slot references and assignments
  - argument references
  - messages
  - constants
  - simple blocks (no closures)
- Quality of generated code unimportant, as long as it works.

### GC

Steal from Jazzlang, but give up on the double-wide stuff. Double-word
allocations.

Allocation header:
  Bits 00-01: GC marks
  Bits 02-09: number of raw words
  Bits 10-17: number of gc slots
  Bits 18-25: number of weak slots
  Bits 26-28: no tail / raw tail / weak tail / pointer tail
  Bits 29-31: n^2 = tail element width in bytes if raw tail

## Specificationish

Smalltalk syntax, except:

- Comma used for message chaining instead of parentheses.
- Blocks use {}
- Blocks can have implicit argument _.
- x => { ... } desugars into { ... } value: x

## Motivating Examples

### Example 1

Using blocks:

    Backend select: #postgres => {
      _ connect: "localhost" => {
          _ query: "select * from users", do: { "User: {_ name}" print }.
          _ query: "select * from suppliers", do: { "Vendor: {_ name}" print }.
      }.
      _ connect: "remote" => {
          _ query: "select * from users", do: { "User: {_ name}" print }.
          _ query: "select * from suppliers", do: { "Vendor: {_ name}" print }.
      }.
    }.

Factored using phrases:

    define -print-names-in-table: table as: pretty {
      _ query: "select * from {table}", do: { "{pretty}: {_ name}" print }
    }

    define -print-main-tables {
      _ -print-names-in-table: "users" as: "User".
      _ -print-names-in-table: "suppliers" as: "Vendor".
    }

    Backend select: #postgres => {
      _ connect: "localhost" -print-main-tables.
      _ connect: "remote" -print-main-tables.
    }.

## References

- Smalltalk-80: The Language and Its Implementation
  http://www.mirandabanda.org/bluebook/bluebook_imp_toc.html

## BIG GOAL

class Ackermann
  m: m<i64> n: n<i64> ^<i64>
    m == 0 then: {
      ^n + 1
    }.
    n == 0 then: {
      ^Ackermann m: m - 1 n: 1
    }.
    ^ Ackermann m: m - 1 n: (Ackermann m: m n: n - 1)

This should compile into decent native code. Something close enough
to what gcc -O0 would produce.

## MMmmmaaaybe

- Annotation assisted escape analysis:

  selector: arg <&Foo>
    arg stuff

  Check that value of arg cannot escape: cannot be passed
  to unknown methods, or as arguments not marked as &Foo.
  If this is true then the object can be stack-allocated automatically.

- Serialize classes using json:

  { "class": "Class",
    "name": "Point",
    "slots": ["x", "y"],
    "methods": [{ "type": "method", "selector": "foo:with:",
                  "args": ["bar", "quux"],
                  "body": "bar ding: quux, reportTo: self"}]}
