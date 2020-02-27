# Tower of Babel

**Status**: WIP

**Identifier**: wip-twower-of-babel

**References**:
- [002 - No Class Inheritance](design/002-no-class-inheritance.md)

**Prior Art**:
- [Spec-ulation](https://github.com/matthiasn/talk-transcripts/blob/master/Hickey_Rich/Spec_ulation.md), talk by Rich Hickey

**History**:
- 2020-02-25: initial version by nikodemus

## Problem Description

To be a real language Foolang would need to have a "cargo-like" package
management system and "crates.io-like" package repository.

Ideally they should co-operate in such a manner that users can fearlessly update
their dependencies: ideally new versions should never break dependents, and
breaking changes should be relegated to a renamed package. (This is probably too
high a bar, but let's accept it as a point of discussion for now.)
    
See the _Spec-ulation_ talk transcript in prior art for section
context&mdash;this problem description follows fairly directly from thinking
about that.

One of the things Hickey doesn't really get to in that talk is how language
features impact this, but they very much do.

This problem description enumerates different types of changes that can be made
to Foolang code, and how likely they are to break dependents depending on
existence of specific language features. Making such an enumeration obviously
requires making assumption about the design space to avoid considering all
possible permutations of features. If the design space changes over time this
note should be retired and replaced.

The proposed solution should provide a stance to various language features, and
a sketch of how a future package management system could facilitate fearless
updates.

### Terminology

- **Safe** change is one that cannot break existing code, ever.
- **Nice** change is one that cannot break "well-behaved" existing code.
  (This definition is loose, but "ill-behaviour" typically involves in things
  like handling a TypeError or MessageNotUnderstoodError to implement something.)
- **Wrought** change is one that depends on details of the implementation. A
  carefully considered wrougth change could be a safe change, and a badly done
  could be a breaking change, and the compiler cannot tell which is which. that
  it would be reasonable to expect it to be writen in a safe or nice way, but
  the compiler cannot with reasonable effort tell.
- **Obvious conflict risks** are changes which may cause name conflicts
  in existing code, that the compiler should be able to trivially detect.
- **Subtle conflict risks** are changes which may cause a name conflicts
  in existing code, but this conflict might not be obvious to the compiler.
  (Not necessarily intractable, but requiring far more analysis then an obvious
  conflict.)
- **Breaking changes** are changes that will likely break well-behaved dependents.
- **Performance breaker** is a change that may cause severe performance
  degradation outside the changed code by preventing critical optimizations
  which could be previously done.

### Taxonomy of Changes

#### 1. **Removing a global name**

Applies equally to modules, globals, and methods.

Always a _BREAKING CHANGE_.

#### 2. Renaming a global name

Applies equally to modules, globals, and methods.

Always a _BREAKING CHANGE_.

#### 3. Adding a global name

A global can be a class, interface, constant, etc.

_SAFE_ if there are no wildcard imports.

_OBVIOUS CONFLICT RISK_ if wildcard imports are allowed.

#### 4. Adding a method to a concrete class

_NICE_ if the class cannot be inherited from and if the class previously
responsed only to statically known messages, ie. it has no delegates or
wildcard message handlers.
    
_SUBTLE CONFLICT RISK_ if the class can be inherited from, or responds
to arbitrary messages.
    
#### 5. Adding a provided method to an abstract interface

_SUBTLE CONFLICT RISK_ since classes implementing this interface may
already respond to the message but not implement the expected behaviour.

#### 6. Adding a required method to an abstract interface

_SUBTLE CONFLICT RISK_ since classes implementing this interface may
already respond to the message but not implement the expected behaviour.

_BREAKING CHANGE_ since classes implementing this interface that do not
provide method with the given name will no longer be valid implementations
of the interface.

#### 7. Changing a method's implementation

_WROUGHT_ since the change may be a bugfix that only makes broken code now
work, but it may also change behaviour that dependents depended on.

#### 8. Widening a slot's declared type

_NICE_ since only way that dependent code can tell is if they previously
provoked a type error.

#### 9. Narrowing a slot's declared type

_WROUGHT_ since it is reasonable to expect that the change could be made in a
manner completely invisible to dependent code, but proving that this is the case
is hard if not impossible.

#### 10. Widening a method argument's declared type

_SAFE_, but may be a _PERFORMANCE BREAKER_ if the compiler utilizes
non-local type information and succesful message sends to derive types, ie.
using `y messageWithDeclaredArgType: x` to derive `x`'s type. 

#### 11. Narrowing a method argument's declared type

_WROUGHT_ since the code might have previously already only worked with
objects of the newly declared type, but it is also easy to imagine a case where
the code worked previously for a wide array of types, but now does not.

#### 12. Widening a method's declared return type

_SAFE_, but may be a _PERFORMANCE BREAKER_ if the compiler compiler utilizes
non-local type information.

#### 13. Narrowing a method's declared return type

_WROUGHT_ since code might have previously already returned objects of
exactly that type, but this is not necessarily the case and might cause type
errors in user code that previously worked.

#### 14. Implementing an interface in a concrete class

The "new methods become available" aspect is effectively same as [4. Adding a
method to a concrete class](#_4-adding-a-method-to-a-concrete-class), so: _NICE_
or _SUBTLE CONFLICT RISK_ depending on inheritance and wildcard dispatch.

The "member of a new type" aspect depends on if object's abstract type can be
tested without relying on type errors or reflection (including ). If type can be
tested it is _BREAKING CHANGE_ since there are so many ways dependent code could
assume things that are no longer true, eg:

- A tree a is implemented as collections containing other collections
  or objects of class X or Y (which are not collections).
- Visiting a node tests if the node is a collection or not, and
  either recurses or does the visit.
- Class X becomes a collection, and tree visiting code stops working.

Note: Testing the concrete type for identity is non-problematic. Only asking "is
this object a collection" is what causes trouble.

Note: The "member of a new type" aspect is non-problematic if the type itself is
new, ie. if it did not exist previously, regardless of ability to test abstract
types.

#### 15. Implementing an interface in an abstract interface

The "new methods become available" aspect is effectively same as [5. Adding a
provided method to an abstract
interface](#_5-adding-a-provided-method-to-an-abstract-interface), so: a _SUBTLE CONFLICT RISK_.

The "member of a new type" aspect works similarly to : a _BREAKING CHANGE_ if abstract types
can be tested and the implemented interface was a previously existing one.

#### 16. Moving a method's implementation to existing interface

Effectively same as [5. Adding a provided method to an abstract
interface](#_5-adding-a-provided-method-to-an-abstract-interface), so: a _SUBTLE
CONFLICT RISK_.

#### 17. Moving a method's implementation to new interface

Effectively the same as adding a global name, so _SAFE_ if there are
no wildcard imports, and a _BREAKING CHANGE_ if they are allowed.

#### 18. Adding a slot to a class, removing a slot from a class

Effectively the same as **Changing a method's implementation**, so
_WROUGHT_.

#### 19. Globally extending classes defined in other modules

This is an _OBVIOUS CONFLICT RISK_.

## Proposal

- Strong "no inheritance of concrete classes" position.
- Explicit module exports
- Explicit experimental markers
- Consider a "no tests for abstract types" position: it would decrease
  uniformity of the language, but allow more changes to be non-breaking.
- Condider semantic disctinction between Exceptions and Errors: "handling errors
  is nasty" -- so using `{ x::Collection. True } onError: { False }` to
  implement a type-test would be nasty. (Or maybe call them panics?)
- Consider adding _reserved methods_ to interfaces, meaning: not yet part of the
  interface, but will be, so implementing classes are not allowed to have one.

### Summary

None as of yet.

#### Safety

No impact.

#### Ergonomics

No impact.

#### Uniformity

No impact.

#### Implementation

TODO

#### Users

TODO

## Alternatives

TODO

## Implementation Notes

None.

## Discussion

None.
