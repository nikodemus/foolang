# Indexed Classes

**Status**: WIP (not implemented)

**Identifier**: wip-indexed-classes

**References**: none

**Prior Art**:
- Smalltalk indexed instance variables
- C structs with tailing arrays
- Self?

**History**:
- 2020-03-21: initial version by Nikodemus

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
direct access to it as if a collection.

### Summary

Why this proposal is a good idea.

#### Safety

Impact on safety: memory safety, concurrency, authority, fault tolerance, etc.

#### Ergonomics

Impact on ergonomics & (user) aesthetics.

#### Uniformity

Impact on performance, including impact on future compiler optimizations.

#### Implementation

Impact on implementation: complexity, amount of work, maintenance burden, etc.

#### Users

Impact on users, including backwards compatibility.

## Alternatives

Alternatives to the proposal that were considered.

## Implementation Notes

Current implementation status and related notes.

## Discussion

Typically amended as design note moves to a different stage, especially
when _REJECTED_.