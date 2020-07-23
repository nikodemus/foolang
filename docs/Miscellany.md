# Foolang Miscellany

## Zip

While `with:collect:` and friends are not bad, they require a fair amount
of duplication

An efficient zip could be so much nicer:

`(names zip: addresses) do: { |name address| db save: name :and address }`

Efficient meaning a `Zipper { left right }`, which can then be open
coded into equivalent code as hand written `with:collect:`

## Method catenations

Consider, what if method selectors were broken up based on existence.

Assuming Foo#bar: -> Quux, and Quux#zot: -> Zot

Foo bar: 42 zot: 13 === (Foo bar: 42) zot: 13

This means that absent `Integer#to:do`, `1 to: 10 do: ...` would still work.

Existence of methodNotUnderstood makes this hard, though.

...and it might not be so nice anyhow.

Still, kind of like method currying, except not.

## Block vs object syntax

Should Foolang should use `[]` for blocks instead? Rectangular blocks suit ST
style code, and look much better with square bracket. Problem: what about arrays
then?

OTOH, `{}` is for better and worse what everyone expects blocks to look like.

## is vs equals

`a = b --> True` iff a is same object as b (currently this is `is`)

- Pro: natural because: `let a = b. a = b --> True`
  I also think this is a reasonable thing to ask.
- Pro: less exceptions syntax
- Con: "doesn't scan"
- Con: if this is a regular method, then it can be overridden, which seems
  terrible

Same reasoning except for the first pro applies to making a `is:` method, and
`is:` does scan, but doesn't work as nicely vs. precedence.

Non-overridable methods in Object?

## Literate Programming

mymodule.litfoo:

```
This is a description of an algorithm from input to output.

@algorithm_1: input, output
---
   input each: { output handleInput: _ }
---

This is a description of a method.

@method_foo
---
method foo: input
    let output = Output new
    @algorithm_1
    return output
---

This is a description of a class.

@@
---
class Foo
    @method_foo
end
---
```

Hm.

## before, after, and around methods

Yes.

## Extending pratt parsing to do non-transitive precedence?

1. Instead of passing around precedence and comparing it numerically
   pass around precedence object and use that.

2. Precedence classes like Arithmetic > Comparison > Logical
   are non-transitive.

3. Precedence groups within classes are transitive and either
   left-associative, non-associative, or composing-associtive.

   `x > y < z` is an example of a composing-associative group with
   the composing operator &&.

   `--> let tmp = y. (x > tmp) && (tmp < z)`

4. Precedence groups are organized into rows. Operators on the same row
   are at the same precedence. Operators at higher rows are at higher
   precedence.

5. Each row has an implicit "before" and "after" extension slot. Things
   in the before slot are higher than the row but not lower than the
   higher rows. Things in the after slot are lower than the row but not
   higher than lower rows.

   ```
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
   ```

   So ^ relates to all other arithmetic operators except \arrow.

   ```
   extend[Integer] ^ x
      x == 0 then: { return 1 }
      x == 1 then: { return self }
      self * self ^ (x-1)
   ```

   ...but this gets the associativity wrong: 2^3^4 == (2^(2^3))

   Bleg.

## Three Uses Of Whitespace in Fortress

- Meaning of vertical bar:

  ```
   |x|   --> abs
   a | b --> a or/cat/whatevs b
  ```

- Subscripts vs array constructor

- Verifying intent of operator precedence

### Idea

`X` is an expression.

`SigilF` is a non-alphabetic symbol containing no `<|>` characters.

`SigilB` is a non-alphabetic symbol containing only `<|>` characters.

`SigilG` is a non-alphabetic symbol.

Prefix sigils:

    <SigilF>X

    prefix method <SigilF> ...

    -X

    !X

    ~X

Suffix sigils:

    X<SigilF>

    suffix method <SigilF> ...

    X!

    X?

Surround sigils:

    <SigilB-left>X<SigilB-right>

    bracket method <SigilB-left> <SigilB-right> ...
    
  These need to be limited to catenations of <|>.

Binary sigils:

    x <sigil> y

    method <sigil> other ...
    
Reserve operators ending with = -> makes

    x <anyop>= 1

parsable generally.

Precedence remains an issue: precedende is a syntactic property, so it needs to
be known when parsing. (Well. I could defer it probably if I really wanted.)

## Plan

```
delimiters := , ; ( ) { } [ ]
```

these will only ever tokenize to a single character!

