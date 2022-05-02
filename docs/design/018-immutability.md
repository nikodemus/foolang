# Immutability

**Status**: ADOPTED (not implemented)

**Identifier**: 018-immutability

**References**:
- None

**History**:
- 2022-05-02: initial version by Nikodemus

## Problem Description

The interactive environment must be able to transition objects from mutable to
immutable in order to create new globals. Immutability must apply to whole
subgraph: any object reachable from an immutable object must be immutable.

It would be beneficial for the user to be able to also make objects immutable,
not only to ensure correctness, but to allow sharing between actors.

It would be beneficial to be able to mutate immutable objects in a debugger /
interactive environment. (Eg. changing a global while the system is running.)

This needs to be implementable in a C backend, ideally with no platform specific
code.

## Proposal

Maintain a mutable / immutable bit in allocation header for all non-immediate
objects, and check it on writes. On modern CPUs branches are generally speaking
very cheap. Experience with dynamic typechecking speaks to this, so the expected
cost of a well predicted immutability check is minimal. The GC will also likely
use a write barrier to maintain a remembered set, so the same barrier can do
both jobs.

Provide a privileged method accessible through `System` which makes an entire
subgraph immutable. Possibly `System#makeDeeplyImmutable:`. This allows making
globals immutable in the interactive environment.

Provide a built-in method `Object#makeImmutable` which checks if all member of
the object are immutable, and if so makes the object immutable, or raises an
error if some member is mutable.

Provide a built-in method `Object#isImmutable` for checking immutability of an object.

Provide a privileged method accessible through `System` for mutating an
immutable object. Possibly `Debugger#setSlot:in:`, raising a continuable error
on encountering an ummutable object.

### Summary

This design appears to satisfy all criteria, while maintaining language uniformity.

#### Safety

#### Ergonomics

Minial impact.

Ability to make existing objects immutable without the class exposing that
functionality seems a bit ugly, though. Possibly `#makeImmutable` should be a
private method, which objects can expose if needed?

#### Performance

Minimal impact, hopefully.

#### Uniformity

Positive impact: instead of there being built-in magic for making globals immutable
there's `System#makeDeeplyImmutable:`.

#### Implementation

No real impact. This needs a write barrier, but that is needed anyhow. As long
as `#makeImmutable` is public there is no need for new message concepts either.

#### Users

No users, no impact.

## Alternatives

Separate memory pages make write-protected is another option, but requires platform
specific code, and is more complex to implement, and makes ignoring immutability
of specific object hard.

Instead of `#makeImmutable` there could also be a built-in interface `Immutable`
which would make an object immutable on construction.

## Implementation Notes

None.

## Discussion

None.
