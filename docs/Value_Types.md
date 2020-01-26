# Values Types

## Motivation

The main motivation for values type is efficiency. If all user
defined types are reference types then each layer of composition
introduces a level of indirection.

Given value types and appropriate type annotations arbitrarily
deep levels of composition can be used without indirection.

## UX Sketch

Value tag to a class defines it as a value type.

    @class Point { x <Int>, y <Int> }
        value

Annotating a slot as containing a value type member reserves
space for the whole value in the class:

    @class Other { point <Point> }

If the value of the point slot is needed outside Other (or Point) it
will be copied to heap. Effectively there are two Point-classes:
Point(unknown) and Point(known), the first of which is used when Point
appears in context where it's type is not statically known, and the
second is used when it is known.

## Question

What happens here?

    let a = Point x: 0 y: 0
    let b = a
    a x: 1
    b x == 0 -- assert

### Option 1

Value types are immutable. (So the example is illegal.)

This is somewhat elegant because it means that there are
effectively no semantic differences between value and
reference objects -- only allocation strategy difference
and performance difference.

### Option 2

Value types are copied silently: `let b = a` is a copy.

This implies that all objects need to have a copy-operator, which
just happens to be a no-op for references.

It also implies a lot of unnecessary copies. Though the compiler
can help with that.

### Option 3

Copy on write: `a x: 1` causes `a` to be copied. This requires
methods to have access to the location where the instance is
stored. In case of the Point(known) nothing needs to be done,
in case of Point(unknown) the write causes a copy.

## Tentative Conclusion

Since option 1 can be later extended to 2 or 3, it seems best
to start with it: the split representations need to be handled,
but there is no need to deal with implicit copies except in
the case of moving a known value to an unknown context.

## Notes

- In principle the decision to use inline allocation could
  be moved to the client class: { point <inline:Point> }
  or similar.
