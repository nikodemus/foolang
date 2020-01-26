# FOOLANG

**_The Foo Programming Language_**

Foolang is a Smalltalk-inspired language that, like all new languages,
has _somewhat_ unrealistic aspirations:

- Elegance and power of Smalltalk
- Performance of C++
- Fault tolerance of Erlang

**_"Are you there yet?"_**

Hah, not by a long shot. Foolang is nothing but a toy. Treat it as such.

**_"When are you going to get there?"_**

Probably never: this project is more about play and exploration than burning
midnight oil to hit a milestone.

## Hello World

    class Main {}
        class method run: command in: system
            system output println: "Hello world!".
    end

## Getting Started

1. Install [Rust](https://www.rust-lang.org/tools/install).
2. Clone the foolang repository:
       git clone https://github.com/nikodemus/foolang.git
3. Build Foolang & run the REPL:
       cd foolang
       cargo run -- foo/repl.foo
4. Read the [syntax](syntax.md#foolang-syntax) document, read the code, play around.

## How Will Foolang Be Different?

**Legend**: done = <span class="done">&check;</span>, todo = <span
class="todo">&cross;</span>.

To be clear: I believe I have a fairly good idea how to implement all of this.
I'm sure there are surprises hiding in the weeds, but the big picture is clear.

<span class="done">&check;</span>
**No ambient authority**: all OS interfaces must be passed to through dependency
injection, starting from the system object passed to `Main##run:in:`: there is
no such thing as a globally accessible `File` or `Socket` class you can
instantiate and use. 

<span class="done">&check;</span>
**Optional typing & typechecks**: all expressions can
be annotated with types, including method parameters and return values.
Currently these types are checked at runtime. 

<span class="done">&check;</span>
**Interactive development**: Foolang supports a dynamic and interactive way of
working: existing methods can be redefined and new classes added while
programming is running in development mode.

<span class="done">&check;</span>
**Lexical closures**: Foolang has Smalltalk-style blocks that are full
closures. Currently class definitions and methods can only appear at the
top level and as such methods are never closures in that sense. 

<span class="todo">&cross;</span>
**Interfaces and classes**: Like Julia and Rust Foolang allows multiple
inheritance of interfaces, but no inheritance of concrete classes.

<span class="todo">&cross;</span>
**Dynamic bindings**: Dynamic bindings + lexical closures combined are
enough to implement Common Lisp -style error handling in user code, allow
context oriented programming, and make dependency injection easier.

<span class="todo">&cross;</span>
**Extensible syntax & code generation**: Foolang is intended to offer a
compile-time computation facility similar to Lisp's.

<span class="todo">&cross;</span>
**Type inference**: since Foolang's typesystem should be considered
weaksouce by today's standards doing basic type inference is trivial.

<span class="todo">&cross;</span>
**Compiled**: once boostrap implementation and the language have stabilized
sufficiently, Foolang efforts will focus on a native compiler producing
monolithic executables&mdash;but this must not compromise the interactive
development experience.

<span class="todo">&cross;</span>
**Supervised and isolated threads**: Foolang threads will not share memory, hopefully providing
a subtrate for fault tolerant computing akin to what Erlang does. (Unlike Erlang
Foolang does allow thread-local side-effects.)

<span class="todo">&cross;</span>
**Smalltalkish development environment**: while Foolang keeps it's code in files
and allows you to use your favorite editor, it still wants to provide an
integrated experience similar to Smalltalks.

## Design Priorities, In Order

1. **Safety**: No memory errors. No race conditions. No ambient authority.
   No undefined behaviour. Fault-tolerant applications.

2. **Ergonomics**: Code should be a pleasure to read and write.

3. **Performance**: Code with type annotations should run on par with -O0
   compiled "equivalent" C or C++.

4. **Uniformity**: Built-in code should not be privileged over user code.

If one of these is violated, that violation should be driven by a higher
priority concern.

## Design Compromises

### No Optional Arguments

**Problem**: occasionally questionable ergnomics, needing to create multiple
methods instead of just one.

**Excuse**: not critical right now, and Smalltalk world doesn't seem to really
miss them.

### No Variable Argument Methods or Blocks

**Problem**: occasionally questionable ergnomics.

**Excuse**: not critical right now, and neither Smalltalk nor Rust worlds seem
to really miss them.

### Unsound Returns

**Problem**:

```
class Foo {}
  class method bad
     { return 42 }
  class method bang
     self bad value
end
```

Ie. one can construct blocks that try to return from frames that have
already returned.

**Excuse**: having returns inside a block return from the lexically
enclosing method allows implementing unwinding control structures
and exception handling in user code. (Esp. when combined with dynamic
bindings.)

**Possible palliative**: `<sound-returns>` pragma could ensure that all
returns are statically known to be safe, and signal a compile-time
error if not.

**Possible alternatives**:
- Implement exceptions and control flows using magic primitives
- Error values instead of exceptions
