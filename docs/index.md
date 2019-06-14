# foolang

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
- **Opt-in static analysis.** While the compiler does not require you to
  prove your code to be correct, you can ask the compiler to prove your
  code.

Foolang will be open source, but is still in early development: this website
mostly exists to squat on the name.

Today Foolang consists of a bootstrap evaluator written in Rust, and
piles of design notes.

First public release is intended around the beginning of 2020, but that is
still going to be a long way from 1.0.

## Syntax

Syntax starts out with Smalltalk, but drifts out:

_Comma sequences expressions_

```
someObject someMessage, anotherObject anotherMessage
```

_Braces create blocks_

```
collection do: { :elt | output print: elt toString }
```

_Brackets create arrays at runtime_

```
[1, 1+1, 6/2]
```

## Roadmap

1. Bootstrap evaluator. **done**
2. Minimal smalltalk-style IDE. **wip**
3. VM and bytecode compiler.
4. Native compiler.

## Design Intentions

_Random sampling for the curious._

Allow also newlines to sequence expressions.

Add a chaining operator so that `(Foo bar: x) frob` does not
need parentheses, but can be written something like
`Foo bar: x -- frob`.

Remove colon-prefix from block arguments.

Replace `|x y z|`-style bindings with `let x := 1`.

Return value of last expression from methods and blocks both.

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
