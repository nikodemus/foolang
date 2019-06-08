# foolang

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

Timeboxing in releases of 50-200h of work.

### WIP: 0.1.0: The Evaluator

_Isn't this just a bad Smalltalk without any dev environment?_

**Goals**: syntax v1, working evaluator, threads.

**Non-goals**: performance, fancy extensions, useful class library.

Time spent: 72h
Estimated remaining: 44h

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
- Program loader
      - [x] self
      - [x] createInstance. 5h
      - [x] instance variables. 5h
- [x] Method evaluator (handles ^ in method bodies)
- [X] ~~return in an expression context "just returns". Est 5h.~~
- [x] "comments" (preserved in the AST and methods, returned using help: #selector) Est 5h.
- [x] Blocks are closures (variables). 5h.
- [x] Blocks are closures (return). 5h
- [x] Local variables in methods. 5h
- Terminal REPL
  - [x] REPL implemented in foolang.
  - [ ] Input stdin. 1h
  - [ ] Input readLine. 1h
  - [ ] Input stdout. 1h
  - [ ] Output print: 1h
  - [ ] Output flush 1h
  - [ ] Output newline 1h
  - [ ] Foolang compiler 1h
  - [ ] Compiler tryParse 1h
  - [ ] Compiler evaluate 1h
  - [ ] String new 1h
  - [ ] String append 1h
  - [ ] String clear 1h
  - [ ] Block repeat 1h
  - [ ] Block whileFalse 1h
- [ ] Source file execution. Est 5h.
- [ ] Threads. Est 10h.
- [ ] Benchmarks: foolang, pharo, python, SBCL, clang. Est 5h.
- [ ] Read Pharo by Example. 10h
- Optional extras:
  - [ ] String interpolation '1 + 1 = {1 + 1}'
  - [ ] => { :that | ... }
  - [ ] Message chaining with ,
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
  - [ ] Phrases
  - [ ] Rework string syntax: raw, interpolated, block
  - [ ] # Comments
  - [ ] Receiver stack (method local!)
  - [ ] Prefix messages: -foo, !foo, ~foo
  - [ ] Indexing foo[x] as sugar for at:
  - [ ] Indexing assignment foo[x] := 42 as sugar for at:put:

### Planned: 0.2.0: The IDE

_...a bad SmallTalk with an environment from the 80s?_

**Goals**: a working Smalltalk-like IDE using Mirrors, actually writing foolang
code.

**Non-goals**: performance, fancy extensions, useful class library.

Estimated remaining: 110h

- [ ] Source formatter. Est 10h.
- [ ] Wrap a rust web server as foolang object. 5h
      Actix bind: "/foo" to: { |request| ... }
- Bare bones class browser widget:
  - [ ] List classes and methods. 5h
  - [ ] Adding and editing classes. 5h
  - [ ] Adding and editing methods. 5h.
- Make the editor components nice
  - [ ] Generate an overlay: highlight classes and errors stuff. 5h
  - [ ] Offer completions for classes and selectors. 5h
  - [ ] Keyboard shortcuts. 5h
- Add a persistent playground widget. 5h
- Add a persistent transcript widget. 5h
- Add a finder (opens browser) widget. 5h
- Add an inspector widget. 10h
- Add debugger widget. 20h
- Integrate these components into a nice whole. 20h

### Planned: 0.3.0: The Virtual Machine

_Ok, kind of impressive you're doing all this, but why are you doing this?_

**Goals**: self-hosted VM, ability to extend the environment in foolang.

**Non-goals**: performance, fancy extensions, useful class library.

Estimated remaining: 150h.

- [ ] Self-hosted AST -> IR compiler. Est 10h
- [ ] Self-hosted AST -> javascript compiler. Est 10h
- [ ] Self-hosted IR -> bytecode compiler. Est 10h
- [ ] Self-hosted IR -> Rust compiler. Est 40h. (subsetted for VM & GC implementation)
- [ ] Self-hosted VM. Est. 40h.
- [ ] Self-hosted GC. Est. 40h

### Planned: 0.4.0: The Compiler

_Ok. I can kinda see the point now!_

**Goals**: matching -O0 C++ performance for simple benchmarks with native AOT.
Beating python with JIT. Ability to deliver standalone applications and
precompiled libraries. First public release.

**Non-goals**: sophisticated type system, fancy extensions, useful class
library.

Estimated remaining: 130h

- [ ] Proper name instead of foolang. Est 5h.
- [ ] Bytecode -> IR decompiler. Est 5h.
- [ ] IR JIT compiler. Est 40h.
- [ ] Simple static type annotations with dynamic checks. Est 10h.
- [ ] IR optimization to elide dispatch and typechecks in trivial cases. Est 10h.
- [ ] IR inlining. Est 10h.
- [ ] IR AOT compiler (platform specific OK). Est 10h.
- [ ] The Book. Est 20h.
- [ ] Examples + livestreaming Est 20h.

### Planned: 0.5.0: The Type System

_This is getting interesting!_

**Goals**: enough fancy features to match -O1 C++ performance for harder
cases. Ability to controllably extend builtin classes.

**Non-goals**: useful class-library.

Estimated remaining: 110h

- [ ] Value types. Est 40h.
- [ ] Parameteric classes. Est 40h.
- [ ] Class extensions. Est 20h.
- [ ] Module system. Est 10h.

### Planned: 0.6.0: Useful Class Library

_What are people using this for? Can you share some success stories?_

**Goals**: a "batteries included" release, or at least the infrastructure for one.

Estimated remaining: 120h

- [ ] Libraries. Est 40h.
- [ ] Library system (ie. cargo-lookalike). Est 40h.
- [ ] Tooling for wrapping Rust code. Est 40h

### Planned: 0.7.0: Fix What's Broken

_I'm using this at work to do X!_

**Goals**: After some use by other people than me, identify and fix
the main issues likely to cause future problems.

### Planned: 1.0.0: Stable Release

_When's the next conference?_

**Goals**: No foreseeable need for breaking changes. (Of course they
will come!)

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

## Refactoring Notes

// Mutex needed because lazy_static! requires Sync, and
// closures require mutability.
type Bindings =  HashMap<Identifier,Mutex<Object>>
// Rc needed because entire lexenv is captured by Closures,
// which makes lifetimes unpredictable.
type Lexenv = Rc<LexenvFrame>

  env.eval(&Expr) -> Object

  // env.rs
  trait Env {
    fn eval(&self, &Expr) -> Object
    fn extend(&self, &Bindings) -> Lexenv {

    }
  }

  impl Env for Lexenv {

  }

  impl Env for GlobalEnv {

  }

  // globalenv.rs
  struct GlobalEnv {

  }

  impl GlobalEnv {

  }

  // lexenv.rs
  struct LexenvFrame {
    bindings: Bindings,
    parent: Lexenv
  }

  impl Lexenv {

  }

Need to move things out of
GlobalEnv is ok
Not 100% sure about separate Lexenv and GlobalEnv, but it doesn't seem like a real issue.


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

- Various browser editor components
  - Rich text: https://quilljs.com/guides/why-quill/
  - Code editor: https://ace.c9.io/
  - Code editor: https://codemirror.net/doc/manual.html
  - Code editor: https://github.com/Microsoft/monaco-editor
  - Code editor: https://icecoder.net/
  - Drawing: http://literallycanvas.com/

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

- Stdout problem solution:

  1. I cannot fully fix this given that users can create their own
     equivalents by using the FD directly. So I should not try.

  2. So there is no reason not to have a System stdout method which
     returns the stream. That is not the nice way to use it, though,
     just the medium layer.

  3. For niceness have actor wrapping the object

        @actor Stdout = System stdout

  4. ...initially I can just use System stdout.

- Considering how I'm planning to use files in but have a class browser
  as the editing environment... maybe I should go "full Java" and have

     pkgname.foopkg/
        classname/
           _classname_.fooclass
           methodname1.foo
           methodname2.foo

  That way there is no need to figure out which bit needs rewriting,
  and thing remain sane-ish to browse too.

  ...though names will have to be mangled, since : isn't allowed on
  windows. So I will have to forbid either - or _ in selectors to use
  for ":"

  ...and I need to be case-insensitive?! Well, forbidding multiple
  selectors that only differ in capitalization in the same class doesn't
  sound too bad.

  __ prefix for class
  _ prefix for class methods
  _ in place of :
  ...allow dashing-names?

  ```
    core.foopkg/Box/__Box.fooclass
                    _new.foomethod
                    value.foomethod
                    value_.foomethod
  ```

  Then _allow_ human written .foo files which mix everything.

- Being able to load "regular smalltalk code" might a big timesaver...

- Finding using examples is amazing! The reason it isn't _that_ dangerous
  is that you specify the object.

  ...but still, doing Directory("/") . "/" spinning through deleteTree
  gives me the creeps. MAYBE if it trapped in a system object?

- Annotation assisted escape analysis:

  selector: &arg
    arg stuff

  Check that value of arg cannot escape: cannot be stored,
  cannot be passed to as non-& arguments. If this is true
  then the object can be stack-allocated safely.
