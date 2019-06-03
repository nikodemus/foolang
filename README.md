# foolang

(Name is a placeholder.)

Foolang is a Smalltalk-inspired object language that tries to take some
lessons from catenative languages to heart.

## Goals & Beliefs & Suspicions

- Code should be written and read left-to-right.
- The less variables one needs to express oneself clearly, the better.
- Image based development is not a good idea: focusing on reproduction of
  state is better than saving state.
- Stateful notebooks are a good idea, but as an exploration tool, not as
  a development environment.
- Smalltalk-style environment is still better than anything other
  languages have to offer: interactive development and tools
  like browser and inspector and transcript and... are a must. They
  need to be front and center, not an afterthought. (Supporting people's
  favorite editor is not important, as long as they don't need to buy into
  the infrastructure for their first Hello-World.)
- Predictably good performance is a must. AOT compilation is needed for that.
  Predictably good performance means matching -O0 compiled C++ without jumping
  through hoops. This in turn requires type annotations and replacing metaobjects
  with mirrors, and possibly having value types of some sort to avoid extra
  layers of indirection.
- Messages everywhere is the right idea, but given how Smalltalk needs to
  reserve ifTrue and some other messages for performance reasons, then
  maybe it would be better to just have control structures in the language
  as primitives?
- Multithreading is everywhere. Either you have a solution for doing it
  well or you're irrelevant. Plan: global objects immutable, every thread
  has its own heap so GC pauses are per thread, passing objects between
  threads means copying the object.

## Project Plan

### WIP: 0.1.0: The Evaluator

_Isn't this just a really bad Smalltalk with effed up syntax?_

**Goals**: syntax v1, working evaluator, threads.

**Non-goals**: performance, fancy extensions, useful class library.

- [x] AST
- [x] Expression parser
- [x] Expression evaluator
- [x] Use #[] for literal arrays.
- [x] Array ctor [x . y . z]
- [x] Method tables live in a global array, objects refer to it by index.
- [x] Program parser: Class parser
- [x] Program parser: method parser
- [x] Program parser: class-method parser
- [x] Explicit representation for the global environment
- [ ] Program loader
      - [ ] self
      - [ ] createInstance
      - [ ] instance variables
      Both self and instance variables require that eval_in_env knows
      the current receiver. I think this is the perfect opportunity to
      turn lexenv into context that holds also self and a 'still-valid'
      refcell: then returns can (1) check that they're still value, and
      (2) unwind until the method holding the right refcell.
- [x] Method evaluator (handles ^ in method bodies)
- [ ] "comments" (preserved in the AST and methods, returned using help: #selector)
- [ ] Source formatter
- [ ] Terminal playground
- [ ] Source file execution
- [ ] Threads
- Optional extras:
  - [ ] String interpolation '1 + 1 = {1 + 1}'
  - [ ] Phrases
  - [ ] # Comments
  - [ ] Receiver stack (method local!)
  - [ ] => { :that | ... }
  - [ ] Message chaining with ,
  - [ ] Prefix messages: -foo, !foo, ~foo
  - [ ] Record syntax [ foo: x signum. quux: y. ] and #[foo: 42]
  - [ ] Positional arguments in keyword messages:
        Blocks implement: value, value::::, and apply:
        b value            # unary
        b value: 1         # keyword
        b value: 1 : 2 : 3 # keyword
        b apply: array
  - [ ] Operator precedence for binary messages
        0. Attached (unary) - and ~
        1. Attached ^ ie, 2^x
        2. * / // %
        3. + -
        4. << >>
        5. & |
        6. == < > =< >=
        7. &&
        8. ||
   - [ ] Indexing foo[x] as sugar for at:
   - [ ] Indexing assignment foo[x] := 42 as sugar for at:put:
   
### Planned: 0.2.0: The Virtual Machine

**Goals**: self-hosted VM, getting to write code in foolang to figure out what works.

**Non-goals**: performance, fancy extensions, useful class library.

- [ ] Self-hosted AST -> IR compiler (arbitrary code)
- [ ] Self-hosted IR -> bytecode compiler (arbitrary code)
- [ ] Self-hosted IR -> Rust compiler (subsetted for VM implementation)
- [ ] Self-hosted VM

### Planned: 0.3.0: The IDE

**Goals**: a working Smalltalk-like IDE using Mirrors, one that I don't mind using.

**Non-goals**: performance beyond "IDE is usable", fancy extensions, useful class library.

- [ ] Session: this is closest thing to an image -- like a notebook
- [ ] Playground
- [ ] Browser
- [ ] Transcript
- [ ] Inspector
- [ ] Editor

### Planned: 0.4.0: The Compiler

**Goals**: matching -O0 C++ performance for simple benchmarks with native AOT. Beating
python with JIT. Ability to deliver standalone applications and precompiled libraries.

**Non-goals**: sophisticated type system, fancy extensions, useful class library.

- [ ] Bytecode -> IR decompiler
- [ ] IR JIT compiler
- [ ] Simple static type annotations with dynamic checks
- [ ] IR optimization to elide dispatch and typechecks
- [ ] IR inlining
- [ ] IR AOT compiler

### Planned: 0.5.0: The Type System

**Goals**: enough fancy features to match -O1 C++ performance for harder
cases. Ability to controllably extend builtin classes.

**Non-goals**: useful class-library. 

- [ ] Value types
- [ ] Parameteric classes
- [ ] Class extensions
- [ ] Module system

### Planned: 1.0.0: Useful Class Library

- [ ] The Book
- [ ] Examples
- [ ] Libraries
- [ ] Library system (ie. cargo-lookalike)
- [ ] Tooling for wrapping Rust code.

## Declarative Syntax

@-prefixes are a workaround for LR conflicts for now.

### Example

```
@class Bar [x y z]

@class-method Bar x: xv y: yv z: zv
    ^self create-instance: [xv . yv . zv]

@method Bar foo: change
    x := x + change

@phrase ring!
    "No unbound variables allowed! Constants are OK, though."
    ding ding ding

@constant PI := 3.14

@import Quux
@import foopkg.Foo as: Foofoo

```

## Parts

### Bootstrap Compiler

Written in Rust.

Goal: specify VM core in foolang:

    "newContext: newMethod on: newSelf
        stack push: context
        context = Context new: newMethod on: newSelf with: context args"

    "invoke: method
        self newContext: method on: receiver"

Quality of generated code unimportant, as long as it works.

### GC

Simplest thing that could possibly work: mark and sweep on top of malloc and
free.

Allocation header:
  Bits 00-01: GC marks
  Bits 02-09: number of raw words
  Bits 10-17: number of gc slots
  Bits 18-25: number of weak slots
  Bits 26-28: no tail / raw tail / weak tail / pointer tail
  Bits 29-31: n^2 = tail element width in bytes if raw tail

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

  selector: &arg
    arg stuff

  Check that value of arg cannot escape: cannot be stored,
  cannot be passed to as non-& arguments. If this is true
  then the object can be stack-allocated safely.
