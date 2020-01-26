# Foolang Syntax

Foolang is expression oriented: every expression has a value.

**Reserved Words**

Following words have special meaning:

    let      extend    return
    class    import
    end      method

This restricts their use as both messages and as variables.

**Comments**

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

    x = y + z --> sum of x and y

Similarly end of line comments describing an error are conventionally
prefixed with `--|`.

    4 append: 2 --| ERROR: 4 does not understand append: 2

**Literals**

Numbers are quite conventional, with _ as allowed readability separators.
Literal floats are currently always doubles.

    1_000_000
    0xDEADBEEF
    0b011011
    1.0     -- double
    1.0e0   -- double

Strings allow interpolations to be embedded:

    "Hello {user name}!"
    "The Answer Is: {40 + 2}!"

**Messages**

Messages are sent by simple concatenation, Smalltalk-style:

    -- Message with selector "foo" and no arguments to object
    object foo

    -- Message with selector "foo:bar:" and arguments 1 and 2 to object
    object foo: 1 bar: 2

**Compound Expressions**

Expressions are separated form each other with dots, Smalltalk-style:

    -- Message with selector "foo" to objectA, message with selector "bar" to
    -- objectB. Newline can be elided.
    objectA foo.
    objectB bar

A sequence of expressions like this is an expression that evaluates to
the value of the last subexpression.

**Operator Precedence**

Unlike Smalltalk operators have conventional precedence:

    1 + 2 * 3 --> 7

**Local Variables**

Defined using let:

    let x = 42.
    x * 100 --> 420

Assigned to using equal sign:

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
