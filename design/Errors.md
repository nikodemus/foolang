# Errors

## Model

A panic aborts execution and cannot be caught. The thread of
execution is immediately abandoned without any (user-level)
cleanups. (In development mode you do get a debugger, etc,
though.)

Message sends can result in errors in addition to values.
(Errors are not panics.) Only two things can be done with
error results:

- Propagate it upwards.

- Handle it (aka detect it aka convert it to an object).

Doing anything else with an error will result in panic:

- Trying to store an error in a variable without handling it
  will panic.

- Trying to ignore an error without handling it will panic.

Ie. if `foo bar` causes an error, this will panic because
it is not handled.

```
     foo bar, quux bar
```

Easiest way to handle it is to propagate it:

```
foo bar?, quux bar
```

Handling it converts it to object:

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
