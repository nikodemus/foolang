# Extension Methods

Extension methods are added using toplevel method definitions:

    method ClassName selector
          body
    end

Extension methods that are explicitly prefixed with the name of the
current module can be added to any existing class.

By declaring extension selectors using `extension` section the
prefixes can be elided, and these selectors become aliases for the
extended selectors.

Eg. in file bitwise.foo:

    extension bitOps
        & | << >> ~
    end

    -- Prefix can be elided due to 'extension'
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

Using importExtension allows eliding the prefix from all methods in
a declared extension:

    importExtension bitwise.bitOps
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

Note: this includes bitwise.andInteger: selector.

If `default.&` were not explicitly defined, then regular `&` would not
be defined: extensions forward to vanilla methods, but not the wise versa.

When `bitwise.&` hits `doesNotUnderstand` it gets that instead of
`default.&`. This is so that delegation works properly -- when it
finally ends at an object that implements either, then aliasing takes
care of the rest. If for some reason someone wants to dispatch on the
selector text, then they can use `Selector name`

    'bitwise.&' name == "&" --> True

Extensions can be chained:

bitwise2.foo:

     importExtension bitwise.bitOps

     extension bitOps2
       & | << >> ~
     end

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

Then:

    importExtension bitwise2.bitOps2

    bitvec1 & bitvec2 --> bitvec

    int1 & int2 --> int

    bool1 & bool2 --> bool

all work:

- the first goes directly to the new extension methods
- the second find bitwise2.& aliased to bitwise.&
- the third find bitwise2.& aliased to bitwise.& which was already aliased to default.&

## Use case probing

Module a:

      extension collectionOps
         atCollection
      end

      class Collection
         method at: pos
           pos atCollection: self
      end

      method Integer atCollection: collection
        collection atIndex: self
      end

      method Array atCollection: collection
        collection atIndices: self
      end

Module a:

      class Box { value }
      end

Module c wants to use Box value to address Collection.

boxkeys.foo:

    import a.Collection
    importExtension a.collectionOps

    extension boxkeyOps
        at atCollection
    end

    method Collection at: pos
        pos atCollection: self
    end

    method Box atCollection: collection
        collection at: self value
    end

c.foo:

    import a.Collection
    importExtension boxkeys.boxkeyOps

    coll at: box
    coll at: int
    coll at: array

all now work as expected?

Win?

## Overhead

There's space bloat in vtables, but no runtime overhead.

## More Thinking

- It seems to me that the "extension" stuff is a level up. Ie. it's not
  superclear that it's a semantic requirement instead of a convenience.

- Even if it is a convenience I think a BIT more explicitness is needed.
  Looking at the extension in boxkeys.foo after the fact it is not clear
  to me what is going on.

  Something like:

      extension boxkeyOps
          at -> default.at,
          atCollection -> a.collectionOps.atCollection
      end

- Overall I like the way using names + forwarding seems to make both
  the dynamic and lexical behaviour make sense.
