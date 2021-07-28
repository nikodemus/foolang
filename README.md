# Foolang

![CI](https://github.com/nikodemus/foolang/workflows/CI/badge.svg) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**_This is a toy language implemented by a single person&mdash;use for quiet enjoyment only._**

See https://foolang.org for syntax, design notes, etc. This README is a
smaller version of the main page there.

See [CONTRIBUTING.md](CONTRIBUTING.md) for information on how to contribute.
You'll be the first. :)

## About

Foolang is a Smalltalk-inspired language that, like all new languages, has
somewhat _optimistic aspirations_:

- **_Elegance and power of Smalltalk and Self:_** Smalltalk/Objective-C -like
  syntax, deep object orientation, late binding, interactive development.

- **_Performance of C++:_** AOT compilation to native code, support for early
  binding so that the compiler can do its thing, low-level datatypes and
  operations when you need them for performance.

- **_Fault tolerance of Erlang:_** Actor-model, isolated heaps, and supervisors.
  No undefined behaviour. No deadlocks, or memory errors or races.

- **_Multiplatform Citizen of the Web:_** WASM is a supported target in addition
  to Windows, MacOS, Linux, and BSDs.

**_"Are we there yet?"_**

:rofl:

Syntax is still going to change, WASM isn't supported, BSDs might work but
aren't tested, early binding support isn't quite there, compiler is a
work-in-progress trivial transpiler, actors and continuations haven't even been
started, there is no interactive development environment to speak of, etc.

## Hello World

``` foolang
class Main {}
    direct method run: command in: system
        system output println: "Hello world"!
end
```

## Repository Organization

In rough order of interest:

```
foo/       Foolang code, including prelude, self hosting, tests, and examples
src/       Rust code for the bootstrap interpreter
docs/      Markdown files for the https://foolang.org website
elisp/     Emacs mode for Foolang
c/         Scaffolding for transpiled-to-C code
tests/     Rust code for integration tests
ext/       External C code included in the runtime, like dtoa.c.
```
