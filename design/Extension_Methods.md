# Extension Methods

## Class-Suffixed Selectors

Extension methods suffixed with the name of a class defined in the
current module can be added to any existing class.

They are formally associated with that class, not the classes their
implementations recide in:

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

Modules importing the suffixing class can also define these methods in
classes they implement.

This is sufficient for freely extensible double-dispatch, I believe.

## Module-Prefixed Selectors

Extension methods adding new functionalities to existing classes are
defined using `extend`, which is like `class` but only allows methods
that do not use instance variables:

Eg. in file bitwise.foo:

    extend Integer
       method Integer & other
          other andInteger: other
    end

The actual selector of extension methods is module.selector, so in the
above case `bitwise.&`. The prefixed selector is added to all vtables
which use it's regular counterpart, as an alias for that -- so:

    bitvector1 & bitVector2

works just as before, even if & is actually bitwise.Integer.&.

Those importing the module can use them with the prefix,

    import bitwise
    1 bitwise.& 2 --> 3

or import the selector itself to use it without prefix:

    import bitwise.&
    1 & 3 --> 3

XXX: Conflicts with `import foo.*`, which needs to be replaced with
something else -- or importing selectors needs another syntax.

Prefixed selectors can also be added to locally defined classes,
allowing same class to support both the vanilla selector and the
overridden one:

    import bitwise
    class Foo
        method bitwise.& other
          ...

        -- This is the "normal" &, explicit prefix for clarity.
        method foolang.& other
          ...
    end

If `foolang.&` were not explicitly defined, then regular `&` would not
be defined: extensions forward to vanilla methods, but not the wise versa.

When `bitwise.&` hits `doesNotUnderstand` it gets that instead of `foolang.&`.
This is so that delegation works properly -- when it finally ends at an
object that implements either, then aliasing takes care of the rest.
