# Syntax

delimiters := , ; ( ) { } [ ]

  these will only ever tokenize to a single character

literal-prefix := #

  affects the parsing of the following syntactic element

dynamic-binding-prefix := $

  Identifier prefix

line-comment := --

identifier := _?[:alphabetic:][:alphanumeric:]*[?!']*?

  GENERALLY identifiers in value position are evaluated as local
  variables or global constants.

  The system provides parsers for some identifiers which appear
  in value positions:

   - return
   - let

  In value position identifiers which have a case must be local if
  lowercase and global if uppercase or titlecase.

        -- local
        foo

        -- global
        Foo

        -- either
        测试
        _foo

   Identifiers in operator position are evaluated as unary messages to
   the value.

        -- message bar to value of foo
        foo bar

        -- message 跑 to 测试
        测试 跑

   Outside development mode messages prefixed with underscore are
   allowed only to self.

        -- Syntax error outside development mode
        foo _bar

        -- Always legal
        self _bar

sigils := [:non-alphabetic:]+

   Sigils in value position are evaluated as prefix operators.
   Prefix operators _must_ precede their arguments without
   intervening whitespace.

   Sigils in operator position are evaluated as postfix or binary
   operators.

   Postfix operators _must_ follow their arguments without intervening
   whitespace.

   Binary operators _must_ have balanced whitespace, and whitespace _must_
   match precedence:

        -- Legal, precedence matches whitespace
        x*y + b

        -- Legal, precedence does not conflict with whitespace
        x * y + b

        -- Illegal, precedence conflicts with whitespace
        x * y+b

   Sigils beginning with . : and = are served for the implementation

        -- Type annotation
        foo::Float

keyword := _?[:alphabetic:][:alphanumeric:]*: (must be followed by whitespace!)

   Keywords in value position are not allowed.

   Keywords in operator position are parts of keyword messages.

        -- Message bar:quux: with arguments 1 and 2 to foo
        foo bar: 1 quux: 2

The only truly global reserved words are:

    import
    from
    as
    done
    self

Whitespace and , are also fully built-in.

Rest of Foolang needs to be imported:

    import foolang.v0.*

Importing while keeping the package prefix:

    import foolang.v0

Partial imports and aliasing:

    from foolang.v0 import
       let
       return as ret
    done

Toplevel definitions:

    -- Core --

    define Foo = 12

    -- Sugars --

    define Foo {}
       method bar 42
    done

    -- desugars to:

    define Foo = class {}
       method bar 42
    done <name: Foo>

    define Foo(a, b) { a + b }

    -- desugars to:

    define Foo = { |a,b| a + b } <name: Foo>

Built-in parser words:

    is

        a is b ==> True IFF a is the same object as b

    let 

        let a = b, ... ==> local variable binding for a as b in the sequence
        that follows.

    return

        return a ==> returns a from current frame.  All methods have an implicit frame,

    __do__

        do { ... } establishes an explicit frame for the enclosed code: return inside
        the lexical scope of the block terminates do, not the method.

    __block__ ... done

        What { ... } desugars into.

    __array__ ... done

        what [ ... ] desugars into.

    __expr__ ... done

        what ( ... ) desugars into.

