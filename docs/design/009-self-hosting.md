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

Rough sketch of options:

- C calling convention, using fixed or varargs as appropriate. Very little
  flexibility, live debugging hard. Cost: low.

- Separate data stack. Live debugging much easier than with C calling
  convention. With enough work also allows materialization of contexts. Cost:
  medium.

- Explict heap allocated contexts. This is the most flexible solution, but even
  then all of classic Smalltalk context trickery will remain out of reach unless
  execution is also trampolined. Cost: high.

Compromise:

1. Strict methods (methods declaring that at least one of their arguments is an
   instance of specific class) are implemented in two parts: external entry
   point using a data-stack, and internal entry point which pulls arguments from
   the stack, typechecks and unboxes the ones with class types, and calls the
   internal entry point using C-convention, finally boxing up the return value
   if it's class was declared. The internal entry point uses C calling
   convention.

   When a strict method has a known send to it, it is compiled directly using
   the internal entry point.

2. Non-strict methods do not have an internal entry point, and directly use
   the data stack for both arguments and local variables.

Backtraces can be generated for both with with `libunwind` / Windows'
`CaptureStackBackTrace`, and C names demangled/mapped into class and selector
names, and external/internal interface frames merged.

For internal entry point frames arguments and variables will no be accessible.

For external entry point frames it should be possible to identify the section of
the data stack and the associated argument and variable names.

Full sends can use `setjmp/longjmp` to provide both restartability and debugger
return capability, as per following sketch:

```
foo_send(struct FooStack* stack) {
  restart:
    jmp_buf control;
    switch (setjmp(control)) {
        case SETJMP_INIT:
            return foo_send_internal(&control, stack);
        case SETJMP_RETURN:
            return stack->unwind_return_value;
        case SETJMP_RESTART:
            goto restart;
    }
}
```

## Discussion

None.
