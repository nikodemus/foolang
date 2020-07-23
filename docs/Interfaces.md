# Foolang Interfaces

## Desiderate

- Interfaces can provide default implementations
- Interfaces can inherit each other
- Interfaces can be used as types
- Interfaces can require implementations of specific methods
- Interfaces can specify types

## Open Questions

What about direct methods?

While it may make sense for there to be Interface##method, it doesnt'
really feel right for that to imply that there is also Class##method, does it?

## Example

```
interface Scalar
   required method + other
   required method - other
   required prefix -
end

interface Printable
  required method printOn: stream
  
  method toString
      (self printOn: StringStream new) string
end

class Integer { value }
   is Scalar
   is Printable
   ...
end

```

For non-required methods the implementing class copies the vtables of the interfaces.

It is an error if there is a send to self in an interface method
without a corresponding method listed there.

If there's a conflict, that's an error.

Interface methods can also be used directly:

    something Printable.printOn: stream

Most importantly, this is the way to "call superclass methods".
