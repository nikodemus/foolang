[![Build Status](https://dev.azure.com/nikodemus0619/foolang/_apis/build/status/nikodemus.foolang?branchName=master)](https://dev.azure.com/nikodemus0619/foolang/_build/latest?definitionId=1&branchName=master)

# foolang

A Smalltalk-inspired object language that liberally cribs from Erlang
and Fortress.

**Status**: bootstrap evaluator implementation in progress.

## Hello World

    @main: system
      system output println: "Hello world!"

## Design Principles

1. Ergonomics: Code should be a pleasure to write and easy to read.

2. Safety: No memory errors. No race conditions. No ambient authority.
   Fault-tolerant applications.

3. Performance: Code with type annotations should run on par with -O0
   compiled C++.

4. Don't summon anything bigger than your head. Foolang is currently
   a one-person effort, so design has to take implementability into
   consideration.

## Core Features

- Everything is an object, and semantics are descibed by sending
  messages to objects.

- Smalltalkish development environment -- except code lives in files
  where it belongs, not in an image.

- Supervised processes and isolated heaps for Erlang-style
  fault-tolerence.

- Syntactic extensibility. Users can define new operators and other
  syntax extensions.

## Syntax notes

- | would be really useful as just syntax. ...but it _is_ also really convenient
  the second you start twiddling bits.

- Use -> for chaining instead of --

- Canonicalize multipart keyword messages by stable-sorting the keys.
  (Evaluatior order stays.)

- Possible conflict in record comprehensition syntax: Well, dict
  comprehension maybe.

  Consider:

      staff select: {_senior} -> inject: Dictionary new into: { |senior, dict| 
        dict put: senior at: senior name
      }

      @method[Iterable] select: block
        self inject: self collector into: { |elt, collector|
            block value: elt -> then: { collector add: elt }
            collector
        }

      @method[Iterable] inject: collector int: block
        let x := collector
        self do: { |elt| x := (block value: elt value: x) }
        x

      If the compiler sees these three, what can it do?

      First inline the select:

        { |self,block|
          self inject: self collector into: { |elt, collector|
              block value: elt -> then: { collector add: elt }
              collector
          } value: staff value: block

      Then inline the block:

        { |self|
          self inject: self collector into: { |elt, collector|
              {_ senior} value: elt -> then: { collector add: elt }
          collector } value: staff

       Again:

        { |self|
          self inject: self collector into: { |elt, collector|
              elt senior -> then: { collector add: elt }
          collector } value: staff

       Inline staff:

          staff inject: staff collector into: { |elt, collector|
              elt senior -> then: { collector add: elt }
          collector }

       Inline inject:

          { |self,collector,block|
            let x := collector
            self do: { |elt| x := block(value, x) }
            x } value: staff
                value: staff collector
                value: { |elt, collector|
                         elt senior -> then: { collector add: elt }
                         collector }

       Inline staff:

          { |collector,block|
            let x := collector
            staff do: { |elt| x := block(value, x) }
            x } value: staff collector
                value: { |elt, collector|
                         elt senior -> then: { collector add: elt }
                         collector }

       Inline block:

          { |collector|
            let x := collector
            staff do: { |elt| 
                         x := { |elt, collector|
                                elt senior -> then: { collector add: elt }
                                collector } value: elt value: x }
            x } value: staff collector

        Inline elt & x

          (let x := staff collector
           staff do: { |elt| 
                         x := (elt senior -> then: { x add: elt },  x)
                     }
           x)


      So... I think St-style iterators can be optimized OK, as long
      as we understand the types.

      ^-- these don't deal with multiple dimensions nicely, which one of
          the reasons comprehensions are cool.

      For iterators: [as, bs, cs]
          where: { |a,b,c| a*a == (b*b + c*c) }
          collect: { |a,b,c| [a,b,c] }

      ...is not terrible, though.


   Rendered as:

       { employee name: employee | employee <- staff, employee senior }

   Cannot tell that this is a comprehension very easily.

       { employee name => employee | employee <- staff, employee senior }

   Is clearer, but Json-compatible literals are just awfully nice.

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


- I really miss question mark as part of message syntax. Look at the "senior"
  above. It would be so much better as senior?

  

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

  @method[Integer] ^ x
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

