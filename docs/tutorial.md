# Tutorial

_Foolang &mdash; the Foo programming language_

Assuming you have Foo downloaded and installed, let's get started!

Normally you will interact with Foo using it's own development environment,
but let's keep things simple at first.

## Hello World

```
@main: env
  env output println: "Hello World!"
```

Stick that in file called `hello.foo`, and run it using `foo hello.foo`.
If everything goes well it should output `Hello World!` and exit.

### Main Program

`@main: <name> <body>` specifies the main program for Foo.

The `<name>` (here 'env') will be bound an environment object
which provides access to not only command-line arguments, but operating
system facilities like the standard output.

### First Messages

Let's walk through the main program body.

`env output` sends the message `output` to the value of `env` (the
environment object), which responds by returning the standard output.

Messages are chained: the following `println: "Hello World!"` sends the
message `println:` with the argument `"Hello World!"` to the result of
the previous expression &mdash; ie. the standard output.

With nothing else left to do the main program exits.

### Lessons

`object message` is the general pattern for messages that don't
take any arguments, called unary messages.

`object key1: arg1 key2: arg2` is the general pattern for messages with
one or more arguments, called keyword messages. Note that this is a _single_
message `key1:key2:`, not `key1:` message followed by `key2:` message.

## First Class

In addition to `@main` other things can appear at toplevel. The most
important are `@class` and `@method`.

Open a file called `greeter.foo` and insert the following code.

```
@class Greeter { input, output }
```

This is a class definition. It specifies that there is a class called
`Greeter` which has instance variables called input and output.

Given this definition we can send the Greeter class the message `new`
to gain access to a constructor which will take the keyword message
`input:output:` to construct an instance of Greeter.

Add:
```
@method Greeter name
   output println: "What is your name?"
   input readline

@method Greeter nickname
   output println: "What is your nickname?"
   input readline
```

This defines the methods `name` and `nickname` on instances of Greeter. As you
can see instance variables of Greeter can be directly accessed.
By default return value is the value of the last expression evaluated,
here the lines read from input.

Add:
```
@method Greeter greet: name aka: nick
   output println: "Hello: {name}, also known as {nick uppercase}"
```

We're defining a keyword method with two arguments. The name of the method is
`greet:aka:` and the arguments will be bound to variables `name` and `nick`.

The method body contains an interpolated string that is printed.

Add:
```
@method Greeter run
    Do loop: {
      self greet: self name aka: self nickname
      output println: "Next!"
    }
```

The braces enclose a _block_. If you're familiar with closures, that's
what blocks are -- if you're not, don't worry.

`Do` is a global object hosting a number of control flow messages. Sending
it the `loop:` message initiates an infinite loop: it will execute the
block passed to it ad infinitum.

Here we also see `self`, which allows object to send messages to themselves.

Finally, add:
```
@main: env
  Greeter new
    input: env input
    output: env output
  -- run

```

Like before, `env` gives us access to the operating system environment,
so we pass `Greeter` the input and output it needs.

The doubledash is a chaining separator. Because unary messages have
a higher precedence than keyword messages, if we had `run` immediately
after `env output` it would send the `run` message to the standard output,
not the Greeter!

The chaining separater has a lower precedence than any message, so sticking
it in between will cause the `run` message to be sent to the newly
constructed Greeter instead.

You can think of it as meaning "completely evaluate everything on the
left side, then send the resulting object the message on the right side."

### Lessons

`@class ClassName { instance, variable, names }` is the general pattern
for defining classes.

`ClassName new instance: 1 variable: 2 names: 3` is the general pattern
for constructing instances of classes.

`@method ClassName message ...` is the general pattern for defining unary
methods.

`@method ClassName key1: arg1 key2: arg ...` is the general pattern for
defining keyword methods.

`"This is an {interpolated} string."` is the general pattern for interpolated
strings.

`{ code here }` is the general pattern for a parameterless block.

`{ :arg1 :arg2 | arg1 with: arg2 }` is the general pattern for a block
with parameters.
