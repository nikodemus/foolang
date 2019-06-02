# foolang

(Name is a placeholder.)

Foolang is a Smalltalk-inspired object language that tries to take some
lessons from catenative languages to heart.

## Goals & Beliefs & Suspicions

- Code should be written and read left-to-right.
- The less variables one needs to express oneself clearly, the better.
- Image based development is NOT a good idea. Being able to reproduce
  state is more important than saving it, and source files are actually
  a GOOD idea.
- Smalltalk-style environment is still better than anything other
  languages have to offer: VM and JIT are a must to support interactive
  development, and the environment must be front and center, not an
  afterthought.
- Good support for AOT compilation is needed for good performance: good
  performance means matching -O0 compiled C++ without jumping through
  hoops. This in turn requires type annotations and replacing metaobjects
  with mirrors, and possibly having value types of some sort to avoid
  layers of indirection.
- Messages everywhere is the right idea, but given how Smalltalk needs to
  reserve ifTrue and some other messages for performance reasons, then
  maybe it would be better to just have control structures in the language
  as primitives?
- Multithreading is everywhere. Either you have a solution for doing it
  well or you're irrelevant. Plan: global objects immutable, every thread
  has its own heap so GC pauses are per thread, passing objects between
  threads means copying the object.

## 0.1.0: The Evaluator

_Isn't this just a really bad Smalltalk with effed up syntax?_

Non-Goals:
- Speed
- Completely settled down syntax
- Fancy extensions

TODO:
- [x] AST
- [x] Expression parser
- [x] Expression evaluator
- [x] Use #[] for literal arrays.
- [x] Array ctor [x . y . z]
- [x] Method tables live in a global array, objects refer to it by index.
- [x] Program parser: Class parser
- [x] Program parser: method parser
- [ ] Program loader
- [ ] Method evaluator (handles ^)
- [ ] "comments"
- [ ] Program parser: class-method parser
- [ ] Source formatter
- [ ] Bare bones environment that works directly on files
      - [ ] Session: this is closest thing to an image -- like a notebook
      - [ ] Playground
      - [ ] Browser
      - [ ] Transcript
      - [ ] Inspector
- [ ] The Book

## Later

- 0.2.0: typechecking: AST -> TST
- 0.3.0: bootstrap compiler and VM, threads, coroutines
- 0.4.0: JIT and AOT compilers
- 0.5.0: immutable value types

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

@-prefixes are a workaround for LR conflicts for now.

### Example

```
@class Bar [x y z]

@class-method Bar x: xv y: yv z: zv
    ^self create-instance: [xv . yv . zv]

@method Bar foo: change
    x := x + change

class-method Bar x: xv y: yv z: zv
   ^self create-instance: [xv . yv . zv]

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

  selector<&Foo>: arg
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
