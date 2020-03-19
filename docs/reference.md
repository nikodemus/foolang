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

#### **method** `==` _other_

Returns true if _other_ has the same size and elements as the receiver when
compared using `==`. (Depends on `size` and `at:` methods of _other_.)

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

#### **method** `concat:` _array_

Returns concatenation of receiver and the _array_.

#### **method** `copy`

Returns a shallow copy of the receiver.

#### **method** `count:` _block_

Returns the number of elements in the receiver for which the _block_ returns true.

#### **method** `displayOn:` _stream_

Displays the receiver on _stream_, sending the `displayOn:` message to
individual elements.

#### **method** `do:` _block_

Executes _block_ with each element of the receiver, returns receiver. See also:
`with:do:`.

#### **method** `do:` _block_ `interleaving:` _interBlock_

Executes _block_ with each element of the receiver, and _interBlock_ between
each execution of _block_. Returns receiver.

#### **method** `dot:` _array_

Dot product of receiver and _array_.

#### **method** `first`

Returns the first element of the receiver. Raises an exception of the receiver is
empty.

#### **method** `find:` _block_

Returns the first element of the receiver for which _block_ returns true,
false otherwise.

#### **method** `find:` _block_ `ifNone:` _noneBlock_

Returns the first element of the receiver for which _block_ returns true,
or the value of _noneBlock_ if no element matched.

#### **method** `ifEmpty:` _block_

Executes _block_ if the collection is empty, and returns its value. Otherwise
returns false.

#### **method** `ifEmpty:` _block_ `ifNotEmpty:` _notBlock_

Executes _block_ if the collection is empty, and returns its value. Otherwise
executes _notBlock_ and returns its value.

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

#### **method** `reject:` _block_

Returns an array containing those elements of the receiver for which _block_
did not return true.

#### **method** `scalarProjection:` _array_

Returns the scalar projection of receiver on _array_.

#### **method** `second`

Returns the second element of the receiver. Raises an exception if the
receiver has size < 2.

#### **method** `select:` _block_

Returns an array containing those elements of the receiver for which _block_
returns true.

#### **method** `size`

Returns the number of elements in the array.

#### **method** `sort`

Sorts the array in place into ascending, order using `<`. Returns the receiver.

#### **method** `sort:` _sortBlock_

Sorts the array in place into using _sortBlock_ as comparator: it should return
true if first argument should be placed before the second argument. Returns the
receiver.

#### **method** `sorted`

Returns a sorted copy of the array in ascending, order using `<`.

#### **method** `sorted:` _sortBlock_

Returns a sorted copy of the using _sortBlock_ as comparator: it should return
true if first argument should be placed before the second argument.

#### **method** `swap:` _index1_ `with:` _index2_

Swaps the receiver's elements at _index1_ and _index2_. Returns the receiver.

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

#### **method** `apply:` _array_

Executes the block using values from the array.

#### **method** `ascending`

Assumes that the receiver is a single-argument block that returns a value to be
used as sort key. Converts this into a comparison block that can be used with
`#sort:` methods to sort in ascending order. See also: `descending`.

Example:
``` foolang
["aa", "a", "aaa"] sort: { |s| s size } ascending --> ["a", "aa", "aaa"]
```

#### **method** `descending`

Assumes that the receiver is a single-argument block that returns a value to be
used as sort key. Converts this into a comparison block that can be used with
`#sort:` methods to sort in descending order. See also: `ascending`.

Example:
``` foolang
["aa", "a", "aaa"] sort: { |s| s size } descending --> ["aaa", "aa", "a"]
```

#### **method** `finally:` _cleanup_

Executes the receiver and returns the resulting value, arranging cleanup to be
executed after the receiver even if the receiver raises an exception.

#### **method** `loop`

Executes the receiver repeatedly forever, or until the receiver uwinds through
a return or exception.

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

#### **method** `create` -> _FileStream_

