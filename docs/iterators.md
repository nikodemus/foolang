# Foolang Iterators

!> Sketching out the interfaces. Does not match what is currently implemented.
Would also like to disentangle this into purely functional and side-effectful
parts.

!> Not super fond of minor inconsistencies in naming: `select:` but `indexIf:`,
etc.

!> Would also like a method of operating on slices, reflecting the changes
back to original. Would make `(x from: start to: end) rotateLeft: n` work
like magic.

!> Need better documentation structure to be able to define things like
"violating bounds causes an exception to be raised" without needing to
repeat them over and over again.

``` mermaid
graph TD
    _iterable(["Iterable&nbsp;"])

    _collection(["Collection&nbsp;"])
    _interval(["Interval&nbsp;"])

    _generator["Generator"]

    _set([Set])
    _ordered([Ordered])
    _map([Map])

    _intervalf64["IntervalF64&nbsp;"]
    _intervali64["IntervalI64&nbsp;"]

    _hashset[HashSet]
    _orderedset[OrderedSet]
    _list[List]
    _hashmap[HashMap]
    _orderedmap[OrderedMap]

    _iterable-->_collection
    _iterable-->_interval
    _iterable-->_generator

    _interval-->_intervalf64
    _interval-->_intervali64

    _collection-->_set
    _collection-->_ordered
    _collection-->_map

    _set-->_hashset
    _set-->_orderedset
    _ordered-->_orderedset
    _ordered-->_list
    _ordered-->_orderedmap
    _map-->_orderedmap
    _map-->_hashmap

```

## Iterable

A potentially infinite sequence of objects that can be iterated over.

- **required method** `iterator`

  Returns an _Iterator_ object that can be used to step over the collection
  using messages `next`, `atEnd`, and `tryNext:`.

  ---

- **method** `do:` _block_ \
  Executes _block_ repeatedly, with each object in the receiver.
  Returns the receiver.

- **method** `inject:` _initialValue_ `into:` _block_ \
  Returns the result of executing _block_ repeately with an accumulator
  and each object in the receiver. Accumulator starts as _initialValue_
  and is updated to value returned by _block_ after each execution.

- **method** `with:` _iterable2_ `do:` _block_ \
  Executes _block_ repeatedly, with each object in the receiver and
  the corresponding object in _iterable2_.
  If the receiver is exausted before _iterable2_, the elements remaining
  in _iterable2_ are ignored.
  If _iterable2_ is exhausted before the receiver, an exception is raised.
  Returns the receiver.

- **method** `with:` _iterable2_ `inject:` _initialValue_ `into:` _block_ \
  Returns the result of executing _block_ repeatedly with an accumulator, each
  object in the receiver, and the corresponding object in _iterable2_.
  Accumulator starts as _initialValue_
  and is updated to value returned by _block_ after each execution.
  If the receiver is exausted before _iterable2_, the elements remaining
  in _iterable2_ are ignored.
  If _iterable2_ is exhausted before the receiver, an exception is raised.

  ---

- **method** `ifEmpty:` _block_ \
  If receiver is empty executes _block_ and returns its value.
  Otherwise returns false.

- **method** `ifEmpty:` _emptyBlock_ `ifNotEmpty:` _notEmptyBlock_ \
  If receiver is empty executes _emptyBlock_ and returns its value.
  Otherwise executes _notEmptyBlock_ and returns its value.

- **method** `ifNotEmpty:` _block_ \
  If receiver is not empty executes _block_ and returns its value.
  Otherwise returns false.

- **method** `isEmpty` \
  Returns true if the collection is empty and false otherwise.

## Collection

A finite collection of objects, allowing addition of new elements.

- **required method** `size` \
  Returns the number of objects in the collection.

- **required method** `species` \
  Returns the class that should be used to construct new instances of the
  collection. The class must understand following messages:
  - `new` which returns a new, empty instance of the collection
  - `newWithCapacity:` which returns a new, empty instance of the collection
     sized to hold the specified number of objects.

- **required method** `add:` _object_ \
  Adds a new object to the collection. Size of the collection typically
  increases by one as a result.

  ---

- **method** `collect:` _block_ \
  Executes _block_ repeatedly, with each object in the receiver.
  Returns a new collection of the same species as the receiver, containing the
  objects returned by _block_.

- **method** `with:` _collection2_ `collect:` _block_ \
  Executes _block_ repeatedly, with each object in the receiver, and
  the corresponding object from _collection2_.
  Returns a new collection of the same species as the receiver containing the
  objects returned by _block_.
  If the receiver is exausted before _collection2_, the elements remaining
  in _collection2_ are ignored.
  If _collection2_ is exhausted before the receiver, an exception is raised.

  ---

- **method** `allSatisfy:` _block_ \
  Returns true if _block_ is true for all objects in the receiver,
  false otherwise.

- **method** `addAll:` _iterable_ \
  Add all objects in the _iterable_ to the receiver.

- **method** `anySatisfy:` _block_ \
  Returns true if _block_ is true for at least one object in the receiver,
  false otherwise.

