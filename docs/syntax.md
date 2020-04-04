# Foolang Syntax

Foolang is expression oriented: every expression has a value.

## Aesthetic

The syntax tries to minimize the amount of visual noise from puctuation, and be
easy to read out loud and understand when read out loud: order of operations
should match reading order.

## Reserved Words

Following words have special meaning:

``` foolang
class   extend  method    return
let     import  panic
end     is      required
```

This restricts their use as both messages and as variables.

All other reserved words except `is` are used in prefix position.

## Comments

Line comments are prefixed with `--`:

``` foolang
-- This is a comment
```

Block comments are surrounded by `---`:

``` foolang
output println: --- This is a comment --- "Hello!"

---
This is a comment
Still a comment
---
```

End of line comments describing the value the line evaluates to are
conventionally prefixed with `-->`, but syntactically this is just a
line comment that starts with a greater-than sign:

``` foolang
x = y + z --> sum of y and z
```

Similarly end of line comments describing an error are conventionally
prefixed with `--|`.

``` foolang
4 append: 2 --| ERROR: 4 does not understand append: 2
```

## Literals

Numbers are quite conventional, with `_` as allowed readability separator.
Literal floats are currently always doubles.

``` foolang
1_000_000
0xDEADBEEF
0b011011
1.0     -- double
1.0e0   -- double
```

Strings allow interpolations to be embedded:

``` foolang
"The Answer Is: {40 + 2}!"
```

The interpolation is evaluated in the current lexical environment.

Escape sequences usable in strings:

``` foolang
"\n" -- linefeed
"\r" -- carriage return
"\t" -- horizontal tab
"\"" -- doublequote
"\\" -- backslash
"\{" -- open brace
```

There are currently no character objects: the elements of a multicharacter
string are single character strings.

## Messages

Messages are sent by simple concatenation, Smalltalk-style.

Unary prefix message with selector `-` to `x`:
``` foolang
-x
```

Unary suffix message with selector `foo` and no arguments to object:
``` foolang
object foo
```

Binary message with selector `+` and argument `y` to `x`:
``` foolang
x + y
```

Keyword message with selector `foo:bar:` and arguments 1 and 2 to object
``` foolang
    object foo: 1 bar: 2
```

Message chaining is also simple concatenation:
``` foolang
object foo bar quux: 42
```
The above is:
1. Message `foo` to `object`.
2. Message `bar` to response of message #1.
3. Message `quux:` with argument 42 to response of message #2.

Message chains with keyword messages in the middle need parenthesis.

``` foolang
-- Without the parenthesis "bar" would be sent to 42 and not the
-- response of "quux:".
(object foo quux: 42) bar 
```

Similarly for chaining multiple keyword messages:

``` foolang
-- Without the parenthesis this would a single "quux:quux:" message,
-- instead of two messages.
(object quux: 1) quux: 2
```

There is currently no literal selector syntax. Using a punctuation character to
allow chaining keyword messages without parenthesis is being considered, but it
may be that long chains of messages may be an anti-pattern.

## Precedence Rules

Unary prefix messages have the highest precedence.

Unary suffix messages have the second highest precedence.

Binary messages as a group have the third highest precedence. Amongst
themselves they have conventional precedence unlike in Smalltalk:

1. `*`
2. `/`
3. `+ -`
4. `< <= > >= ==`
5. All other non-alphabetic message operators.

!> Current precedence rules and implementation is a placeholder: Foolang is
intended to have non-transitive user-definable operator precedence.

Keyword messages have the lowest precedence.

## Compound Expressions

Expressions are separated form each other with full stops ("dots"),
Smalltalk-style:

``` foolang
-- Message with selector "foo" to objectA, message with selector "bar" to
-- objectB. Newline can be elided.
objectA foo.
objectB bar
```

A sequence of expressions like this is an expression that evaluates to
the value of the last subexpression.

?> Foolang very much wanted to use newlines as separators, but it turns out
that that there are far too many places where a human would want to enter
a newline that doesn't terminate the expression. If things change and stars
align that might still happen.

## Local Variables

Local variables are defined using `let`:

``` foolang
let x = 42.
x * 100 --> 420
```

Local variables can be assigned to using `=`:

``` foolang
let x = 42.
x = x - 2.
x --> 40
```

## Blocks

Unlike Smalltalk Foolang uses braces for blocks:

``` foolang
{ 1 + 2 } value --> 3
```

Blocks can take arguments:

``` foolang
{ |x| x + 2 } value: 40 --> 42

{ |x y| x + y } value: 40 value: 2 --> 42
```

## Records

Records are objects that respond to messages corresponding to their
fieldnames.

``` foolang
let value = 2.
let coords = { x: value, y: value }.
coords x: 40.
coords x + coords y --> 42
```

!> Records are currently immutable.

## Dictionaries

``` foolang
let key = "foo".
let value = "lang".
let dict = { key -> value }.
dict at: "foo" --> "lang"
dict at: "foo" put: "Foolang".
dict at: "foo" --> "Foolang".
```

!> Dictionary syntax is not implemented.

## Class Definitions

``` foolang
class Point { x y }
   method + other
      Point x: x + other x
            y: y + other y.
   method displayOn: stream
      stream print: "#<Point {x,y}>".
end
```

Implicit constructor using the slot names:

``` foolang
let p0 = Point x: 1 :y 2     --> #<Point 1,2>
let p1 = Point x: 100 y: 200 --> #<Point 100,200>
let p2 = p0 + p1             --> #<Point 101,202>
```

Slot values are not accessible externally without
accessor methods:

``` foolang
p0 x --| ERROR: #<Point 1,2> does not understand x
```

## Type Annotations

Double-colon suffix is used for type annotations.

``` foolang
expression::Type

let variable::Type = value

class Foo { slot::Type }
    method bar: argument::Type -> ReturnType
        ...
end

let block = { |arg::Type| ... } -> ReturnType
```

Intersection types can be expressed by chaining assertions:

``` foolang
-- Asserts that `x` is both T and K
x::T::K
```

## Arrays

Are constructed using square brackets

    let array = [1, 2, 3]

## Module Import

Modules are units of import: files and directories of Foolang code. Names from
other modules can be imported to current lexical environment.

Import can be of three forms:
1. Module import: `import coolmodule`. This makes global names in coolmodule
   visible in the current environment with a `coolmodule.` prefix.
2. Name import: `import coolmodule.CoolClass`. This makes `CoolClass`
   visible in the current environment without a prefix.
3. Star import: `import coolmodule.*`. This makes all names in `coolmodule`
   visible in the current environment without a prefix.

Prefixing the import specification with a dot makes it a relative import,
looking for the module relative to current file.

``` foolang
import .myutils.*
```

Otherwise the first part of the import specification must match the final
part of a path provided to Foolang with the `--use` option.

Eg.
``` shell
foo --use /path/to/bar.foo
```

makes the following import the module from `/path/to/bar.foo`.
``` foolang
import bar
```

This applies to hierarchies as well. Eg.
``` shell
foo --use /path/to/bar
```

makes the following import the module from `/path/to/bar/quux.foo`.
``` foolang
import bar.quux
```
