# Foolang

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

!> This is a toy language implemented by a single person&mdash;use for quiet enjoyment only.

**_The Foo Programming Language_**

Foolang is a Smalltalk-inspired language that, like all new languages,
has somewhat _optimistic aspirations_:

- **_Elegance and power of Smalltalk and Self:_** Smalltalk-like syntax, deep object
  orientation, and late binding as default.

- **_Performance of C++:_** Support for early binding when you need it, so that
  the compiler can go to town. Low-level operations which allow eliding overflow
  checking in tight loops, etc.

- **_Fault tolerance of Erlang:_** Agent-model, isolated heaps, and supervisors.
  No undefined behavior.

**_"Are we there yet?"_**

Nope.

Syntax is still settling down, early binding support isn't quite there, the
compiler is an work-in-progress trivial transpiler, agents and threads haven't
even been started yet, and many things which should be first class objects
aren't yet, etc.

**_"When we going to get there?"_**

Someday, I hope!

## Hello World

``` foolang
class Main {}
    direct method run: command in: system
        system output println: "Hello world"!
end
```

## Getting Started

1. Install [Rust](https://www.rust-lang.org/tools/install).
2. Clone the Foolang repository:
   ``` shell
   git clone https://github.com/nikodemus/foolang.git
   ```
3. Build Foolang & run the REPL:
   ``` shell
   cd foolang
   cargo run -- foo/repl.foo
   ```
4. Read the [syntax](syntax.md#foolang-syntax) document, read the code, play around.

## Features & Status

### Done

<span class="done">&check;</span>
**Lexically scoped object language**: everything is an object. Names are
scoped lexically. Smalltalk-style blocks are lexical closures.

<span class="done">&check;</span>
**Ergonomic syntax**: while Foolang syntax is not _quite_ as minimal as
Smalltalk's it is very simple and ergonomic.

<span class="done">&check;</span>
**No ambient authority**: all OS interfaces must be passed explicitly, starting
from the [system object](system.md) passed to `Main##run:in:`&mdash;without it
there is no way to open a file or a socket, check the clock, or run an external
program. (Note: the system object doesn't contain nearly all facilities it
should yet!)

<span class="done">&check;</span>
**Reified types**: Types like _Integer_ are runtime objects capable of
responding to messages.

<span class="done">&check;</span>
**Optional typing & typechecks**: all expressions can
be annotated with types, including method parameters and return values.
Currently these types are checked at runtime.

<span class="done">&check;</span>
**Multiple inheritance of interfaces**: classes can inherit from multiple
interfaces, which can both provide default implementations for methods, and
require other methods to be implemented by the class. Interfaces can also
inherit from other interfaces. Foolang interfaces are fairly similar to traits,
but do not yet support explicit conflict resolution. (See
[Schärli2003](bibliography.md#scharli2003).)

<span class="done">&check;</span>
**No inheritance of implementations**: similarly to Julia and Rust, concrete
instantiable classes cannot be inherited from in Foolang.

<span class="done">&check;</span>
**Non-local returns**: Foolang supports Smalltalk-style non-local returns
from closures, allowing simple implementations for powerful control structures.

<span class="done">&check;</span>
**Dynamic bindings**: dynamic bindings combined with lexical closures
allow powerful error handling to be implemented in user code,
implementation of interesting paradigms like context oriented programming,
and make dependency injection easier.

<span class="done">&check;</span>
**Condition system**: Foolang' exception/error handling is inspired by
Common Lisp's acclaimed condition system: errors can be handled without
unwinding the stack when appropriate.

<span class="done">&check;</span>
**Interactive development**: Foolang supports
a dynamic and interactive way of working: in development mode existing methods
can be redefined and new classes added while the program is running.

<span class="done">&check;</span>
**Self-Hosted**: Foolang is implemented in Foolang: it has a self-hosted parser,
interpreter, and a transpiler-to-C, making it capable of building itself.
(Bootstrap is currently through an interpreter written in Rust.)

### Pending

<span class="todo">&cross;</span>
**Compiled**: Foolang is intended to be capable of producing native, monolithic
executables&mdash;without compromising the interactive development experience.
The current compiler does produce monolithic executables, but there's no
interactive experience with the compiler yet.

<span class="todo">&cross;</span>
**Performant**: type-annotated and compiled Foolang code should perform about as
well as equivalent `-O0` C++ code. To be fair: this will require more effort
from the compiler than C++, but not drastically so&mdash;a partial evaluation
pass should cover most of it. Current compiler is *definititely* not there yet:
it's ~7 x slower than the bootstrap evaluator at the moment!

<span class="todo">&cross;</span>
**Type inference**: while Foolang's typesystem should be considered
weaksauce by today's standards, doing basic type inference is critical for
the intendend functionality.

<span class="todo">&cross;</span>
**Supervised and isolated threads**: Foolang threads will not share memory,
hopefully providing a subtrate for fault tolerant computing akin to what Erlang
does. (Unlike Erlang Foolang does allow thread-local side-effects.)

<span class="todo">&cross;</span>
**Smalltalkish development environment**: while Foolang keeps it's code in files
and allows you to use your favorite editor, it still wants to provide an
integrated experience similar to Smalltalks.

<span class="todo">&cross;</span>
**Extensible syntax & code generation**: Foolang is intended to offer a
compile-time computation facility similar to Lisp's macros, allowing both
syntactic convenience and ability to generate code.