- **method** `count:` _block_ \
  Returns the number of objects in receiver for which _block_ returnes true.

- **method** `find:` _block_ \
  Executes _block_ repeatedly, with each object in the receiver,
  until it find one where block returns true.
  Returns the object for which the block returned true, or
  raises an exception if no such object was found.

- **method** `find:` _block_ `ifNone:` _noneBlock_ \
  Executes _block_ repeatedly, with each object in the receiver,
  until it find one where block returns true.
  Returns the object for which the block returned true, or
  if no such object was found executes the _noneBlock_ and returns
  its value.

- **method** `includes:` _object_ \
  Returns true if receiver contains the _object_, false otherwise.

- **method** `reject:` _block_ \
  Executes _block_ repeatedly, with each object in the receiver.
  Returns a new collection of the same species as the receiver, containing the
  objects for which _block_ returned false.

- **method** `select:` _block_ \
  Executes _block_ repeatedly, with each object in the receiver.
  Returns a new collection of the same species as the receiver, containing the
  objects for which _block_ returned true.

## Set

Sets are collections which contain each object only once: adding the same object
multiple times to a set is same as adding it once.

!> This seems a bit awkward. All of these could in principle be supported by any
collection. It's just that sets are designed to make these operations efficient
and maintain the invariant.

- **method** `intersection:` _iterable_ \
  Returns a fresh set of the same species as the receiver containing
  those objects in the receiver that also appear in the _iterable_.

- **method** `difference:` _iterable_ \
  Returns a fresh set of the same species as the receiver containing
  those objects in the receiver that do not appear in the _iterable_.

- **method** `union:` _iterable_ \
  Returns a fresh set of the same species as the receiver containing
  objects in the receiver and those of the _iterable_.

## Ordered

Ordered collections allow accessing their members by integer indexes from 1 to
size of the collection. Iteration order of an Ordered collection is from index 1
upwards. `add:` method adds to the end.

- **required method** `at:` _index_ \
  Returns the object at the specified _index_ in the receiver. Raises an exception
  if the _index_ is less than one or greater than size of the receiver.

- **required method** `at:` _index_ `put:` _object_ \
  Puts the _object_ at the specified _index_ in the receiver. Raises an
  exception if the index is less than one or greater than size of the
  receiver.

- **required method** `removeFirst` \
  Returns first element in the receiver, and removes it.

- **required method** `removeLast` \
  Returns last element in the receiver, and removes it.

- **method** `from:` _start_ `to:` _end_ \
  Returns a new ordered collection of
  the same species containing elements from _start_ to _end_. Raises an
  exception if either _start_ or _end_ is less than one or greater than size of
  the receiver.

  _XXX: a lot of methods should probably have variants prefixed with `from:to:`,
  along the lines of `from: i to: j select: test`._

- **method** `from:` _start_ `to:` _end_ `replaceFrom:` _iterable_
  Replaces elements of the receiver from _start_ to _end_ with elements from
  _iterable_. Raises an exception is _iterable_ is exhausted before specified
  range has been replaced.

  _XXX: or maybe the answer is to have slices/iterators that allow mutation:
  `(seq from: start to: end) replaceFrom: other`. This is also very close
  to the slicing and broadcasing protocol I was thinking about for arrays:
  `seq at: (start to: end) put: replacement` -- but needs a bit of finesses
  to be able to communicate `replaceFrom:` vs `replaceWith:` type operations._

- **method** `from:` _start_ `to:` _end_ `replaceWith:` _object_
  Replaces elements of the receiver from _start_ to _end_ with _object_.

- **method** `index:` _object_ \
  Returns first position of _object_ in the receiver. Raises an exception
  if _object_ is not in receiver.

  _XXX: should there be also `indexes:`?_

- **method** `index:` _object_ `ifNone:` _block_ \
  Returns first position of _object_ in the receiver. Executes _block_
  if _object_ is not in receiver, and returns its value.

- **method** `indexIf:` _block_ \
  Returns position of first _object_ in the receiver for which _block_
  returns true. Raises an exception if _object_ is not in receiver.

- **method** `indexIf:` _block_ `ifNone:` _noneBlock_ \
  Returns position of first _object_ in the receiver for which _block_ returns
  true. Executes _noneBlock_ if _object_ is not in receiver, and returns its value.

- **method** `reverse` \
  Reverses the receiver in place.

- **method** `reversed` \
  Returns copy of the receiver, in reverse order in reverse order.

- **method** `rotateLeft:` _n_
  Rotates elements of receiver left _n_ times.

- **method** `rotateRight:` _n_
  Rotates elements of receiver right _n_ times.

- **method** `sort:` _block_ \
  Sorts the receiver using _block_ as comparator: _block_ is called
  with two objects from the receiver in time, and should return true
  if the first element belong before second one.

- **method** `sorted:` _block_ \
  Returns a new ordered collection of the same
  species as the receiver, using _block_ as comparator: _block_ is called with
  two objects from the receiver in time, and should return true if the first
  element belong before second one.

## Map

## Interval
