# Foolang Syntax

Foolang is expression oriented: every expression has a value.

## Aesthetic

The syntax tries to minimize the amount of visual noise from puctuation, and be
easy to read out loud and understand when read out loud: order of operations
should match reading order.

## Reserved Words

Following words have special meaning:

    class    extend    method
    let      import    return
    end      is

This restricts their use as both messages and as variables.

All other reserved words except `is` are used in prefix position.

## Comments

Line comments are prefixed with `--`:

    -- This is a comment

Block comments are surrounded by `---`:

    output println: --- This is a comment --- "Hello!"

    ---
    This is a comment
    Still a comment
    ---

End of line comments describing the value the line evaluates to are
conventionally prefixed with `-->`, but syntactically this is just a
line comment that starts with a greater-than sign:

    x = y + z --> sum of y and z

Similarly end of line comments describing an error are conventionally
prefixed with `--|`.

    4 append: 2 --| ERROR: 4 does not understand append: 2

## Literals

Numbers are quite conventional, with `_` as allowed readability separator.
Literal floats are currently always doubles.

    1_000_000
    0xDEADBEEF
    0b011011
    1.0     -- double
    1.0e0   -- double

Strings allow interpolations to be embedded:

    "The Answer Is: {40 + 2}!"

The interpolation is evaluated in the current lexical environment.

Escape sequences usable in strings:

    \n \t \r \{ \" \\

There are currently no character objects: the elements of a multicharacter
string are single character strings.

## Messages

Messages are sent by simple concatenation, Smalltalk-style.

Unary prefix messages are non-alphabetic:

    -- Unary prefix message with selector "-" to x
    -x

Binary messages are non-alphabetic:

    -- Binary message with selector "+" and argument y to x.
    x + y

Unary suffix messages are alphabetic: 

    -- Unary suffix message with selector "foo" and no arguments to object
    object foo

Keyword messages are alphabetic:

    -- Keyword message with selector "foo:bar:" and arguments 1 and 2 to object
    object foo: 1 bar: 2

Message chaining is also simple concatenation:

    -- Message #1: "foo" to object.
    -- Message #2: "bar" to response of message #1.
    -- Message #3: "quux:" with argument 42 to response of message #2.
    object foo bar quux: 42

Message chains with keyword messages in the middle need parenthesis:

    -- Without the parenthesis "bar" would be sent to 42 and not the
    -- response of "quux".
    (object foo quux: 42) bar 

Similarly for chaining multiple keyword messages:

    -- Without the parenthesis this would a single "quux:quux:" message,
    -- instead of two messages.
    (object quux: 1) quux: 2

There is currently no literal selector syntax. Using a punctuation character to
allow chaining keyword messages without parenthesis is being considered, but it
may be that long chains of messages may be an anti-pattern.

**Precedence Rules**

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

**Compound Expressions**

Expressions are separated form each other with full stops ("dots"),
Smalltalk-style:

    -- Message with selector "foo" to objectA, message with selector "bar" to
    -- objectB. Newline can be elided.
    objectA foo.
    objectB bar

A sequence of expressions like this is an expression that evaluates to
the value of the last subexpression.

**Local Variables**

Local variables are defined using `let`:

    let x = 42.
    x * 100 --> 420

Local variables can be assigned to using `=`:

    let x = 42.
    x = x - 2.
    x --> 40

**Blocks**

Unlike Smalltalk Foolang uses braces for blocks:

    { 1 + 2 } value --> 3

Blocks can take arguments:

    { |x| x + 2 } value: 40 --> 42

    { |x y| x + y } value: 40 value: 2 --> 42

**Class Definitions**

    class Point { x y }
       method + other
          Point x: x + other x
                y: y + other y
       method displayOn: stream
          stream print: "#<Point {x,y}>"
    end
    
Implicit constructor using the slot names:

    let p0 = Point x: 1 :y 2     --> #<Point 1,2>
    let p1 = Point x: 100 y: 200 --> #<Point 100,200>
    let p2 = p0 + p1             --> #<Point 101,202>

Slot values are not accessible externally without
accessor methods:

    p0 x --| ERROR: #<Point 1,2> does not understand x

**Type Annotations**

Double-colon suffix is used for type annotations.

    expression::Type

    let variable::Type = value

    class Foo { slot::Type }
       method bar: argument::Type -> ReturnType
          ...
    end

**Arrays**

Are constructed using square brackets

    let array = [1, 2, 3]
