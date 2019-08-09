# Extension Methods

Normal methods live in the vtable(s) of the class.

Normal methods can be added post-hoc using reflection -- but that
implies development mode. This is about disciplined extensions, not
reflection.

Extension methods live in per method table that dispatches on the class of the receiver and
are lexically distinct.

## Double Dispatch implementation

Extension methods for double-dispatch protocols as identified by
classname suffixed to the selector. They are formally associated with
that class, not the classes their implementations recide in, and c

      class Collection
        method at: pos
           pos at.Collection: self
       end

      method Integer at.Collection: collection
        collection atIndex: self
      end

      method Array at.Collection: collection
        collection atIndexes: self
      end

## Proper extensions

Extension methods adding new functionalities to existing classes
are defined using similar toplevel syntax, but are associated with the package
they are defined in:

    -- in package pretty
    method Object pprint
       ...
    end

    -- in package mypkg
    import pretty
    class Foo
       method foo: x
         x pretty.pprint
    end

    -- in package otherpkg
    import pretty.pprint
    class Foo
       method foo: x
         x pprint
    end

## Interaction with Compiler new and evaluator

If object from another Compiler leak into parent and have extension methods, those methods
need to be resolved to the correct tables even in an evaluator.

This implies an analysis step or careful co-operation with the parser for evaluator.
