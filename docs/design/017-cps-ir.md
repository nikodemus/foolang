# CPS IR

**Status**: WIP (implementation in progress)

**Identifier**: cps-ir

**References**:
- https://anydsl.github.io/Thorin.html

**History**:
- 2022-04-24: initial version by Nikodemus

## Problem Description

Foolang doesn't currently have an IR besides the AST, and the AST itself has
several issues -- and the compiler really needs one.

### Decision Drivers

- Having a reasonable amount of applicable literature.
- Excellent support for inlining.
- Easy translation to C / LLVM IR / bytecode.
- Something I feel comfortable with.

## Proposal

Continuation Passing -style IR in the Sea-of-Nodes mode, mainly modelled after
Thorin, with some inspiration from Guile.

### CPS Expression

Is one of:
- Constant
- Continuation
- Global
- Primop
- Variable

### Constant

Has:
- ID: unique numeric identifier.
- VALUE: Value of the constant.
- USES: set of CPS expressions which use the constant.

### Continuation

Has:
- ID: unique numeric identifier.
- NAME: either user-provided name, or optionally a debug name
  prefixed by `$`.
- PARAMS: array of _Variables_ bound by this continuation. Usable
  by the continuation itself or its successors.
- ARGS: array of _Continuation Expressions_ evaluated in order,
  whose values are used with the target.
- TARGET: a Continuation Target, one of:
  - Continuation
  - Variable
  - Application
  - Select
- USES: set of CPS expressions which use the continuation.

### Global

Has:
- ID: unique numeric identifier.
- NAME: user-provided name of the global.
- USES: set of CPS expressions which use the global.

### Primop

- ID: unique numeric identifier
- KIND: one of:
  - ADDI, sum of the two integer arguments. Foldable.
  - CLASS\_OF, class of the argument. Foldable. Value usually used by
    Primop with kind FIND\_METHOD.
  - DATUM_OF, returns the datum of an instance argument. Foldable.
  - FIND_METHOD, finds the method function for the given selector in
    the given class. Foldable. Value usually used by Application.
  - INVALID, used to mark deleted primops.
  - MAKE\_BOX, boxes a value for an assignable variable.
  - MAKE\_INSTANCE, creates an instance value from a type and datum.
  - SET_VALUE, changes the value of a boxed variable.
  - VALUE\_OF, returns value of the a boxed variable.
- ARGS: array of CPS Expressions consumed by this primop.
- USES: set of CPS expressions which use the primop.

!> More primitive operation kinds like ADDI to be defined. (ADDF, MULI, MULF...)

### Variable

Has:
- ID: unique numeric identifier.
- NAME: user-provided name of the global.
- DEFS: set of CPS expressions which provide a value for this variable
- USES: set of CPS expressions which use the variable.

### Summary

This representation should allow me to apply the lambda-mangling
algorithm from the Thorin paper reasoanbly, as well as do other
optimizations.

#### Safety

No impact.

#### Ergonomics

No impact. Well, internally this feels pretty nice.

#### Performance

Great positive impact when fully integrated.

#### Uniformity

No impact as specified here. However, the current implementation
allows a fairly easy way to specify custom lowerings, which could
be potentially exposed as part of the language -- positive in that
case.

#### Implementation

Implementation seems fairly straightforward so far, but of course
any graph IR has a somewhat higher complexity than an AST.

Then again, the C-emitter for this IR should be a lot simpler
than the one for the AST.

#### Users

No users, no impact.

## Alternatives

SSA. The problem with SSA is that it doesn't really support
HoFs as well -- or at least I don't see how to make that happen.
The upside with SSA is that there's way more litetature.

...but CPS isn't a dead land, and these days there's a bunch
of papers on how to apply SSA algorithms in CPS.

## Implementation Notes

Code in foo/impl/cps.foo. Zero integration at the moment.

## Discussion

None.
