# Extension Methods

Extension methods are added using toplevel method definitions:

    method ClassName selector
          body
    end

## Class-Suffixed Selectors

Extension methods suffixed with the a name of a class or interface in
the current module can be added to any existing class or interface.
(Methods added to interfaces are always optional.)

      class Collection
        method at: pos
           pos at.Collection: self
       end

      method Integer at.Collection: collection
        collection atIndex: self
      end

      method Array at.Collection: collection
        collection atIndices: self
      end

Modules importing the suffixing class can also define these methods in
classes they implement -- but they cannot extend existing classes.

These suffixes can never be elided. 'at.Collection' name == "at.Collection"

The use case here is double-dispatch extensions when defining new classes
that want to dispatch existing classes as arguments.

(No conflicts can occur.)

## Module-Prefixed Selectors

Extension methods explicitly prefixed with the name of the current
module can be added to any existing class.

By declaring an extension selector using `extend` the prefix can
be (optionally) elided, and these selectors become aliases for the
extended selector.

Eg. in file bitwise.foo:

    extend &

    -- Prefix can be elided due to 'extend'
    method Integer & other
          other andInteger: self
    end

    -- Prefix required since not declared as an extension
    method Integer bitwise.andInteger: other
          foolang.builtins.Logior value: self value: other
    end

The selectors of the extension methods are `bitwise.&` and
`bitwise.andInteger`.

Due to the extension declaration all vtables which use the regular
`default.&` selector gain the `bitwise.&` selector as an alias for it.

Importing the module allows using these methods with an explicit prefix:

    import bitwise
    1 bitwise.& 2 --> 0
    1 bitwise.andInteger: 2 --> 0

Importing a specific selector explicitly allows eliding the prefix:

    import bitwise.&
    1 & 2 --> 0

    import bitwise.andInteger:
    1 andInteger: 2 --> 0

Using importAllExtensions allows eliding the prefix from all declared
extensions:

    importExtensionsFrom bitwise
    1 & 2 --> 0

Modules importing the prefix-extension defining module can also define
these methods in classes they implement.

    import bitwise
    class Foo
        method bitwise.& other
          ...

        -- This is the "normal" &, explicit prefix for clarity.
        method default.& other
          ...
    end

If `default.&` were not explicitly defined, then regular `&` would not
be defined: extensions forward to vanilla methods, but not the wise versa.

When `bitwise.&` hits `doesNotUnderstand` it gets that instead of
`default.&`. This is so that delegation works properly -- when it
finally ends at an object that implements either, then aliasing takes
care of the rest. If for some reason someone wants to dispatch on the
selector text, then they can use `Selector name`: 'bitwise.&' name == "&".

Extensions can be chained:

bitwise2.foo:

     import bitwise
     extend bitwise.&

     ---
     Now & in this file is bitwise2.&, and vtables containing
     bitwise.& have gained it as an alias.
     ---

     method BitVector & other
         other andBitVector: self
     end

     method BitVector andBitVector: other
         foolang.builtins.BitVectorOr value: self value: other
     end

Therefore after importing `bitwise2.&`

    bitvec1 & bitvec2 --> bitvec

    int1 & int2 --> int

    bool1 & bool2 --> bool

all work:

- the first goes directly to the new extension methods
- the second find bitwise2.& aliased to bitwise.&
- the third find bitwise2.& aliased to bitwise.& aliased to default.&

## Notes

The reason why there is no way to import "selectors associated with a
specific class": if two classes share the same extension both would
misleadingly be brought in even if only one was requested.

The reason why prefix does not include class, such as in
`bitwise.Integer.&`: allow implementing multiple classes using the
same extension in a single module.

## Use case probing

Module A defines Collection, providing the dispatch method at.Collection.

Module B defines Box

Module C wants to use Box (contents) to address Collection.

c.foo:

    import a.Collection
    extend at
    extend at.Collection

    -- this is c.at --> c.at.Collection: !
    -- The latter aliases to at.Collection for existing defs.
    method Collection at: pos
        pos at.Collection: self
    end

    method Box at.Collection: collection
        collection at: self value
    end

Now in this file, or elsewhere after `import c.at:` boxes are opened
when used as collection keys.

