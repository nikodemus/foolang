# About

Foolang is what happens when former Common Lisp compiler hacker
starts writing a Smalltalk-like language after thinking a lot
about concatenative languages and Erlang.

As is normal for new languages, Foolang aspires to unrealistic goals:

- **Excellent ergonomics.** Code should be a pleasure to write and
  easy to read.
- **Competitive performance.** Programs that do not require late binding
  features should perform on par with -O0 compiled C++ programs. (After that
  it's a question of having a serious compiler instead of a halfway decent one.)
- **Dynamic development.** No one wants to wait for the compiler: being
  able to change a single method and immediately see the effect on a running
  program is the way things should work.
- **Opt-in static analysis.** The compiler does not require you to
  prove your code correct, but you should be able to ask the compiler to do so.

Foolang will be open source, but is still in early development: this website
mostly exists to squat on the name.

Today Foolang consists of a bootstrap evaluator written in Rust, and
piles of design notes.

First public release is intended around the beginning of 2020, but that is
still going to be a long way from 1.0.

## Syntax

Syntax has Smalltalk influence, but has drifted quite far from it.

_Comments_

```
# Hash-mark starts a line comment.
```

_Unary messages_

```
# Unary messages have the highest precedence and chain themselves
# naturally. Unary message names can contain letter and numbers
# and underscores, but must start with a letter.
object message

object message1 messageToResultOfMessage1
```

_Keyword messages_

```
# Keyword messages have a lower precedence. This is the message
# foo:bar: with arguments x and y. Keyword messages use the same
# naming rules as unary, with the addition of colon terminating each
# part.
object foo: x bar: y

# Because of their lower precedence keyword message arguments can take
# unary messages without the need for parentheses.
object foo: x msgToX
       bar: y msgToY

# Keyword messages can be chained with other messages using the -- operator.
# If the operator was omitted the messageToResult would be sent to y instead.
object foo: x bar: y -- messageToResult
```

_Binary messages_

```
# Binary messages use symbols instead of names and follow usual
# precedence rules amongs them selves, but bind weaker than unary
# messages and stronger than keyword messages.

1 + 2 * 3 # => 7

sin pi + 1 # => 1

object foo: x + 1 bar: y + 1

# Chaining operator -- can be also used to chain binary and unary messages
# without need for parentheses.
x == y -- assert
```

_Braces create blocks (closures)_

```
collection do: { :elt | output print: elt toString }

# Blocks without parameters respond to the 'value' message.
# Return value is the last expression evaluated.
{ 1 + 1 } value # => 1

# Blocks with more parameters take them as unnamed arguments
# to the value: keyword message.
{ :x | x + 1 } value: 41 # => 42
{ :x :y | x + y } value: 21 : 21
```

_Class definition and instantiation_

```
# Definition of class Foo with three instance variables: a, b, and c, of which
# c has the default value of 42.
@class Foo { a, b, c: 21 + 21 }

# ClassName new returns the default constructor which responds to
# messages matching instance variable names. Those with defaults can be omitted.
Foo new a: 1 b: 2
```

_Methods definition_

```
# Method on class Foo with selector a:b: and arguments aval and bval.
# Wrapping the default constructor.
#
# Methods return the value of the last expression.
@classMethod Foo a: aval b: bval {
    Foo new a: aval b: bval
}

# Methods a b and c on instances of class Foo, reading instance variables.
@method Foo a { a }
@method Foo b { b }
@method Foo c { c }

# ^ returns the value of the following expression from the method
# in which it appeared. To be specific: return embedded in a block does
# not return from the block, but from the method. It is an error to
# return from a method which has already returned. (There will be
# compiler support for checking this statically.)
@method Foo max {
  a >= b & a >= c then: { ^ a }
  b >= a & b >= c then: { ^ b }
  c
}

# Addition: Foo+Foo, Foo+Number, and Number+Foo.
#
# Using Smalltalk-style double-dispatch for now, considering
# alternatives.
#
# ...but for now, as demonstrated, extending existing classes
# is allowed. However, extension methods
# - must be instance methods.
# - cannot access instance variables directly.
# - cannot replace existing methods.
# - are primarily associated with the class that provided them,
#   not the class they extend.
@method Foo + arg {
    arg addToFoo: self
}

@extension Foo
@method Number addToFoo: foo {
    foo addToNumber: self
}

@method Foo addToFoo: foo {
    Foo a: a + foo a
        b: b + foo b
        c: c + foo c  
}

@method Foo addToNumber: n {
    Foo a: a + n
        b: b + n
        c: c + n
}
```

_Unnamed keyword arguments_

```
# Keyword messages can also be defined without naming the individual arguments.
# Spaces are significant here!
@method Foo bar :x :y :z {
    a * x + b * y + c * z
}

# Sending the corresponding message, again spaces are significant here!
foo bar: 1 : 2 : 3
```

_Local variables, assignment, equality_

```
# Let creates a local variable binding. Assignment uses :=
# = is un-overridable identity comparison. == is overridable
# class specific equality.
let x := 1
x := x + 1
x == 2.0 -- assert: "For numbers == uses numeric equality, so this is ok."

# := is also used to assign to instance variables.
@method Foo inc {
    a := a + 1
    b := b + 1
    b := c + 1
}
```

_Type annotations are either proven to hold or asserted at runtime_

```
# Values can be annotated
x::Float + y::Float

# Method return values can be annotated
@method Foo sum -> Number {
    a + b + c
}

# Method parameters can be annotated
@method Foo a: newa::Number
            b: newb::Number
            c: newc::Number
  -> Foo
{
    a := newa
    b := newb
    c := newc
    self
}

# Local variables can be annotated
let x::Float := 42.0

# Block parameters can be annotated
{ :x::Float| x*2.0 }
```

_Semicolon cascades messages_

```
# Value that the expression before the first semicolon evaluates
# to becomes the object on which the cascade is run.
#
# The chain of messages following each semicolon starts with the
# cascade object. If a message returns another value the chain
# continues with it.
#
# Each semicolon in a cascade starts again from the cascade object.
#
# Cascade is terminated when the next expression starts. Value of
# the cascade is the value of the last chain of messages.
expr to create an object
 ; messageToIt key: word -- chained
 ; anotherMessage
nextExpr
```

_Expressions can be separated by newlines or commas_

```
# The rules are fairly DWIM:
# - Newline separates expressions if and only if the expression
#   on the previous line is syntactically complete and the following
#   line starts with primary expression.
#
# - In practice this means that keyword and binary messages can be split
#   across lines, but unary messages cannot.
#
# - \ at the end of a line indicates that the expression continues on
#   the next line.
@method Foo printTo: stream {
    # Single expression on two lines: 'print:' is not a primary expression
    # so the newline has no effect.
    stream
      print: "<Foo "
    # Two expressions separated by newlines.
    stream print: a toString
    stream print: ", "
    # Continuation marker needed because toString at the start of
    # line _could_ be ment as a variable reference.
    stream print: b \
      toString
    # This way of splitting is legal too: 'print:' takes an argument
    # so the expression is not complete.
    stream print:
      ", "
    # Using a comma we can put multiple expressions on a single line.
    stream print: c toString, stream print: ">"
    ^self
}
```

_Brackets create arrays at runtime_

```
# Commas separate the elements.
[1, 1+1, 6/2]

# Immutable literal arrays are prefixed with $ and can contain
# only literals.
$[1, 2, 3]
```

## Roadmap

1. Bootstrap evaluator for the core language. **done**
2. Full syntax support. **wip**
3. Minimal smalltalk-style IDE.
4. VM and bytecode compiler.
5. Native compiler.

## Design Intentions

_Random sampling for the curious._

Globals are immutable.

Threading is based on agent model. Every agent has a fully isolated
heap, and as such can be killed with impunity. This also means there
is no stop-the-world GC pause.

Initial GC is mark-and-sweep, but with automatic stack allocation when system
can prove the value does not escape, aided by (proven) annotations.

Main entry point receives System object as argument, which
provides access to operating system facilities. Other methods can access
these facilities only they are passed them: there is no ambient authority.

Value types so abstraction does not mean indirection.

Support for asking compiler to prove various things:
- This method does not allocate.
- This method never unwinds.
- This method is type-safe.
- This method is pure.
- This method mutates only self.
- This method does no dynamic dispatch. (Classes known.)
- This method does no method lookups. (Interfaces known.)

---

Stay tuned!
