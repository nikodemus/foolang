[![Build Status](https://dev.azure.com/nikodemus0619/foolang/_apis/build/status/nikodemus.foolang?branchName=master)](https://dev.azure.com/nikodemus0619/foolang/_build/latest?definitionId=1&branchName=master)

# foolang

A Smalltalk-style object language that liberally cribs from Common
Lisp, Erlang and Fortress.

Like all new languages it has _somewhat_ unrealistic aspirations:

- Elegance of Smalltalk.
- Performance of C++.
- Convenience of Python.
- Fault tolerance of Erlang.
- Extensibility of Common Lisp.
- Excitement of Logo.
- Success of Fortress

## Hello World

    class Main { system }
        method run
            system output println: "Hello world!"
    end

## Metagoals

- Experienced programmers must feel comfortable: the system trusts
  them and they can trust the system.

- People who've only ever done "spreadsheet programming", or who last
  programmed some Basic on Commodore 64 over 30 years ago should feel
  empowered to do things.

- Programs written by others to be easy to understand after the
  fact.

- The environment is should be engaging and inspiring and empowering.
  Not toys, but good and easy to use tools.

## Design Principles

In order of priority:

1. Safety: No memory errors. No race conditions. No ambient authority.
   No undefined behaviour. Fault-tolerant applications.

2. Ergonomics: Code should be a pleasure to read and write.

3. Performance: Code with type annotations should run on par with -O0
   compiled "equivalent" C or C++.

4. Uniformity: Built-in code should not be privileged over user code.

Finally, there is an absolute requirement of implementability: Foolang
is to be a real language, not a pipe dream, but it is also a
one-person effort at the moment. So: "Don't summon anything bigger
than your head."

## Core Features

- Everything is an object, and semantics are descibed by sending
  messages to objects.

- Smalltalkish development environment -- except code lives in files
  where it belongs, not in an image. Usable with your favorite editor,
  even if the best experience is in the "native" environment.

- Supervised processes and isolated heaps for Erlang-style
  fault-tolerence.

- Syntactic extensibility. Users can define new operators and other
  syntax extensions.

- Dynamic bindings in addition to lexical ones provide ability to
  implement things like context oriented programming and exception
  mechanisms as libraries.

- No ambient authority: third-party libraries don't have access to your
  filesystem and network unless you give it to them.

## Implementation Plan & Status

1. WIP: Bootstrap evaluator written in Rust, supporting the full
   language, including a web-based IDE.

2. TODO: Self-hosted Foolang to LLVM assembly compiler (possibly for a
   subset of Foolang.)

   Rationale: Since Foolang is to be an AOT compiled language going
   from interpreter to compiler very early makes perfect sense.
   Generating LLVM source files instead of directly interfacing should
   both make debugging easier and avoid headaches with interfacing to
   LLVM.

   Option: instead of targeting LLVM assembly generate rust source
   code instead. Pro: LLVM not needed for bootstrapping, easier to
   keep using Rust-provided dependencies, can use Rc for GC. Con: Rust
   may be a painful target.

3. TODO: Self-hosted Foolang to bytecode compiler, VM, and GC
   (compiled using compiler from step 2.)

   Rationale: Self-hosting GC and VM are perfect battlegrounds to make
   sure Foolang can support high performance systems programming tasks.
   VM is required for a nice development experience.

4. TODO: Self-hosted JIT and AOT native compiler. (LLVM backend.)

   Rationale: half the LLVM backend exists already from step #2,
   slow JIT should not be a huge issue as it is to be mainly a
   development time facility.

## Syntax

Foolang is expression oriented: every expression has a value.

**Reserved Words**

Following words have special meaning:

    let
    return
    class
    define
    method
    end
    import

**Comments**

Line comments are prefixed with `--`:

    -- This is a comment

Block comments are surrounded by `---`:

    output println: --- This is a comment --- "Hello!"

    ---
    This is a comment
    Still a comment
    ---

End of line comments describing the value the line evaluates to are
conventionally prefixed with `-->`, but syntactically this is just a
line comment that starts with a greater-than sign:

    x = y + z --> sum of x and y

**Literals**

Numbers are quite conventional, with _ as allowed readability separators.
Literal floats are doubles, unless written in the exponential format using
"f" as the exponent separator.

    1_000_000
    0xDEADBEEF
    0b011011
    1.0     -- double
    1.0e0   -- double
    1.0f0   -- single

Strings allow interpolations to be embedded:

    "Hello {user name}!"

**Messages**

Messages are sent by simple concatenation, Smalltalk-style:

    -- Message with selector "foo" and no arguments to object
    object foo

    -- Message with selector "foo:bar:" and arguments 1 and 2 to object
    object foo: 1 bar: 2

**Expression Separators**

Expressions are separated form each other with dots, Smalltalk-style:

    -- Message with selector "foo" to objectA, message with selector "bar" to
    -- objectB. Newline can be elided.
    objectA foo.
    objectB bar

A sequence of expressions like this evaluates to the value of the last one.

**Operator Precedence**

Unlike Smalltalk operators have conventional precedence:

    1 + 2 * 3 --> 7

**Local Variables**

Defined using let:

    let x = 42.
    x * 100 --> 420

**Blocks**

Unlike Smalltalk Foolang uses braces for blocks:

    { 1 + 2 } value --> 3

Blocks can take arguments:

    { |x| x + 2 } value: 40 --> 42

    { |x, y| x + y } value: 40 value: 2 --> 42

Underscore can be used as an implicit argument in blocks:

    -- Odd numbers from an array
    array select: { |elt| elt isOdd }

    -- Same using the implicit argument
    array select: { _ isOdd }

**Class Definitions**

    class Point { x, y }
       method + other
          Point x: x + other x
                y: y + other y

       method display
          "#<Point {x,y}>"

    end

    -- Implicit constructor using the slot names
    let p0 = Point x: 1 :y 2     --> #<Point 1,2>
    let p1 = Point x: 100 y: 200 --> #<Point 100,200>
    let p2 = p0 + p1             --> #<Point 101,202>

**Type Annotations**

Double-colon suffix is used for type annotations.

    expression::Type

    let variable::Type = value

    class Foo { instanceVariable::Type }
       method bar: argument::Type -> ReturnType
          ...
    end

**Arrays**

Are constructed using square brackets

    let array = [1, 2, 3]

Can be specialized using a message

    let byteArray = [1, 2, 3] of: U8

**Dictionaries**

Are constructed using braces

    let dict = {1: "one", 2: "two"}

## Notes

```
For iterators: [as, bs, cs]
        where: { |a,b,c| a*a == (b*b + c*c) }
       collect: { |a,b,c| [a,b,c] }
```

maybe?

- before, after, and around methods

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

     => let tmp = y. (x > tmp) && (tmp < z)

  4. Precedence groups are organized into rows. Operators on the same row
     are at the same precedence. Operators at higher rows are at higher
     precedence.

  5. Each row has an implicit "before" and "after" extension slot. Things
     in the before slot are higher than the row but not lower than the
     higher rows. Things in the after slot are lower than the row but not
     higher than lower rows.

      precedenceClass Arithmetic
         before: Comparison
  
      precedenceClass Comparison
         before: Logical

      precedenceClass Logical

      associativeGroup[Arithmetic]
         [\arrow]
         [*, /]
         [+, -]

      associativeGroup[Arithmetic]
         [<<, >>]
         [|, &, \xor]

      binaryOperator[Arithmeric before: *] ^

   So ^ relates to all other arithmetic operators except \arrow.

      extend[Integer] ^ x
         x == 0 then: { return 1 }
         x == 1 then: { return self }
         self * self ^ (x-1)

   ...but this gets the associativity wrong: 2^3^4 == (2^(2^3))

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
phrase ring!
    "No unbound variables allowed! Constants are OK, though."
    ding ding ding
end

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

- Binary messages

    Consider binary messages to be syntax level transformations into
    messages.

        binaryOperator left => right
           right value left

        binaryOperator left - right
           left sub: right

    Because the parser needs to know the precedence anyhow, it seems to
    me that binary operators don't need to be strictly restricted to symbols anyhow.

        import MyOps.div

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


     1...10 sum: {|n| (n+1) / n }


  The editor will render them pretty.

- Can I make |a| do the right thing? Ie. a magnitude

- How to use mincore to guide GC?
  (1) incremental GC could use mincore to find out if a page is available before
      hitting it.
  (2) non-incremental GC could use mincore + MADV_WILLNEED to try to order operations
      to avoid blocking on faults. ...then again, just keeping a stack and staying on
      the same page trying to work linearly should get there too.
