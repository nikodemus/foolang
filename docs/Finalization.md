# Finalization

Finalization is an annotation on an instance variable, and comes in
two varieties: always and gc.

    <finalize always: close>

    <finalize gc: close>

Gc-finalization is done when the parent object is garbage collected.
Always-finalization is additionally done when the instance variable is
assigned a new value.

Finalization is done in the same thread as allocation, ie.
synchronously with GC. I believe this is ok due to the thread local heaps.

The final part of the finalization specification is the unary message
to send to the instance variable. If the message causes a non-local
transfer of control the thread is immediately aborted.

Instance variables which are nil at the time they would be finalized
are silently ignored.

## Example

If Database goes away while DatabaseQuery is held, next read will
trigger an exception.

```
let db = Database connect: (system network) host: "localhost" port: 1234
let nicknames = db query: "select * from users" collect: { |uid|
   (db query: "select nickname from nicknames where uid = $1"
       with: [uid]) one
} to: Array

define class Database
    { network, host, port,
      _socket <finalize always: close> }

   classMethod connect: network host: host port: port
      (self network: network host: host port: port) connect

   method connect
      # This sends a #close message to the previous socket if any.
      _socket = network openSocket: host port: port

   method query: sql
      # Takes care of the wire protocol.
      DatabaseQuery send: sql to: _socket
      
done

define class DatabaseQuery { _socket, row }

   is: Iterator

   classMethod send: sql to: socket
      (self _socket: socket, row: nil) exec: sql

   method exec: sql
      # presumably more work would be done here...
      _socket send: sql

   method inject: collector into: block
      {
         self nextRow
      }
      whileTrue:
      {
         collector = block value: row value: collector
      }

   method nextRow
      let size = _socket readU32
      # Again, there's probably a more complex on-wire protocol
      size > 0 ifTrue: { row = _socket readUtf8: size. True }

done
```

