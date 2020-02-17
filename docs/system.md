# Foolang System

Programs originally gain access the a _System_ object by receiving it as the
second parameter to `Main##run:in:`. After this the facilities provided by the
can be passed on in messages.

Outside development mode this is the only way to gain access to things like
the filesystem and the network.

In development mode (as provided by the webrepl and `repl.foo`) the system
object is accessible as lexically bound `system` variable.

## System Methods

!> `output:` and `random:` are inconsistent. `output:` will probably
change to `withOutput:`. Both `random` and `random:` will probably change to
returning a `Random` object.

- **method** `abort` \
  Aborts the current process.

- **method** `clock` \
  Returns a _Clock_.

- **method** `exit` \
  Exits the current process with exit code 0.

- **method** `exit:` _code_ \
  Exits the current process with the specified _code_.

- **method** `files`
  Returns a _FilePath_ object providing access to files. If `foo` executable
  was run with a `--files-root` argument, access is limited to this point
  and below.

- **method** `input` \
  Returns the standard input as an _Input_.

- **method** `output` \
  Returns the standard output as an _Output_.

- **method** `output:` _output_ \
  Returns a new _System_ where standard output has been replaced _output_.

- **method** `random` \
  Returns a _ByteArray_ initialized with random data from the operating system.
  Currently returns 32 bytes of random data, but this is subject to change.

- **method** `random:` _size_ \
  Returns a _ByteArray_ of specified size initialized with random data.

- **method** `sleep` \
  Sleeps one millisecond.

- **method** `sleep:` _milliseconds_ \
  Sleeps the speficied number of milliseconds.

- **deprecated method** `window:` _name_ \
  Returns a _Window_ object with specified name. (This will be moved into
  a plugin.)

## FilePath

Object representing a point in the filesystem and permission to operate
at the point and below.

Existence of _FilePath_ object does not mean that the corresponding file
or directory exists.

!> Symlinks in the filesystem can provide access to parts outside the
_FilePath_.

!> Initially _FilePath_ provides full permissions, but subsequently there
will be ways to limit it to read-only operations, write-only operations,
checking timestamps, etc.

!> This area is definitely going to go through a few iterations before
it settles down.

- **method** `path:` _pathname_
  Returns a new _FilePath_ object representing the _pathname_ relative
  to the receiver. Using `..` in pathnames is not allowed.

- **method** `exists`
  Returns true if the receiver designates a filesystem resource that exists.
  !> On Windows the Foolang root filepath describes a level above drives,
  meaning `exists` will return false for it!

- **method** `isDirectory`
  Returns true if the receiver designates a directory that exists in the filesystem.
  !> On Windows the Foolang root filepath describes a level above drives,
  meaning `isDirectory` will return false for it!

- **method** `isFile`
  Returns true if the receiver designates a file that exists in the filesystem.
