[![Build Status](https://dev.azure.com/nikodemus0619/foolang/_apis/build/status/nikodemus.foolang?branchName=master)](https://dev.azure.com/nikodemus0619/foolang/_build/latest?definitionId=1&branchName=master)

# foolang

## Constant Classes + Inline members = Value Semantics

    @class Point { x <Int>, y <Int> }
        constant
        accessors
    @class Rectangle { p1 <inline:Point>, p2 <inline:Point> }
    @classMethod Rectangle new
        self p1: (Point x: 0 y: 0)
             p2: (Point x: 0 y: 0)
    @method Rectangle p1
       p1
    @method Rectangle left
       p1 := Point x: (p1 x + 1) y: p1 y
       p2 := Point x: (p2 x + 1) y: p2 y
       self

- `p1 x` becomes a direct read.
- Constructor targeting p1 becomes an assignment
  Optimize away p1 y := p1 y
- Returning `p1` allocates a Point

How to support `p1 x: newval`? Since Point is a constant a
writer doesn't make sense. If Point is a 'value' instead,
then it makes sense, but then what happens here?

      let box := { value: Point x: 0 y: 0 }
      let p = box value
      box value x: p x + 1
      p x == box value x    

Point/x: cannot make a copy because it doesn't have access
to the slot holding the value.

(What if it had? Ie. pointer to the location as part of the
arguments. THAT might work. Non-values don't need to care.
Values could then do copy-on-write. I still don't like that
it means semantics depend on the type.)

Record/value cannot make a copy because it doesn't know it
should.

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

- Benchmarking: https://github.com/sharkdp/hyperfine

- https://github.com/microsoft/mimalloc
- https://www.microsoft.com/en-us/research/uploads/prod/2019/06/mimalloc-tr-v1.pdf

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
