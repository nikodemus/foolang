# Integer Types

**Status**: ADOPTED (not implemented)

**Identifier**: 006-integer-types

**References**: none

**Prior Art**:
- Common Lisp's integer types:
  - [INTEGER](http://www.lispworks.com/documentation/HyperSpec/Body/t_intege.htm)
  - [SIGNED-BYTE](http://www.lispworks.com/documentation/HyperSpec/Body/t_sgn_by.htm)
  - [UNSIGNED-BYTE](http://www.lispworks.com/documentation/HyperSpec/Body/t_unsgn_.htm)
  - Etc.

**History**:
- 2020-03-02: initial version by Nikodemus

## Problem Description

I want Foolang to be able to express tight foreign ABI compatible memory
layouts, along the lines of:

``` foolang
class Slice { stride::U32, start::U32, size::U32 }
   layout: Packed
   ...
end
```

I want Foolang to be able to express modular arithmetic restricted to certain
width.

All this implies existence of integral types such as `U8` and `I32`.

How should these different integer types interact?

``` foolang
let a::U8 = 2.
let b::I32 = 40.
a + b --> ?
```

Is the above operation legal? If if not, what about`I8 + I32`? What is the type
of the result? What is the _class_ of the result?

Are these width specified integer types classes or other kind of types?

### Decision Drivers

- Consistency and ease of understanding.
- Matching mathematical conventions.
- Ability for compiler to generate good code.
- Ability to express storage constraints for typed slots.
- Ability to express "modular arithmetic here, please".

## Proposal

This proposal follows Common Lisp's lead.

Modular arithmetic will be expressed using specific messages, eg. `add32:`
instead of vanilla binary arithmetic operators, so that `a + b` always returns a
result consistent with Peano arithmetic. Specific messages used to implement
modular arithmetic are no part of this proposal.

`Integer` is an interface, and there are two implementing classes: `Int` and
`BigInt`, with the first representing all signed integers that fit into a
machine word.

The interface and these classes support expressing intervals as subtypes:

`Integer above: 0` is the type representing all integers above zero.

`Integer from: 1 to: 10` is the type representing integers from 1 to 10. 

Named integer subtypes such as `I8` are simply names given to intervals. This
makes it clear that addition of `U8` and `I32` is just a regular integer
addition.

One of the major downsides is that `U64` is potentially a BigInt.

Arithmetic is implemented through triple dispatch: `+` -> `addInteger:` ->
`addInt:` | `addBigInt:`. This way classes which want to participate in
arithmetic with integers only need to implement `addInteger`. (Note: possibly
consider adding an `addNumber:` dispatch before `addInteger:` to even further
reduce the burden of non-numbers participating in arithmetic.)

Therefore `Integer + Integer` inlines as follows:

```
    a * b
==> b addInteger: a
==> Typecase for: b
      if: Int then: { a addInt: b }
      if: BigInt then: { BigInt add: a to: b }
==> Typecase for: b
      if: Int then: { Typecase for: a
                        if: Int then: { Int add: a to: b }
                        if: BigInt then: { BigInt add: a to: b } }
      if: BigInt then: { BigInt add: a to: b }
```

This in turn should linearize into something along the lines of:

```
is-int %a
branch if-not: bigint-add-to
is-int %b
branch if-not: bigint-add-to
add %a %b
branch if-overflow: allocate-bigint
```

If argument types are known to be in certain intervals, eg. `I32`, then the
branches can be eliminated. To facilitate determining that intervals will
support normal interval arithmetic.

Packaged storage representation can be provided by a protocol:

``` foolang
type I8
   is: Integer from: -128 to: 127.
   method packedType
      Bits size: 8.
   ...
end
```

Packed representations need not be used unless specifically requested via
something like the `layout: Packed` directive in the problem description.

### Summary

This proposal takes a well-known approach to Integer arithmetic. Experience
with Common Lisp's stategy has been generally positive, with minor caveats
related to expressibility of modular arithmetic and storage representation,
which this proposal tries to address.

#### Safety

No impact.

#### Ergonomics

Good.

#### Uniformity

Good.

#### Implementation

Mixed: basic implementation in terms of integer classes required this is quite
simple. Complete implementation in terms of having the interval arithmetic and
packing protocol and bit-classes is not quite so simple.

Getting good performance requires an inlining compiler with good ability
to eliminate branches based on type information, or careful use of modular
arithmetic messages by the user.

#### Users

No users, no impact.

## Alternatives

- Represent integer subtypes like `I8` as distinct classes. Storage would
  be very straightforward.

  Suboptions:

  - Only allow arithmetic within the same type. This would be the natural way to
    implement modular arithmetic via `+`, and be very easy to compile well.
    However, if arithmetic follows mathematical convention the result becomes
    always `TypeUnion of: <this> or: <next-widest-type>`, which is not much more
    useful than plain `Integer`. The ergonomics are also questionable, as
    questions like "what is the type of a literal 1" suddenly become hard.

  - Allow mixed type arithmetic. This solves the "what is type of literal 1"
    problem partially, but not fully. If 1 is `I8`, can it be passed to a method
    that requires an `I32`? The ergnomic answer then implies automatic coercion
    between types. Contagion rules are needed are needed, dispatch is hairier,
    and result type problem is the same as above.

## Implementation Notes

Not implemented.

## Discussion

None.
