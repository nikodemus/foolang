# Indexed Classes

**Status**: ADOPTED (not implemented)

**Identifier**: 008-indexed-classes

**References**:
- Smalltalk indexed instance variables
- C structs with tailing arrays

**History**:
- 2020-03-21: initial version by Nikodemus
- 2020-04-15: primitive arrays initially approach
- 2020-07-11: adoption
- 2021-01-05: updated to current format

## Problem Description

In current Foolang collections need to be implemented on top of collection
primitives like Array and Dictionary.

It would be preferable to be able to implement these on top of a more primitive
fixed allocation data structure.

### Options

- Smalltalk-like indexed classes, supporting `#resize:` message. (Out of line
  allocation.)

- Indexed classes similar to C structs with tailing arrays cannot be resized.
  (Inline allocation.)

- Unresizable "array-like" primitive class, supporting specialization via
  `#of:size:` -style message.

?> Block allocations are something I'm also interested in pursuing, but not the
same thing: a collection primitive needs the ability to have it's contents
replaced, which a block allocation cannot do (or at least does not need.)

### Analysis

Out of line allocation forces memory indirection, which is not acceptable for
the primitive.

C-like solution has the nice feature that eg. simple fixed capacity buffers can
carry their current size inline, without indirection.

Unresizable array primitive is a generic solution: most of the time that is all
that is wanted, and without parametric types the allocation efficiency of a
typed array cannot be duplicated by a user-implemented class.

Naming issues:

- For general programming a name for an extensible linearly allocated
  sequence is required. List, vector, and array are commonplace names.

- For mathematics vector and matrix are the commonplace, and tensor or
  ndarray for higher or arbitrary dimensions.

## Proposal

1. Initially implement primitive specializable arrays: they are what is needed
   for the current use-cases, and allows constructors of other classes to take
   type parameters sensibly, in a way that can actually provide specialized
   storage.

2. When use-case for indexed classes appears where they offer a benefit over
   arrays, implement the C-style solution.

3. When parametric classes are supported primitive specializable arrays can be
   implemented as a parametrized indexed class. (We could implement Array on top
   of indexed classes just fine even without parametric types, but to gain
   specialized storage parametric types are needed.)

Naming:

- `Array` is the primitive specializable fixed-length class.
- `List` is the general programming class.
- `Vector`, `Matrix`, `Tensor` are the mathematical interfaces.

`Array#of:` creates returns an object that functions both as a specialized
constructor and type, so that:

```
define Floats
    Array of: Float
end

method something: data::Floats
    ...
```

works. (Later supporting a type specifier expression like `::(Array of: Float)`
seems reasonable, but is not needed immediately.)

Syntax:

- `[]` creates an Array of Any.

### Summary

Staged implementation of more general facilities in order which is convienient
is a nice balance between immediate benefits and end results.

#### Safety

No impact.

#### Ergonomics

Decent: `[]` could just as well create a `List`, but is seems like immutable
size is the more common case for literal objects, where as extensible ones
typically start out empty. Can further be improved by syntax extensibility.

#### Uniformity

Improved, as it becomes feasible to implement `List` in Foolang with O(1)
performance, whereas without this the corrsponding class needs to be a builtin.

#### Implementation

Small impact.

- In the bootstrap evaluator the class previously called `Array` gains a
  type parameter and loses most of its methods.

- `List` implemented in prelude, most of methods migrated from old `Array`.

#### Users

No users, no impact.

## Alternatives

See Options above.

## Implementation Notes

None as of yet.

## Discussion

None as of yet.
