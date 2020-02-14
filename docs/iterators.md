# Foolang Iterators

!> Sketching out the interfaces. Does not match what is currently implemented.

``` mermaid
graph TD
    iterable(["Iterable&nbsp;"])

    collection(["Collection&nbsp;"])
    interval(["Interval&nbsp;"])

    generator["Generator"]

    set([Set])
    ordered([Ordered])
    map([Map])

    intervalf64["IntervalF64&nbsp;"]
    intervali64["IntervalI64&nbsp;"]

    hashset[HashSet]
    orderedset[OrderedSet]
    list[List]
    hashmap[HashMap]
    orderedmap[OrderedMap]

    iterable-->collection
    iterable-->interval
    iterable-->generator

    interval-->intervalf64
    interval-->intervali64

    collection-->set
    collection-->ordered
    collection-->map

    set-->hashset
    set-->orderedset
    ordered-->orderedset
    ordered-->list
    ordered-->orderedmap
    map-->orderedmap
    map-->hashmap

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
  Adds a new object to the collection.

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

## Order

## Map

## Interval
