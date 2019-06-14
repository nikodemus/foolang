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

---

Stay tuned!
