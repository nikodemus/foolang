# Foolang

[![Build Status](https://dev.azure.com/nikodemus0619/foolang/_apis/build/status/nikodemus.foolang?branchName=master)](https://dev.azure.com/nikodemus0619/foolang/_build/latest?definitionId=1&branchName=master) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

The main [Foolang website](https://foolang.org) is https://foolang.org, containing
syntax, design notes, aspirations, etc.

``` foolang
class Main {}
    class method run: command in: system
        system output println: "Hello world!"
end
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for information on how to contribute.
You'll be the first, so it'll be interesting to see how that works out!

Repository organization:

```
docs/      Markdown files for the https://foolang.org website
elisp/     Emacs mode for Foolang
foo/       Foolang code, including prelude, tests, and examples
src/       Rust code for the bootstrap interpreter
tests/     Rust code for integration tests
webrepl/   HTML and Javascript for the webrepl, includes CodeMirror which is
             why github thinks this is a javascript repo...
```
