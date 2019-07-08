[![Build Status](https://dev.azure.com/nikodemus0619/foolang/_apis/build/status/nikodemus.foolang?branchName=master)](https://dev.azure.com/nikodemus0619/foolang/_build/latest?definitionId=1&branchName=master)

# foolang

A Smalltalk-inspired object language that liberally cribs from Common Lisp,
Erlang and Fortress.

Like all new languages it has somewhat unrealistic aspirations:

- Elegance of Smalltalk.
- Performace of C++.
- Convenience of Python.
- Fault tolerance of Erlang.
- Extensibility of Common Lisp.

**Status**: bootstrap evaluator implementation in progress.

## Hello World

    @main: system
      system output println: "Hello world!"

## Design Principles

In order of priority:

1. Ergonomics: Code should be a pleasure to write and easy to read.

2. Safety: No memory errors. No race conditions. No ambient authority.
   Fault-tolerant applications.

3. Performance: Code with type annotations should run on par with -O0
   compiled C++.

4. Uniformity: Built-in code should not be priviledged over user code.

Finally, there is an absolute requirement of implementability: Foolang is ment
to be a real language, not a pipe dream, but it is also a one-person effort at
the moment. So: "Don't summon anything bigger than your head."

## Core Features

- Everything is an object, and semantics are descibed by sending
  messages to objects.

- Smalltalkish development environment -- except code lives in files
  where it belongs, not in an image.

- Supervised processes and isolated heaps for Erlang-style
  fault-tolerence.

- Syntactic extensibility. Users can define new operators and other
  syntax extensions.

- Dynamic bindings in addition to lexical ones provide ability to
  implement things like context oriented programming and exception
  mechanisms as libraries.

- No ambient authority: third-party libraries don't have access to your
  filesystem and network unless you give it to them.

## Implementation Plan

1. WIP: Bootstrap evaluator written in Rust, supporting the full language.

2. TODO: Self-hosted (plus Javascript, etc) web-IDE.

3. TODO: Self-hosted Foolang to Rust compiler (possibly for a subset.)

4. TODO: Self-hosted Foolang to bytecode compiler, VM, and GC (compiled to Rust).

5. TODO: Self-hosted JIT and AOT native compiler. (LLVM or Cranelift as backend.)

## Syntax notes

- I quite like the idea of making "reserved words" uppercase keywords:

       Class: Foo { zot = 42 }
         constructor quux
         method bar: y
            Let x = zot + y
            Return: x
         end
      end

  It is reasonably unobtrusive, and allows for extensibility. It also
  makes it obvious for both humans and machines.

  Then reserve another syntactic class for user-level syntax extensions:

       $withDatabase { }

       withDatabase! { }

       !withDatabase { }
  
       !Cond:
          maybe => then

    
       Cond!
          maybe => then

    
       !Cond:
          maybe => then

  ...or maybe allow users to implement Foo: things as well? Just impose
  import rules.

  That works.

- I really miss question mark as part of message syntax. It's so nice
  and obvious for unary tests. is-prefix gets old fast.

  Bang is nice too.

- Forth's "everything is delimited by whitespace" is _really_ powerful.

- '|' would be really useful as just syntax. ...but it _is_ also
  really convenient the second you start twiddling bits.

- Use -> for chaining instead of --

- Canonicalize multipart keyword messages by stable-sorting the keys.
  (Evaluatior order stays.)

- "Object-oriented" comprehensions are a bit awkward for multiple dimensions.
  Something like:

      For iterators: [as, bs, cs]
          where: { |a,b,c| a*a == (b*b + c*c) }
          collect: { |a,b,c| [a,b,c] }

  maybe?

- Possible conflict with possible dict/record comprehensition syntax:

  Consider:

      staff select: {_senior} -> inject: Dictionary new into: { |senior, dict| 
        dict put: senior at: senior name
      }

   Rendered as:

       { employee name: employee | employee <- staff, employee senior }

   Cannot tell that this is a comprehension very easily.

       { employee name => employee | employee <- staff, employee senior }

   Is clearer, but Json-compatible literals are just awfully nice. (Not required, though.)

   Wat about

       { (employee name): employee | employee <- staff, employee senior }

   Alternatively I could reserve => for cases where the names are not literal,
   and for literal string or selector keys.

       { foo: 42 } => record

       { "foo bar": 42 } => dict

       { (foo): 42 } => dict (evalute foo)

   Or

       { foo => 32 } as record

   Because consider: records are really literal objects. I would kind of like
   have the option to delegate, have inline methods, etc.

       { 
           foo => 1
           bar: x => (
             old = _slot
             _slot += x
             old 
           ),
           _slot = 42
       }

       {
          doesNotUnderstand: selector with: args => {
             log write: selector
             object perform: selector with: args
          }
       }

- Using words to define operators. Maybe `x _max_ y` or `x \max y`. Either would
  also be nice syntax for entering unicode operators.

- Extending pratt parsing to do non-transitive precedence?

  1. Instead of passing around precedence and comparing it numerically
     pass around precedence object and use that.

  2. Precedence classes like Arithmetic > Comparison > Logical
     are non-transitive.

  3. Precedence groups within classes are transitive and either
     left-associative, non-associative, or composing-associtive.

     x > y < z is an example of a composing-associative group with
     the composing operator &&.

     => tmp := none, (x > (tmp := y)) && (tmp < z)

  4. Precedence groups are organized into rows. Operators on the same row
     are at the same precedence. Operators at higher rows are at higher
     precedence.

  5. Each row has an implicit "before" and "after" extension slot. Things
     in the before slot are higher than the row but not lower than the
     higher rows. Things in the after slot are lower than the row but not
     higher than lower rows.

      @precedenceCLass Arithmetic
         before: Comparison
  
      @precedenceClass Comparison
         before: Logical

      @precedenceClass Logical

      @associativeGroup[Arithmetic]
         [\arrow]
         [*, /]
         [+, -]

      @associativeGroup[Arithmetic]
         [<<, >>]
         [|, &, \xor]

      @binaryOperator[Arithmeric before: *] ^

   So ^ relates to all other arithmetic operators except \arrow.

      @extend[Integer] ^ x
         x == 0 then: { return 1 }
         x == 1 then: { return self }
         self * self ^ (x-1)

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

- http://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.127.8460&rep=rep1&type=pdf

- https://ferd.ca/the-hitchhiker-s-guide-to-the-unexpected.html

- https://ferd.ca/the-zen-of-erlang.html

- https://devblogs.microsoft.com/cbrumme/reliability/

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

### Phrases

```
@phrase ring!
    "No unbound variables allowed! Constants are OK, though."
    ding ding ding

bell ring!
```

## BIG GOAL

```
@class Ackermann {}
  method m: m<u64> n: n<u64> -> <u64>
    m == 0 then: {
      return n + 1
    }
    n == 0 then: {
      return Ackermann m: m - 1 n: 1
    }
    Ackermann m: m - 1 n: (Ackermann m: m n: n - 1)
```

This should compile into decent native code. Something close enough to
what gcc -O0 would produce.

## MMmmmaaaybe

- Million dollar question: how are binary operations implemented?

  1. Just messages.
  2. Toplevel generic functions.

  I will go with messages for now: inlining should be able to get good-enough
  speed out of double-dispatch, and fast-pathing "simple dispatch" inlining
  (ie. simply sending another message with re-arranged receiver and arguments
  seems pretty simple)

- Smalltalk actually puts a lot of things which might otherwise belong in a
  control flow class (or exist as first-class constructs) into Object:

      self handle: [ :exeption | ... ]
           do [ ... ]

  Control flow class:

      Try do: { ... } or: { ... }

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

- Allow using _underscored_ words to define operators: `x _max_ y` is pretty nice.

- Allow using \escaped to denote unicode symbols with user-defined aliases:

     a \xor b

- Allow globals providing complex operations to have unicode renderings:

     Sigma over: 1...10 do: {|n| (n + 1) / n }

                          10   n + 1
  should be rendered as SIGMA --------
                         n=1     n

  ...though I submit that the fikkiw reads better:


     1...10 -- sum: {|n| (n+1) / n }


  The editor will render them pretty.

- Can I make |a| do the right thing? Ie. a magnitude

- How to use mincore to guide GC?
  (1) incremental GC could use mincore to find out if a page is available before
      hitting it.
  (2) non-incremental GC could use mincore + MADV_WILLNEED to try to order operations
      to avoid blocking on faults. ...then again, just keeping a stack and staying on
      the same page trying to work linearly should get there too.
