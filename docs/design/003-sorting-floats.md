# Sorting Floats

**Status**: ADOPTED (not implemented)

**Identifier**: 003-sorting-floats

**References**:

- IEEE 754-2008 adds a total ordering predicate, the only definition which
  appears to be currently on the internet is in [the Rust issue
  5585](https://github.com/rust-lang/rust/issues/5585).

- Java defines a similar but somewhat different total order over floats using
  [`Float.compareTo()`](https://docs.oracle.com/javase/8/docs/api/java/lang/Float.html#compareTo-java.lang.Float-)

**History**:
- 2020-02-29: initial version by Nikodemus
- 2021-01-05: updated to current format

## Problem Description

The IEEE 754 arithmetic is defined by people who actually know what they're
doing, and it's by definition how floats should operate.

Therefore floating point methods should follow it. This results in `<=` not
providing a total order.

At the same time, sorting floats is a commonplace activity.

## Proposal

Use `order:` message as part of `Ordered` interface for sorting, returning
`Before`, `Same`, or `After`.

Default implementation can be built on `<=`:

```
method order: other
    self <= other
        ifTrue: { other <= self
                      ifTrue: { Same }
                      ifFalse: { Before } }
        ifFalse: { other <= self
                      ifTrue: { After }
                      ifFalse: { NotOrderedException
                                     value: self
                                     value: other } }
```

Floating point classes can implement `order:` directly providing the IEE
754-2008 specified total order.

Sort methods taking a two-argument block can exist in two variants:

- `sort:` Where block provides the equivalent of `<=`.
- `sortOrder:` Where block provides the equivalent of `order:`.

Sort methods taking the equivalent of `<=` as a block argument can also
come in `NotComparableException` handling variants:

- `sort:ifNotOrdered:`, taking arguments `Before`, `After`, `Same`, and `Drop`.

This will allow _"sort this, but put all nans last, no matter the sign"_ to be
handled by: `array sort: { |a b| a <= b } ifNotOrdered: Last`.

### Summary

People need to sort floats. Not providing built-in tools for this just means
everyone will roll something on their own.

Despite everyone saying _"floats don't have a total order"_ the standard
actually does specifiy it, so using it seems like a no-brainer.

Also, of all the complaints levered against Java's floating points I haven't
actually heard anyone complaining about the total order.

#### Safety

None.

#### Ergonomics

Positive: sorting is easier, different behaviours are accessible. Aestherics
are good.

### Performance

None.

#### Uniformity

None.

#### Implementation

Minor increase in complexity, but localized to implementation of the required
methods.

#### Users

No users, no issues.

## Alternatives

- Don't implement `order:` for floats, but provide IEEE754TotalOrder and
  JavaTotalOrder and similar blocks to use with `sortOrder:`. The default
  implementation will work as long as there are no NaN's in the data, and signal
  an exception when they appear.

- Use `<=` or `<` for sorting without checking for non-comparability, allowing
  NaNs to cause unpredictable sorting.

- Implement specialized sorting functions for floats: `sortNansFirst`, etc.

- Implement a non-IEEE defined total order like Java does (all NaNs sorted last.)

## Implementation Notes

None.

## Discussion

None.