```
literal-prefix := #
```

affects the parsing of the following syntactic element

```
dynamic-binding-prefix := $
```

```
line-comment := --
```

```
identifier := _?[:alphabetic:][:alphanumeric:]*[?!']*?
```
GENERALLY identifiers in value position are evaluated as local
variables or global constants.

The system provides parsers for some identifiers which appear
in value positions, like `let` and `return`.

In value position identifiers which have a case must be local if lowercase and
global if uppercase or titlecase.

     -- local
     foo

     -- global
     Foo

     -- either
     测试
     _foo

Identifiers in operator position are evaluated as unary messages to
the value.

     -- message bar to value of foo
     foo bar

     -- message 跑 to 测试
     测试 跑

Outside development mode messages prefixed with underscore are
allowed only to self.

     -- Syntax error outside development mode
     foo _bar

     -- Always legal
     self _bar

```
sigils := [:non-alphabetic:]+
```
Sigils in value position are evaluated as prefix operators.
Prefix operators _must_ precede their arguments without
intervening whitespace.

Sigils in operator position are evaluated as postfix or binary
operators.

Postfix operators _must_ follow their arguments without intervening
whitespace.

Binary operators _must_ have balanced whitespace, and whitespace _must_
match precedence:

    -- Legal, precedence matches whitespace
    x*y + b

    -- Legal, precedence does not conflict with whitespace
    x * y + b

    -- Illegal, precedence conflicts with whitespace
    x * y+b

Sigils beginning with . : and = are served for the implementation

    -- Type annotation
    foo::Float

```
keyword := _?[:alphabetic:][:alphanumeric:]*: (must be followed by whitespace!)
```

Keywords in value position are not allowed.

XXX: keywords in value position _could_ be reserved for implementation:

instead of:
```
class Foo {}
    method bar: x
       let y = x.
       return y + 1
end
```

it could be:

```
class: Foo {}
    method: bar: x
        let: y = x.
        return: y + 1.
end
```
in which case _only_ end would be needed as a reserved word!


Keywords in operator position are parts of keyword messages.

     -- Message bar:quux: with arguments 1 and 2 to foo
     foo bar: 1 quux: 2

The only truly global reserved words are:

    import
    from
    as
    end
    self

Whitespace and , are also fully built-in.

Rest of Foolang needs to be imported:

    import: foolang.*

Importing while keeping the package prefix:

    import: foolang as: foo

Partial imports and aliasing:

    from: foolang.v0 import:
       let
       return as: ret
    end

Toplevel definitions:

    -- Core --

    define: Foo = 12

    -- Sugars --

    define: Foo {}
       method bar 42
    end

    -- desugars to:

    define Foo = class {}
       method bar 42
    done <name: Foo>

    define Foo(a, b) { a + b }

    -- desugars to:

    define Foo = { |a,b| a + b } <name: Foo>

Built-in parser words:

    is

        a is b ==> True IFF a is the same object as b

    let 

        let a = b, ... ==> local variable binding for a as b in the sequence
        that follows.

    return

        return a ==> returns a from current frame.  All methods have an implicit frame,

    __do__

        do { ... } establishes an explicit frame for the enclosed code: return inside
        the lexical scope of the block terminates do, not the method.

    __block__ ... done

        What { ... } desugars into.

    __array__ ... done

        what [ ... ] desugars into.

    __expr__ ... done

        what ( ... ) desugars into.

Reconsidering object syntax:

Assume [] are taken by blocks.

    { 123 } -> Array
    { 1, 2, 3 } --> Array
    { "foo": 42 } --> Dictionary
    { x | x}
 
I think the distinction between objects and blocks is welcome.

However,

    array[x]

is _way_ prettier than

    array{x}

What is the reason I cannot use [] for both blocks and arrays?

Consider

    [ foo ]

Is that an array or a block of one element?

I _could_ say that

    [ foo, ]

is the array. ...but I still lose the distinction between arrays and blocks.


## Webrefs

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

## Pharses

```
phrase ring!
    "No unbound variables allowed! Constants are OK, though."
    ding ding ding
end

bell ring!
```

## MMmmmaaaybe

- Binary messages

    Consider binary messages to be syntax level transformations into
    messages.

        binaryOperator left => right
           right value: left

        binaryOperator left - right
           left sub: right

- Would be pretty cool if http://asciimath.org/ could be used, at least in
  comments

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
  _ prefix for direct methods
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
