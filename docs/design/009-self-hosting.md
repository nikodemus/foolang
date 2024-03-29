# Self Hosting

**Status**: ADOPTED (mostly implemented)

**Identifier**: 009-self-hosting

**References**:
- Back to the Future&mdash;The Story of Squeak, A Practical Smalltalk Written in
  Itself. http://files.squeak.org/docs/OOPSLA.Squeak.html

**History**:
- 2020-08-06: initial version by Nikodemus&mdash;this has been cooking
  a lot longer, but time to put it here.
- 2021-01-05: updated to current format
- 2021-07-30: updated to current status

## Problem Description

Self hosting is a desirable property for multiple reasons.

How to get there?

## Proposal

1. Bootstrap interpeter.
2. Self-hosted parser & AST interpreter.
3. Self-hosted trivial C transpiler.
4. Self-hosted optimizing C transpiler.

Bootstap interpreter needs to be good enough to interpret the self hosted code,
but no more.

The transpiler can initially operate on AST directly.

Transpile the self-hosted code into source for a `foo` executable capable of
acting as both an interpreter and compiler, replacing the bootstrap interpreter.

C selected as initial transpilation target for compilation speed and ease of
type-punning.

### Summary

#### Safety

No impact. (Using C as transpilation target could be considered to be a possible
safety impact, but later AOT compiler will inevitably suffer from the same lack
memory safety, so avoiding C here buys no long term safety.)

#### Ergonomics

No impact.

#### Performance

Positive impact: transpiled code with type annotations should be "pretty
decent", and transpilation will expose any "this is hard to compile well"
issues with the language design sooner than later.

#### Uniformity

No impact / positive impact. Self-hosting tends to make non-uniformities more
explicit, and often shows a way past them.

#### Implementation

Mixed impact. Until the self-hosted implementation is complete there will be two
parallel implementations to maintain. Once it's done, the work should be reduced
and maintenance of self-hosted implementations is generally less work.

#### Users

No users, no impact. Some of the toys like the OpenGL stuff that was an early part of
the bootstrap interpreter had to go, though.

## Alternatives

- Not doing full self hosting. Meh: looking at Factor and Smalltalk in
  particular in comparison to eg. Julia the benefits of self-hosting seem
  self-evident to me.
- VM-first strategy. Would require a bootstrap VM in addition to a self-hosted
  VM, or giving up on full self-hosting.
- Exposing the bootstrap interpreter's AST to Foolang code. Extra complexity
  and the self-hosted parser still needs to be written eventually.

## Implementation Notes

Status: basic self-hosting complete: the compiler can compile itself. Planned
`foo` executable not done yet, bootstrap evaluator remains. Optimizations not
started,

Classic basic structure:

1. Parser produces a CST, which can be pretty printed with fidelity.
2. Syntax translator walks the CST to produce an AST which has all names resolved.
3. Interpreter and transpiler work by walking the AST.

Both syntax translator and interpreter are structured as visitors.

Initial optimizations will operate on the AST.

### Transpiled code

Current implementation uses heap allocated context, with methods taking
only a single argument - a context allocated for the sender by the caller,
containing the arguments and the receiver, and linking to previous context.

Plan is to change this, however: methods will in next iteration receive context
of the caller, and arguments as C arguments. Caller must guarantee that the
arguments already exist either in its frame, or an earlier frame: the current
context is used as the GC root.

This will allow builtin methods to mostly not allocate a context at all,
and will in remove the need to duplicate arguments in two contexts.

Additionally, trivial method which only perform a single send can also elide
allocating a new context.

Backtraces are currently generated by walking over the contexts.

Non-local returns use `setjmp/unwind` with explicit cleanup of contexts
before the jump is done.

## Discussion

None.
