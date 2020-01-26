# Foolang

!> This is a toy language implemented by a single person&mdash;use for quiet enjoyment only.

**_The Foo Programming Language_**

Foolang is a Smalltalk-inspired language that, like all new languages,
has what you might generously call _somewhat optimistic aspirations_:

- Elegance and power of Smalltalk
- Performance of C++
- Fault tolerance of Erlang

**_"Are you there yet?"_**

Hah, not by a long shot.

**_"When are you going to get there?"_**

Probably never: this project is more about play and exploration than burning
midnight oil to hit a milestone.

## Hello World

``` foolang
class Main {}
    class method run: command in: system
        system output println: "Hello world!"
end
```

## Getting Started

!> Sorry, you can't yet&mdash;the repository is still waiting for a couple
of final touches like LICENSE files.

1. Install [Rust](https://www.rust-lang.org/tools/install).
2. Clone the foolang repository:
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
**No ambient authority**: all OS interfaces must be passed to through dependency
injection, starting from the system object passed to `Main##run:in:`&mdash;there is
no such thing as a globally accessible `File` or `Socket` class.

<span class="done">&check;</span>
**Optional typing & typechecks**: all expressions can
be annotated with types, including method parameters and return values.
Currently these types are checked at runtime. 

<span class="done">&check;</span> **Interactive development**: Foolang supports
a dynamic and interactive way of working: in development mode existing methods
can be redefined and new classes added while the program is running.

<span class="done">&check;</span>
**Lexical closures**: Foolang has Smalltalk-style blocks that are full
closures. Currently class definitions and methods can only appear at the
top level and as such methods are never closures in that sense. 

<span class="done">&check;</span>
**No inheritance of implementations**: similar to Julia and Rust, concrete
instantiable classes cannot be inherited from in Foolang.

### Pending

<span class="todo">&cross;</span>
**Multiple inheritance of interfaces**: interfaces and classes can inherit
from multiple interfaces.

<span class="todo">&cross;</span>
**Dynamic bindings**: dynamic bindings combined with lexical closures
allow powerful error handling to be implemented in user code,
implementation of interesting paradigms like context oriented programming,
and make dependency injection easier.

<span class="todo">&cross;</span>
**Extensible syntax & code generation**: Foolang is intended to offer a
compile-time computation facility similar to Lisp's macros, allowing both
syntactic convenience and ability to generate code.

<span class="todo">&cross;</span>
**Type inference**: while since Foolang's typesystem should be considered
weaksouce by today's standards doing basic type inference is critical for
the intendend functionality.

<span class="todo">&cross;</span>
**Compiled**: once boostrap implementation and the language have stabilized
sufficiently, Foolang efforts will focus on a native compiler producing
monolithic executables&mdash;but this must not compromise the interactive
development experience.

<span class="todo">&cross;</span>
**Performant**: type-annotated and compiled Foolang code should perform about as well
as equivalent `-O0` C++ code. To be fair: this will require more effort from
the compiler than C++, but not drastically so&mdash;a partial evaluation pass
should cover most of it.

<span class="todo">&cross;</span>
**Supervised and isolated threads**: Foolang threads will not share memory,
hopefully providing a subtrate for fault tolerant computing akin to what Erlang
does. (Unlike Erlang Foolang does allow thread-local side-effects.)

<span class="todo">&cross;</span>
**Smalltalkish development environment**: while Foolang keeps it's code in files
and allows you to use your favorite editor, it still wants to provide an
integrated experience similar to Smalltalks.
