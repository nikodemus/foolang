# Foolang Class Reference

!> Incomplete, work in progress!

## Array

- _interface [Collection](#collection)_

#### **method** `+` _value_

Returns a fresh array representing the sum of receiver and _value_. Numbers are
broadcasted across the elements, arrays apply the operation element-wise.

#### **method** `-` _value_

Returns a fresh array representing the substraction of _value_ from the
receiver. Numbers are broadcasted across the elements, arrays apply the
operation element-wise.

#### **method** `*` _value_

Returns a fresh array representing the multiplication of receiver with _value_.
Numbers are broadcasted across the elements, arrays apply the operation
element-wise.

#### **method** `/` _value_

Returns a fresh array representing the division of receiver by _value_.
Numbers are broadcasted across the elements, arrays apply the operation
element-wise.

#### **method** `at:` _index_

Returns element of receiver at _index_. Raises an exception if _index_ is
out of bounds.

#### **method** `at:` _index_ `put:` _object_

Stores _object_ at _index_ in the receiver. Raises an exception if _index_ is
out of bounds.

#### **method** `broadcast:` _block_ `to:` _otherArray_

Broadcasts the operation represented by _block_ across _otherArray_ and the receiver,
collecting the results into a fresh array. Used mainly to implement arithmetic
operations.

#### **method** `collect:` _block_

Returns a fresh array same size as the receiver, collecting results of executing
_block_ with each element of the receiver. See also: `with:collect:`.

#### **method** `displayOn:` _stream_

Displays the receiver on _stream_, sending the `displayOn:` message to
individual elements.

#### **method** `do:` _block_

Executes _block_ with each element of the receiver, returns receiver. See also:
`with:do:`.

#### **method** `dot:` _array_

Dot product of receiver and _array_.

#### **method** `first`

Returns the first element of the receiver. Raises an exception of the receiver is
empty.

#### **method** `norm`

Returns the L2 (euclidean) norm of the receiver.

#### **method** `normalized`

Returns a normalized copy of the receiver, ie. one where every element has been
divided the `norm` of the receiver.

#### **method** `printOn:` _stream_

Prints the receiver on _stream_, sending the `printOn:` message to individual
elements.

#### **method** `push:` _object_

Increses the size of the array by one, adding _object_ as the new last element.

#### **method** `scalarProjection:` _array_

Returns the scalar projection of receiver on _array_.

#### **method** `second`

Returns the second element of the receiver. Raises an exception if the
receiver has size < 2.

#### **method** `size`

Returns the number of elements in the array.

#### **method** `vectorProjection:` _otherArray_

Returns a fresh array representing the vector projection of receiver on
_otherArray_.

#### **method** `with:` _otherArray_ `collect:` _block_

Returns a fresh array collecting the results of executing _block_ with each
element of the receiver and the corresponding element of _otherArray_.

#### **method** `with:` _otherArray_ `do:` _block_

Executes _block_ with each element of the receiver and the corresponding element
of the _otehrArray_. Returns the receiver.

## Clock

To gain access to a _Clock_ use [System#clock](#method-clock).

#### method `time`

Returns a [Time](#time) object representing current time.

## Closure

!> Will be renamed `Block`, probably.

#### **method** `finally:` _cleanup_

Executes the receiver and returns the resulting value, arranging cleanup to be
executed after the receiver even if the receiver raises an exception.

#### **method** `onError:` _handler_

Executes the receiver and returns the resulting value, unless the receiver
raises an exception, in which case the _handler_ is executed with the
exception and context, its value is returned instead.

#### **method** `value`

Executes the receiver and returns the resulting value

#### **method** `value:` _argument_

Executes the receiver with _argument_ and returns the resulting value.

#### **method** `value:` _argument1_ `value:` _argument2_

Executes the receiver with _argument1_ and _argument2_ and returns the resulting
value.

#### **method** `value:` _argument1_ `value:` _argument2_ `value:` _argument3_

Executes the receiver with _argument1_, _argument2_, and _argument3_ and returns
the resulting value.

#### **method** `with:` _value_

Executes the receiver with _value_ and returns the resulting value, arranging
`close` method to be sent to the _value_ after receiver even if the receiver
raises an exception.

#### **method** `whileFalse`

Executes the receiver repeatedly as long as the result is false.

#### **method** `whileFalse:` _body_

Executes the receiver and the _body_ repeatedly, as long as the receiver
result is false.

#### **method** `whileTrue`

Executes the receiver repeatedly as long as the result is true.

#### **method** `whileTrue:` _bodyBlock_

Executes the receiver and the _body_ repeatedly, as long as the receiver
result is true.

## File

#### **method** `forAppend` -> _File_

Returns a fresh file similar to receiver, with append-mode set. Overrides
a previous `forWrite`.

#### **method** `forRead` -> _File_

Returns a fresh file similar to receiver, with read-mode set. Overrides
a previous `forAppend`.

#### **method** `forWrite` -> _File_

Returns a fresh file similar to receiver, with write-mode set. Overrides

#### **method** `isAppend` -> _Boolean_

Returns true if the receiver has append-mode set.

#### **method** `isRead` -> _Boolean_

Returns true if the receiver has read-mode set.

#### **method** `isTruncate` -> _Boolean_

Returns true if the receiver has truncate-mode set.

#### **method** `isWrite` -> _Boolean_

Returns true if the receiver has write-mode set.

#### **method** `truncateExisting` -> _File_

Returns a fresh file similar to receiver, with truncate-mode set.

## FilePath

Object representing a point in the filesystem and permission to operate
at the point and below.

Existence of _FilePath_ object does not mean that the corresponding file
or directory exists.

!> Symlinks in the filesystem can provide access to parts outside the
_FilePath_.

#### **method** `file` -> _File_

Returns a [File](#file) associated with the receiver, with open mode unset.

#### **method** `forAppend` -> _File_

Returns a [File](#file) associated with the receiver, ready to be opened in
append-mode.

#### **method** `forRead` -> _File_

Returns a [File](#file) associated with the receiver, ready to be opened in
read-mode.

#### **method** `forWrite` -> _File_

Returns a [File](#file) associated with the receiver, ready to be opened in
write-mode.

#### **method** `path:` _pathname_ -> _FilePath_

Returns a new _FilePath_ object representing the _pathname_ relative
to the receiver. Using `..` in pathnames is not allowed.

#### **method** `exists`

Returns true if the receiver designates a filesystem resource that exists.

#### **method** `isDirectory`

Returns true if the receiver designates a directory that exists in the filesystem.

#### **method** `isFile`

Returns true if the receiver designates a file that exists in the filesystem.

## System

Programs initially gain access to a _System_ object by receiving it as the second
parameter to `Main##run:in:`.

Outside development mode this is the only way to gain access to things like the
filesystem and the network. In development mode (as currently provided by the
webrepl and `repl.foo`) the system object is accessible as lexically bound
`system` variable.

#### **method** `abort`

Aborts the current process.

#### **method** `clock`
  Returns a [Clock](#clock).

#### **method** `currentDirectory`

Returns a _FilePath_ object providing access to files in the current directory.
See also: `files`.

#### **method** `exit`

Exits the current process with exit code 0.

#### **method** `exit:` _code_

Exits the current process with the specified _code_.

#### **method** `files`

Returns a _FilePath_ object providing access to files in the entire
filesystem. See also: `currentDirectory`.

#### **method** `input`

Returns the standard input as an _Input_.

#### **method** `output` \

Returns the standard output as an _Output_.

#### **method** `output:` _output_

Returns a new _System_ where standard output has been replaced _output_.

#### **method** `random`

Returns a _ByteArray_ initialized with random data from the operating system.
Currently returns 32 bytes of random data, but this is subject to change.

#### **method** `random:` _size_

Returns a _ByteArray_ of specified size initialized with random data.

#### **method** `sleep`

Sleeps one millisecond.

#### **method** `sleep:` _milliseconds_

Sleeps the speficied number of milliseconds.

#### **deprecated method** `window:` _name_

Returns a _Window_ object with specified name. (This will be moved into
a plugin.)

## Time

Time object represents an instant in time.

!> This object was initially written to support benchmarking cases, and
doesn't really work for general use. Will be renamed _ProcessTime_ or similar.

#### **method** `addTime:` _time_ -> _Time_

Returns sum of two _Time_ objects.

#### **method** `real` -> _Float_

Returns number of wallclock seconds elapsed since program started as a floating
point value. (Millisecond resolution.)

#### **method** `subTime:` _time_ -> _Time_

Returns difference of two _Time_ objects.

#### **method** `system` -> _Float_

Returns number of _system-time_ seconds elapsed since program started as a
floating point valie. (Millisecond resolution.) _System-time_ refers to amount
of processor time program has used in system calls (ie. in the kernel.)

#### **method** `user` -> _Float_

Returns number of _user-time_ seconds elapsed since program started as a floating
point value. (Millisecond resolution.) _User-time_ refers to amount of processor
time program has used in user-space (ie. not in kernel).
