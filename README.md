[![Build Status](https://dev.azure.com/nikodemus0619/foolang/_apis/build/status/nikodemus.foolang?branchName=master)](https://dev.azure.com/nikodemus0619/foolang/_build/latest?definitionId=1&branchName=master)

# foolang

Foolang is a Smalltalk-inspired object language that tries to take some
lessons from other languages to heart.

## Pleasure to Write, Pleasure to Read

Main purpose of a programming language is expression and communication of
ideas. If it fails in this nothing else matters.

To my mind this implies certain things:

- Syntax should be minimal.
- The closer the reading order matches the execution order the less
  context one has to keep in mind.
- One should be able to read code out loud easily.
- The less named variables one needs to express oneself clearly the better.
  Names do add clarity, and they are in general desirable -- but they
  also impose a cognitive cost and are a barrier to refactoring.

## Real Computers All The Way Down

Alan Kay is full of wisdom. Here's one thing he said that I keep thinking about:

>  To me, one of the nice things about the semantics of real objects is that
> they are "real computers all the way down (RCATWD)" -- this always retains
> the full ability to represent anything. The old way quickly gets to two
> things that aren't computers -- data and procedures -- and all of a sudden
> the ability to defer optimizations and particular decisions in favour of
> behaviours has been lost. In other words, always having real objects always
> retains the ability to simulate anything you want, and to send it around the
> planet."

It took me a long time to appreciate this fully. The moment you start thinking
about invoking methods instead of sending messages you're on the slippery slope
that will lead you to eventually taking references to methods...

Of course, Turing tells us the all computation is equivalent, but if that was
the _whole_ truth we'd still be programming in binary.

It's not about power. It's about ease and it's about how the language influences
your thinking.

## Efficiency

Efficiency is a property of an implementation, not language, but since Foo is
a one-person show at the moment, the question is: "Can I implement this
efficiently now, with reasonable effort?"

Success criteria here is matching speed of unoptimized C/C++ for basic
benchmarks like recursive algorithms, numerics, and instance variable access while
not being completely embarrassed by memory management overhead.

Arbitrary Foolang code need not hit these targets: only code that has been
written with efficiency in mind, and accepts C-like constraints on it, such
as static types and modular arithmetic.

## Goals & Beliefs & Suspicions

- Messages instead of method invocations is the right metaphor.
- Code should be read and written left-to-right.
- The less variables one needs to express oneself clearly, the better.
- Smalltalk-style environment is still better than anything other
  languages have to offer.
- Image based development is not a good idea: focusing on reproduction of
  state is better than saving state.
- Stateful notebooks are a good idea, but as an exploration tool, not as
  a development environment.
- A well designed language plus a semi-decent compiler should reliably produce
  code roughly matching the performance of equivalent -O1 compiled C++ code.
  Factor of 2 is acceptable, factor of 10 is not.
- A language that doesn't have a good solution for doing multithreading reliably
  is not a modern general purpose language.

## Project Plan

Timeboxing in releases of 50-200h of work.

### WIP: 0.1.1: Better Parser

Support the full syntax I have in mind, provide reasonable errors.

- [x] Integers
- [x] Floats
- [x] Unary messages
- [x] Binary messages
- [x] Keyword messages
- [x] Chains
- [x] Cascades
- [x] Sequences
- [x] Newline sequences
- [x] Blocks
- [x] Prefix minus
- [x] Let => bind(var, expr, body)
- [x] Assign
- [x] Return
- [x] Arrays
- [x] Type annotations
- [x] Parenthesis
- [x] Selectors
- [x] Nicer errors (two lines of context, put mark where the error is)
          001 foo,
          002 bar, ,quux
                   ^-- Unexpected comma
          003 zot
- [ ] Literal arrays
- [ ] "foo" Strings
- [ ] """ Strings
- [ ] String interpolation and literal $"strings"
- [ ] # Comments
      Attach end-of-line comments to the previous expression
      Attach block comments to the next expression
- [ ] 'x' Characters
- [ ] Definitions
- [ ] Parse into GlobalVariable, LocalVariable, InstanceVariable, MethodParameter,
      BlockParameter
      - Uppercase is Global.
      - Locals are know to parser.
      - Parameters are lexically visible hence knowable to parser.
      - Ergo: We know which instance variables a method requires immediately
        after a parse, and if some are undefined.
- [ ] Replace the old parser and support ast changes in evaluator

### Planned: 0.2.0: The IDE

_...a bad SmallTalk with an environment from the 80s?_

**Goals**: a Smalltalk-like ID, actually writing foolang code.

**Non-goals**: performance, fancy extensions, useful class library.

Estimated remaining: 89h
Time spent: 5h

- Language:
  - ~~[ ] Message chaining. 2h~~
    This turned out to be harder than I thought. I'm pretty sure it's
    parsable, but I'm having trouble getting it right. Seems like something
    that would be easy to do in a hand-written parser... but I don't think
    I want to write one just now. OTOH, it would be a "good" opportunity
    to write Rust and Foo in parallel.
- Cleanups:
  - [x] Expressions are sequenced with comma, not dot. 1h
  - [x] rename Array::each to Array::do. 1h
  - [x] Fix empty array constructor. 1h
  - [x] Azure pipeline for Windows and OS X as well.
  - [x] Move methods from evaluator.rs into classes/class.rs. 1h.
  - [ ] Move sub-object definitions from objects.rs into classes/class.rs. 1h
- [x] Try out mkdoc for documentation: 1h
- [ ] Wrap a rust web server as foolang object. 5h
- Bare bones class browser widget: 17h
  - [x] Foolang classes. 1h
  - [ ] ClassMirror name. 1h
  - [ ] ClassMirror slots. 1h.
  - [ ] ClassMirror methods. 1h.
  - [ ] ClassMirror help. 1h
  - [ ] MethodMirror source. 1h.
  - [ ] MethodMirror isClassMethod. 1h.
  - [ ] MethodMirror class. 1h.
  - [ ] MethodMirror selector. 1h.
  - [ ] MethodMirror parameters. 1h.
  - [ ] MethodMirror help. 1h.
  - [ ] /foolang/list-classes. 1h
  - [ ] /foolang/class/<name> => {name,help,slots,methods,classMethods} 1h
  - [ ] /foolang/method/<class>/<selector> => {name,help,parameters,source} 1h
  - [ ] browser.html + js. 5h
- [ ] Source formatter for class browser. 10h.
- [ ] Editing in the class browser: 10h
- [ ] Bare bones playground widget. 10h
- [ ] Bare bones transcript widget. 10h
- [ ] Bare bones inspector widget. 10h
- [ ] Integrate these components into a whole. 10h
- [ ] Make things slightly less bare bones. 5h
- Optional extras:
  - Syntax & semantics package
    - [ ] 1 to: 10 creates a range.
    - [ ] Instance syntax: ClassName::{ ... } $ClassName::{ ... }
    - [ ] Change methods to default to last expression as return
    - [ ] Change variable binding to let.
    - [ ] if <expr> {} else if <expr> {} else {}
    - [ ] load-time-eval / compile-time-eval like things.
  - [ ] Remove literal arrays or make the immutable.
  - [ ] Record syntax [ foo: x signum. quux: y. ] and #[foo: 42]
  - [ ] String interpolation '1 + 1 = {1 + 1}'
  - [ ] => { :that | ... }
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

- [ ] Bytecode -> IR decompiler. Est 10h.
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

## IDE

I like the way Smalltalk IDE automate code organization.

I don't like the way the question of presentation is entirely removed:
if two methods are best understood when viewed next to each other then
that should be the default mode of display.

I think I would like Deuce-like views:

- Selected methods of this class.
- All methods with this selector.
- All methods using thise selector.
- All methods referring to this global.
- Current method followed by selectors on self it invokes.

Default presentation of a class is an interesting question.

Documented methods first?

Methods grouped by protocol?

Methods mentioned in class docstring first?

## References

- 1973 - [Top Down Operator Precedence](papers/pratt.pdf) by Vaughan R. Pratt. \
  _Foolang uses a Pratt-style parser._

- 1983 - [Smalltalk-80: The Language and Its Implementation](papers/bluebook.pdf)
  Adale Goldberd and David Robson

- 1993 - [Strongtalk, Typechecking Smalltalk in a Production Environment](papers/strongtalk-typechecking.pdf) by Gilad Brancha and David Griswold.

- 2002 - [Destructors, Finalizers, and Synchronization](papers/Boe02-Destructors.pdf)
  by Hans-J. Boehm. \
  _This paper is a source of a lot of design-angst to me._

- 2007 - [Open, extensible object models](papers/objmodel2.pdf) by Ian Piumarta and
  Alessandro Warth. \
  _Hugely influential in the design of Foolang._

- 2008 - [The Fortress Language Specification](papers/fortress-spec.pdf) by various. \
  _Foolang has quite different design criteria than Fortress, but Fortress
  was an amazing design and well worth learning from._

## Notes

- Various browser editor components
  - Rich text: https://quilljs.com/guides/why-quill/
  - Code editor: https://ace.c9.io/
  - Code editor: https://codemirror.net/doc/manual.html
  - Code editor: https://github.com/Microsoft/monaco-editor
  - Code editor: https://icecoder.net/
  - Drawing: http://literallycanvas.com/

- http://joeduffyblog.com/2016/02/07/the-error-model/

- http://projetos.vidageek.net/mirror/mirror/

- http://crockford.com/javascript/tdop/tdop.html

- http://journal.stuffwithstuff.com/2011/02/13/extending-syntax-from-within-a-language/

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

### GC

Simplest thing that could possibly work: mark and sweep on top of malloc and
free.

Allocation header:
```
  Bits 00-01: GC marks
  Bits 02-09: number of raw words
  Bits 10-17: number of gc slots
  Bits 18-25: number of weak slots
  Bits 26-28: no tail / raw tail / gc tail / weak tail
  Bits 29-31: n^2 = tail element width in bytes if raw tail
```

## BIG GOAL

class Ackermann
  m: m<u64> n: n<u64> ^<u64>
    m == 0 then: {
      ^n + 1
    }
    n == 0 then: {
      ^Ackermann m: m - 1 n: 1
    }
    Ackermann m: m - 1 n: (Ackermann m: m n: n - 1)

This should compile into decent native code. Something close enough
to what gcc -O0 would produce.

## MMmmmaaaybe

- Token:

     Literals: 123
     Names: foo
     Sigils: non-alpha

    Precedence: function, prefix, unary, binary*, keyword

    expr := literal | variable-ref | compound-expr | '(' expr ')' | expr '--' expr

    compound-expr := prefix-expr | unary-expr | binary-expr | keyword-expr



- Million dollar question: how are binary operations implemented?

  1. Just messages.
  2. Translation layer to messages.
  3. Separate dispatch.

  I kind of like #2. It allows making the second argument the receiver,
  and provides an implementation mechanism for unary prefixes. (I really want
  -foo, I think.)

- The more I think about it the more I like the idea of "main" receiving an
  object that provides the OS interfaces. The only question is ergonomics. How
  to support printf-debugging in a random method? I have several answers.

  1. Dynamic variables.
  2. It isn't _that_ hard to pass in a logger.
  3. A nice environment means you don't reach for the printf
     in the first place.

- Smalltalk actually puts a lot of things which might otherwise belong in a
  control flow class (or exist as first-class constructs) into Object:

      self handle: [ :exeption | ... ]
           do [ ... ]

  Control flow class:

      Handler do: { ... }
              onExeption: { ... }

  Block:

      { ... } handle: { ... } finally: { ... }

  First class syntax:

      try expr handle: { ... } finally: { ... }

      unwind <expr> protect: { }

- Terminology: Words and sigils are both symbols.

      foo <-- word

      ++  <-- sigil

- Syntax:

    Allow newline to replace comma?

    double-colons for namespacing and explicit extensions?

    Module::Class new

    "foo" ext::capitalize

- Binary messages

    Consider binary messages to be syntax level transformations into
    messages.

        @binary-operator left => right
           right value left

        @binary-operator left - right
           left sub: right

    Because the parser needs to know the precedence anyhow, it seems to
    me that binary operators don't need to be strictly restricted to symbols anyhow.

        @import MyOps::div

        x div y

- Would be pretty cool if http://asciimath.org/ could be used, at least in
  comments!

- Could I support chaining for comparison operators?

      0 < a < 5   and  a = b = c

    ...I could. Parser could deal with them especially. Just say that comparison
    operators are chained using and.

      a _op1_ b _op2_ c ==> a _op1__ (let tmp := b) & tmp __op2__ c

    The constraint that some things are classed as comparison operators.

    This is not more complicated than precedence.

- I still wonder if blocks should have "selectors" instead,
  ```
    { :foo :ding | ... } foo: x ding: y
  ```
  reserving value for no-arg blocks.
  ...but then block class cannot implement things like repeat.
  ...but then I have the perfect excuse of making control classes like
  ```
   Loop while: { } do: { }
   Loop repeat: { }
   Loop doUntil: { }
   Loop until: { } do: { }
   Cond if: { } then: { }, elseif: { } then: { }, else: { }
  ```
  That's pretty nice but would be even better if I could figure out
  how to make
  ```
      If: { } then: { }
  ```
  work. ...and optimizing
  ```
   Cond if: { a } then: { x }, elseif: { b } then: { y }, else: { z }
   ==>
   Cond if: { a } then: { x } else: { Cond if: { b } then: { y } else: { z }}
  ```
  is an interesting problem! Note though, that I can implement these classes
  even if block have repeat and booleans have ifTrue, etc.
- Make ${} and $[] a literal json objects

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
