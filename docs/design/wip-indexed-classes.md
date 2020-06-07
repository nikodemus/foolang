# Indexed Classes

**Status**: WIP (not implemented)

**Identifier**: wip-indexed-classes

**References**: none

**Prior Art**:
- Smalltalk indexed instance variables
- C structs with tailing arrays

**History**:
- 2020-03-21: initial version by Nikodemus
- 2020-06-06: list of alternatives by Nikodemus

## Problem Description

In current Foolang collections need to be implemented on top of collection
primitives like Array and Dictionary.

It would be preferable to be able to implement these on top of a more primitive
fixed allocation data structure.

## Proposal

Sketch:

```
class Foo { <slots>, <indexed>... }
  ...
end

class Foo { <slots>, <indexed>::<Type>... }
  ...
end

Foo <slot-initializers> <indexed>With: elt1 with: elt2 ...

Foo <slot-initializers> <indexed>From: collection

Foo <slot-initializers> <indexed>: size
Foo <slot-initializers> <indexed>: size value: initial-value
```

The name used for `<indexed>` is used to access the indexed part of the class as
if it was a collection. As a special case, if `<indexed>` is `*` collection
methods of the instances directly access the indexed part, and the constructor
fragments don't have a prefix, and start with lowercase letters.

Allowing references to `<indexed>` reifies it as a special object, allowing
direct access to it as if a collection. (XXX: not collection, since cannot add
or remove items.)

### Summary

TODO: Why this proposal is a good idea.

#### Safety

No safety impact.

#### Ergonomics

TODO: Impact on ergonomics & (user) aesthetics.

#### Performance

TODO: Impact on performance, including impact on future compiler optimizations.

#### Uniformity

Improved uniformity.

#### Implementation

TODO: Impact on implementation: complexity, amount of work, maintenance burden, etc.

#### Users

No users, no impact.

## Alternatives

- Moral equivalent of C++ `new[]`, allowing allocation of blocks of arbitrary
  type. This seems like a complementary feature more than an alternative -- both
  can replace the other in terms of what can be done, but what is convenient is
  different.
- Allowing arbitrary number of indexed slots per class. This would be nice, but
  complicates instance access more than I consider desirable right now - and
  the uses cases are a bit rare.
- Not naming the slot, but automatically providing the collection-like methods
  for the class. (Also, no reification of the slot.)

Syntax variations are a dime a dozen. The current one is almost certainly not
the final one.

## Implementation Notes

Current implementation status and related notes.

## Discussion

Typically amended as design note moves to a different stage, especially
when _REJECTED_.
