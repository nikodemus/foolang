# 012 - Record Implementation

**Status**: ADOPTED (partial implementation)

**Identifier**: 012-record-implementation

**References**: none

**History**:
- 2020-11-10: initial version by Nikodemus
- 2021-01-05: updated to reflect current status

## Problem Description

Foolang records are defined by their constructors. In the bootstrap evaluator
there is a single Record class containing a dictionary that implements all
records, but this is inefficient for memory and accesses both.

It should be possible to declare types of records, and use that type with other
records using the same constructor.

```
define FooBar { foo: 0 bar: 0 } class!

method frob: foobar::Foobar
   ...
```

(Mockup, not the way it is going to be done!)

Having one type per constructor site in source location is too much: then the
type is useless.

### Decision Drivers

- Space efficiency
- Usability of specific record types
- Ability to still create ad-hoc records via Record class.

## Proposal

Record interface no longer works as ad-hoc constructor for records: the
syntactic form must be used. This ensures that all records types are statically
known.

For every constructor, sort the keywords into canonical order.

(Duplicate keys are an error.)

If an anonymous record class identified by this concatenation already exists,
use it. If it doesn't create one with instance methods for accessing the slots.

Bind the arguments to temporaries at the call-site, and generate a call to the
constructor of the anonymous record class.

Record interface now only functions as supertype of all records.

### Summary

- Good space efficiency.
- "Accepts the same reader methods" == "Same record type".
- Ability to manually construct record types not provided, but
  not made any harder.
- Ability to create ad-hoc records by indirect sends to Record lost.
  This functionality can be implemented in a library, even extending
  Record. (Though those records will not have the same types!)

#### Safety

No impact.

#### Ergonomics

No impact.

#### Performance

Positive impact: better space efficiency, better runtime performance.

#### Uniformity

Negative imact: `{ foo: 42 }` becomes a magical construct instead of a simple
rewrite to `Record foo: 42`.

This can potentially be overcome by providing: the moral equivalent of compiler
macros, so that users could hook into `Record foo: 42` to rewrite it into
`( class FooRecord { foo } end. FooRecord foo: 42 )`.

#### Implementation

This is more work to implement than "single record class for all records, store
the values in a dictionary"-strategy used in the bootstrap evaluator.

#### Users

No users, no issues.

## Alternatives

- Keep the bootstrap evaluator stragety. This may actually be what happens
  at first, because it is trivial to implement!
- Allow Record to create ad-hoc types at runtime. Either requires giving up
  on type-equivalence, or allowing the global object to cache things, which
  is wrought to say the least.

## Implementation Notes

- Partial implmentation in transpiler, bootstrap evaluator and interpreter
  do not use this.

## Discussion

- Having the idea of a "record type" is starting to seem suspect. The
  constructors and the shared classes are fine, but being able to specify
  "this thing must be a record of foo and bar" seems wrong.
