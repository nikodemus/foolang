# No Class Inheritance

**Status**: ADOPTED (implemented)

**Identifier**: 002-no-class-inheritance.md 

**References**:
- Rust's structs vs traits
- Julia's concrete types may not subtype each other

**History**:
- 2019-01-01: arbitrarily backdated by nikodemus, since this is an old
  design decision.
- 2020-02-25: initial version of this writeup by nikodemus
- 2021-01-05: updated to current format

## Problem Description

### Part 1: Interprodecural Optimizations

Consider an expression like the following, situated inside a hot loop:

```
object::ConcreteType message: arg
```

Users should be able to rely on this being "compiled well", applying
interprocedural optimizations. (Eg. possibly constant-folding the whole
expression if `arg` happens to be a constant and the method allowing it.)

For the compiler to be reliably able to do so, it must know the exact method
that implements `#message:`&mdash;static interprocedural optimizations are
inhibited if `ConcreteType` has subclasses that override the method.

In particular, adding a new subclass of `ConcreteType` can easily inhibit
optimizations that previously worked, causing surprising performance degradation
due to apparently unrelated changes.

For performance to be reliable and transparent both user's and compiler's views
must coincide without significant extra effort on the user's part.

### Part 2: Representation Selection

Consider a concrete class which contains a single read-only slot with a 32-bit
integer, and code which annotates an object as being of that type.

If the compiler can know that this is the only representation instances
satisfying type A can have, the object can be represented as a single "naked"
32-bit integer in register/where-ever.

If there is also a subclass B of A which adds more slots, the type declaration
is no longer sufficient information to allow this optimization.

Similar arguments to transparency and predictability as for the interprocedural
optimization part above apply here.

### Drivers

- Predictable performance: ability to predict reliably when interprocedutal
  optimizations can be applied, ability to predict reliably when a compact
  non-heap representation can be used.

- Reasonable implementation overhead: "sufficiently smart compilers" need not
  apply, and are antagonistic to predictable performance anyhow.

## Proposal

Concrete instantiable classes cannot be subclassed, or their methods overridden
in instances.

The user's cognitive burden is reduced to asking "is the class here concrete",
if the answer is "yes", then they can expect "reasonably cood code" out of the
compiler.

!> This proposal doesn't ask for indentifier syntax to distinguish concrete
class names from abstract interface names, so the possibility of someone
changing a class to an interface remains. Such an syntax is an interesting
option to explore at a later junction, particularly if tied to type annotations
instead of the names themselves.

### Summary

Disallowing concrete class inheritance makes it feasible for user to maintain a
mental model of "will the compiler be able to do much here", leading to more
predictable performance.

#### Safety

No impact.

#### Ergonomics

Mixed impact: easier to maintain a mental model, but slightly reduced
flexibility as useful concrete classes cannot be subclassed.

#### Performance

Positive impact: easier to optimize message sends when type of the receiver is
known.

#### Uniformity

No impact.

#### Implementation

Easier to implement optimizations than the alternative.

#### Users

Mixed impact as per ergonomics and performance.

Possible future relaxation of this restriction seems backwards compatible
in principle, but likely to have many adverse effects in practise without
significant amounts of extra care.

## Alternatives

- **Allowing subclasses as method extensions only**: Ie. prohibit adding slots
  or overriding methods. This provides exactly the same ebenefits as the
  proposal, but makes for a slightly more complex language and implementation.
  This seems like a reasonable alternative or extension to the proposed design.

- **Allow Subclassing**: Optimization decisions need to check if the method is
  overriden in a subclass, and if subclasses add slots. Not difficult per se,
  but harder for users to track, and brittle in face of changes.

- **C++ style `final` method qualifiers**: they allow the user to communicate
  "this will not change due to another class" at method granularity. The end
  result seems undesirable, however: there is a temptation to "let's make this
  one not final so I can subclass"-type decisions, and making the annotation is
  optional so it is easy to forget. Does not answer the issue of
  representations.

- **Method qualifiers allowing overrides in subclass**: similar to `final`, but
  opposite. Seems like a better default, but does not answer the issue of
  representations either.

## Implementation Notes

This is currently implemented, though there is no compiler to take advantage of
it.

## Discussion

None.
