# Self Hosting

**Status**: WIP (not implemented)

**Identifier**: 009-self-hosting

**Prior Art**:
- Back to the Future&mdash;The Story of Squeak, A Practical Smalltalk Written in
  Itself. http://files.squeak.org/docs/OOPSLA.Squeak.html

**History**:
- 2020-08-06: initial version by Nikodemus&mdash;this has been cooking
  a lot longer, but time to put it here.

## Problem Description

Self hosting is a desirable property for multiple reasons.

How to get there?

## Proposal

1. Bootstrap interpeter.
2. Self-hosted parser & AST interpreter.
3. Self-hosted C transpiler.
4. Other transpilers.

Bootstap interpreter needs to be good enough to interpret the self hosted code,
but no more.

The transpiler could operate on AST directly, but a few straigtforward passes
are probably in order. (Inlining / partial evaluation, typechecking,
devirtualization, lambda-lifting.)

Transpile the self-hosted code into source for a `foo` executable capable of
acting as both an interpreter and compiler, replacing the bootstrap interpreter.

(C selected as initial transpilation target for compilation speed and ease of
type-punning.)

Later given the existing C transpiler, adding transpilers targeting Emacs Lisp
and Javascript should be a quick job: these will allow seamless editor
integration, use of self-hosted pretty printer for indentation, and an
in-browser web-repl.

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

No users, no impact. Some of the toys like the OpenGL stuff currently part of
the bootstrap interpreter will go away, though.

## Alternatives

- Not doing full self hosting. Meh: looking at Factor and Smalltalk in
  particular in comparison to eg. Julia the benefits of self-hosting seem
  self-evident to me.
- VM-first strategy. Would require a bootstrap VM in addition to a self-hosted
  VM, or giving up on full self-hosting.
- Exposing the bootstrap interpreter's AST to Foolang code. Extra complexity
  and the self-hosted parser still needs to be written eventually.

## Implementation Notes

Currently work in progress, classic basic structure:

1. Parser produces a CST, which can be pretty printed with fidelity.
2. Syntax translator produces an AST from the CST.
3. Interpreter walks the AST.

Both syntax translator and interpreter are structured as visitors.

For transpilation rewriting visitors will transmute AST into successively
refined IRs in micro-passes.

### Transpiled code

Tradeoffs abound, mostly debugging and ease of implmentation vs speed.

In order to be able to eyeball the performance we're getting usefully the
C code should use native stack and calling conventions, including XEPs and IEPs:

```
struct FOO* foo_Integer_method_addInteger_xep(struct Foo* receiver, int nargs, ...) {
    va_list args;
    va_start(args, nargs);
    /* IEP inlined since it's a primitive */
    return FOO(.class = &FOO_CLASS_Integer,
               .i64 = receiver.datum.i64 + FOO_INTEGER_ARG(args, FOO_SOURCE_INFO_9861));
    /* IEP out of line like it would normally be */
    return FOO(.class = &FOO_CLASS_Integer,
               .i64 = foo_Integer_method_addInteger_iep(receiver.datum.i64,  FOO_INTEGER_ARG(args, FOO_SOURCE_INFO_9861)));
}
```

Mininum requirements for transpiled code:
- Backtrace with classes and selectors.
- Mixing interpreted and transpiled frames.

Backtrace can be generated with `libunwind` / Windows' `CaptureStackBackTrace`,
and C names demangled/mapped into class and selector names.

Access to XEP receiver and arguments is probably doable via frame pointer fairly
easily -- but also noncritical if missed. Access to IEP arguments is clearly
harder, and will be elided for now. Access to local variables is not going to happen.

Restarting transpiled frames might be doable, but is elided.

Initial implementation of non-local returns can use `setjmp/longjmp`.

Consider:

```
method someInterpretedMethod
    array collect: interpretedBlock!
```

If there's an error in interpreted block, backtrace should look like:

```
...
Block#value:
Array#collect:(compiled)
Foo#someInterpretedMethod
```

This will be a bit tricky, but getting it right seems hugely important for
quality of life.

## Discussion

None.
