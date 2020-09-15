# Foolang

![CI](https://github.com/nikodemus/foolang/workflows/CI/badge.svg) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**This is a toy language implemented by a single person&mdash;use for quiet enjoyment only.**

Foolang is a Smalltalk-inspired language that, like all new languages, has what
you might generously call _somewhat optimistic aspirations_:

- **_Elegance and power of Smalltalk and Self:_** Smalltalk-like syntax, deep object
  orientation, and late binding.

- **_Performance of C++:_** Support for early binding when you need it so that the
  compiler can go to town. Low-level operations which allow eliding overflow
  checking in tight loops, etc.

- **_Fault tolerance of Erlang:_** Agent-model, isolated heaps, and supervisors.
  No undefined behaviour.

**_"Are we there yet?"_**

Nope.

Syntax is still settling down, early binding support isn't quite there, there is
no compiler yet, and many things which should be first class objects aren't yet,
etc.

**_"When we going to get there?"_**

Someday!

## Hello World

``` foolang
class Main {}
    direct method run: command in: system
        system output println: "Hello world"!
end
```

## More Information

The main [Foolang website](https://foolang.org) is https://foolang.org, containing
syntax, design notes, aspirations, etc.

See [CONTRIBUTING.md](CONTRIBUTING.md) for information on how to contribute.
You'll be the first. :)

Repository organization:

```
docs/      Markdown files for the https://foolang.org website
elisp/     Emacs mode for Foolang
foo/       Foolang code, including prelude, self hosting, tests, and examples
host/      Scaffolding for transpiled-to-C code
src/       Rust code for the bootstrap interpreter
tests/     Rust code for integration tests