Creates a new file and opens it for IO in the specified mode, and returns the
resulting [FileStream](#filestream). Raises an exception if the file already
exists, or directory does not have the required permissions.

#### **method** `create:` _block_

Returns result of executing the _block_ with the [FileSteam](#filestream)
resulting from sending `create` to the receiver, and ensures that the stream is
closed after block completes even if an exception is raised.

#### **method** `createOrOpen` -> _FileStream_

If the specified file does not exist, behaves like `create`, otherwise
behaves like `open`.

#### **method** `createOrOpen:` _block_

Returns result of executing the _block_ with the [FileSteam](#filestream)
resulting from sending `createOrOpen` to the receiver, and ensures that the
stream is closed after block completes even if an exception is raised.

#### **method** `forAppend` -> _File_

Returns a fresh file similar to receiver, with append-mode set. Overrides
a previous `forWrite`.

#### **method** `forRead` -> _File_

Returns a fresh file similar to receiver, with read-mode set.

#### **method** `forWrite` -> _File_

Returns a fresh file similar to receiver, with write-mode set. Overrides
a previous `forAppend`.

#### **method** `isAppend` -> _Boolean_

Returns true if the receiver has append-mode set.

#### **method** `isRead` -> _Boolean_

Returns true if the receiver has read-mode set.

#### **method** `isTruncate` -> _Boolean_

Returns true if the receiver has truncate-mode set.

#### **method** `isWrite` -> _Boolean_

Returns true if the receiver has write-mode set.

#### **method** `open` -> _FileStream_.

Opens an existing file in specified mode, and returns the resulting
[FileStream](#filestream). Raises an exception if the file does not
exist, or is not a file with the required permissions.

#### **method** `open:` _block_

Returns result of executing the _block_ with the [FileSteam](#filestream)
resulting from sending `open` to the receiver, and ensures that the stream is
closed after block completes even if an exception is raised.

#### **method** `truncateExisting` -> _File_

Returns a fresh file similar to receiver, with truncate-mode set. Note: existing
files are truncated on open only if write-mode is set.

## FilePath

Object representing a point in the filesystem and permission to operate
at the point and below.

Existence of _FilePath_ object does not mean that the corresponding file
or directory exists.

!> Symlinks in the filesystem can provide access to parts outside the
_FilePath_.

#### **method** `deleteFile`

Deletes the file designated by the receiver. Raises and exception if the
path does not exist or is not a file.

#### **method** `exists`

Returns true if the receiver designates a filesystem resource that exists.

#### **method** `file` -> _File_

Returns a [File](#file) associated with the receiver, with open mode unset.

#### **method** `forAppend` -> _File_

Returns a [File](#file) associated with the receiver, ready to be opened in
append-mode. Convenience around `self file forAppend`.

#### **method** `forRead` -> _File_

Returns a [File](#file) associated with the receiver, ready to be opened in
read-mode. Convenience around `self file forRead`.

#### **method** `forWrite` -> _File_

Returns a [File](#file) associated with the receiver, ready to be opened in
write-mode. Convenience around `self file forWrite`.

#### **method** `ifExists:` _block_

Executes _block_ if the path designated by the receiver exists.

#### **method** `isDirectory`

Returns true if the receiver designates a directory that exists in the filesystem.

#### **method** `isFile`

Returns true if the receiver designates a file that exists in the filesystem.

#### **method** `path:` _pathname_ -> _FilePath_

Returns a new _FilePath_ object representing the _pathname_ relative
to the receiver. Using `..` in pathnames is not allowed.

#### **method** `readString` -> _String_

Returns contents of the file designated by the receiver as a _String_.

#### **method** `readString:` _pathname_ -> __String_

Returns the contents of the file designated by _pathname_ relative to the
receiver as a _String_.

## FileStream

#### **method** `close` -> _Boolean_

Closes the stream if it is currently open. Returns true if the stream was
open, false if it was already closed.

#### **method** `flush` -> _FileStream_

Flushes the stream, ensuring internally buffered data reach their destination.

#### **method** `isClosed` -> _Boolean_

Returns true if the receiver has been closed.

#### **method** `isOpen` -> _Boolean_

Returns true if the receiver is open (has not been closed.)

#### **method** `offset` -> _Integer_

Returns current offset from the beginning of the file.

#### **method** `offset:` _absoluteOffset_ -> _Integer_

Sets and returns offset from the beginning of the file.

#### **method** `offsetFromEnd:` _relativeOffset_ -> _Integer_

Sets and returns offset from the end of the file.

#### **method** `offsetFromHere:` _relativeOffset_ -> _Integer_

Sets and returns offset relative to current position.

#### **method** `readBytes` -> _ByteArray_

Returns all data from receiver (starting at current offset) as a _ByteArray_.

#### **method** `read:` _numberOf_ `bytesInto:` _byteArray_ `at:` _index_ -> _Integer_

Reads exactly the specified _numberOf_ bytes from receiver into _byteArray_
starting at the specified _index_ in the _byteArray_.

If unable to read the specified number of bytes for any reason, raises an exception.

#### **method** `readString` -> _String_

Returns remaining contents of the receiver as a _String_.

#### **method** `size` -> _Integer_

Returns the total size of the underlying file in bytes.

#### **method** `tryRead:` _numberOf_ `bytesInto:` _byteArray_ `at:` _index_ -> _Integer_

Reads at most specified _numberOf_ bytes from receiver into _byteArray_ starting
at the specified _index_ in the _byteArray_.

Returns the number of bytes actually read, which may be less than the requested
number if end of file is reached before.

#### **method** `tryReadOnce:` _numberOf_ `bytesInto:` _byteArray_ `at:` _index_ -> _Integer_

Reads at most specified _numberOf_ bytes from receiver into _byteArray_ starting
at the specified _index_ in the _byteArray_, and returns the number of bytes
actually read.

Assuming more than zero bytes were requested, returning zero from this method
indicates that end of file has been reached.

Normally performs exactly one OS-level read operation. (An interrupted read that
produced no data will be restarted automatically do distinguish it from reaching
end of file.)

Raises an exception in other cases where no data can be read.

#### **method** `tryWrite:` _numberOf_ `bytesFrom:` _byteArray_ `at:` _index_ -> _Integer_

Writes at most specified _numberOf_ bytes from _byteArray_ into receiver, starting
at the specified _index_ in the _byteArray_.

Returns the number of bytes actually written, which may be less than the requested
number if the file is unable to accept all data for some reason.

Raises an exception if no data could be written.

#### **method** `tryWriteOnce:` _numberOf_ `bytesFrom:` _byteArray_ `at:` _index_ -> _Integer_

Writes at most specified _numberOf_ bytes from _byteArray_ into receiver, starting
at the specified _index_ in the _byteArray_, and returns the number of bytes
actually written.

Normally performs exactly one OS-level write operation. (An interrupted write
that consumed no data will be restarted automatically.)

Raises an exception in other cases if no data can be written.

#### **method** `write` _numberOf_ `bytesFrom:` _byteArray_ `at:` _index_ -> _Integer_

Writes exactly the specified _numberOf_ bytes from _byteArray_ into receiver,
starting at the specified _index_ in the _byteArray_.

If unable to write the specified number of bytes for any reason, raises an exception.

#### **method** `writeBytes:` _bytes_

Writes the _bytes_ to the receiver at the current offset.

#### **method** `writeString:` _string_

Writes the string to the receiver at current offset, as UTF8.

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
