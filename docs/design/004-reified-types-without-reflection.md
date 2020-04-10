# Reified Types Without Reflection

**Status**: ADOPTED

**Identifier**: 004-reified-types-without-reflection

**References**:
- WIP - [Tower of Babel](design/wip-tower-of-babel.md)

**Prior Art**:
- 2004 - [Mirrors: Design Principles for Meta-level Facilities of
  Object-Oriented Programming Languages](https://bracha.org/mirrors.pdf) by
  Gilad Bracha and David Ungar.

**History**:
- 2020-03-01: initial version by Nikodemus

## Problem Description

TL;DR: "Should there be a `classOf` primitive or method?"

I'm reasonably convinced that mirrors are _"the right solution"_ for reflection:
particularly the ability to statically determine the absence of reflection in an
application seems important for the performance goals of Foolang.

Foolang's types are reified: `Integer` is a regular object.

Reified types as such are not contraindicated by mirrors: as long as they don't
provide reflective facilities they're just objects like others. Therefore
mirrors do not prevent addition of `classOf`.

At the same time I've avoided adding a `classOf` builtin or method to push code
towards protocols where type-tests aren't used.

Analysis in [Tower of Babel](design/wip-tower-of-babel.md) also makes clear that
type-tests make some changes which would otherwise not be breaking changes into
breaking changes: implementing pre-existing interfaces in other pre-existing
classes and interfaces can break dependents even if the behaviour of the code
did not change.

Type-tests are however not predicated on the existence of `classOf`, and
leveraging reified types for construction of non-class and non-interface types
like intervals implies that the type-objects themselves are responsible for
type-tests: so even though user-accesible type-tests are not currently part of
the language they are implied by the overall design.

Additionally type-tests **can** already be implemented, even in the absence of
reified types:

``` foolang
class Typecheck {}
   method isFloat: obj
        { obj::Float. True } onPanic: { False }
end
```

Therefore type-tests are already part of the language: `classOf` is not required
for them, and non-addition of `classOf` would not keep them out.

So, given the plan for mirror-based reflection, existence of reified types,
and the need for those reified types to be able to check for membership, should
there be a `classOf`-type primitive or equivalent method?

## Proposal

No. There is no need for it.

Demonstration that user-defined subtypes do not need `classOf`:

``` foolang
class Codepoint { value }
end

class GermanicCodepoint {}
    class method includes: obj
        (Codepoint includes: obj)
            and: (Codepoint germanicRange includes: obj value)
    class method subtypeOf: type
        type is Codepoint
```

Notice the default `#includes:` method on classes: if that did not exist there
would be a need to access the class of an object directly&mdash;at the same time
this limits the ability to test arbitrary types unless they choose to allow
that, providing a light the push towards code that doesn't depend on typechecks.

### Summary

Absent a solid use case there is no need to add `classOf` to the language now.

#### Safety

None.

#### Ergonomics

Minimal. If something becomes harder or impossible without `classOf`, that's
obviously a negative&mdash;and may lead to reconsideration of this position, but
I believe most of the time it is just smelly code.

#### Uniformity

Negative: built-in code definitely needs to be able to access the class of an
object, making this a violation of the Uniformity principle. (This is actually
the strongest argument in favor of `classOf` so far.)

#### Implementation

Positive: one less thing to implement.

#### Users

No users, no impact.

## Alternatives

- Yes, implementing it as a primitive. Chiefly it would allow asking "are these
  two objects members of the same class".

- Yes, implementing it as an overridable method that could lie. This is actually
  somewhat interesting, since it would allow serious mocking&mdash;but it would
  _not_ allow asking the "are these two objects members of the same class" in
  equally strict manner.

## Implementation Notes

None.

## Discussion

None.
