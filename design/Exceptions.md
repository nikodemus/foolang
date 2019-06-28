# Exceptions

We consider two different but related behaviours: panics
and exceptions.

## Panics

A panic aborts execution and cannot be caught. The thread of
execution is immediately abandoned. (Execution of user-level
cleanups in response to panics is an open question, but the
current though is to now allow this.)

Note: in development mode panics can be debugged, and even
resumed from.

## Errors

Function calls and message sends can result in errors in
addition to returns.

Errors are _not_ panics.

When an error occurs the send of the message or the caller
of the function must either propagate it or handle it.

Failure to do either _will_ cause a panic.

Ie. if `foo bar` causes an error, this will panic because
it is not handled.

```
     foo bar, quux bar
```

To propagate it use the `?` postfix operator: it causes the
error to be propagated if one occurs, and is a no-op in
case of normal return.

```
foo bar?, quux bar
```

The handle the error us `? as infix operator with a block
as the right argument.

```
foo bar ?{ :e | logger logError: e }
quux bar
```

A handled error can be re-signalled as well:

```
foo bar ?{ :a | logger logError: e, error e }
```

## Annotations

Methods can be annotated as no-errors and no-panics and safe.
The compiler will verify that no message send can result in
an error, or fail. Safe = no-errors and no-panics

```
@method foo
    <no-errors>
    thing doStuff
```

The besides local robustness, the hope is that sufficiently
large parts of the library can be annotated as no-errors
so that using exceptions to implement errors becomes cheap.

## Implementation

Simplest thing that could possibly work:

Every message send that doesn't have an explicit check
gets an implicit ?{ panic "Unexpected error: {_}" }

Not fast, but good enough for the evaluator.
