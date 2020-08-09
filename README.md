# Foolang

[![Build Status](https://dev.azure.com/nikodemus0619/foolang/_apis/build/status/nikodemus.foolang?branchName=master)](https://dev.azure.com/nikodemus0619/foolang/_build/latest?definitionId=1&branchName=master) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

The main [Foolang website](https://foolang.org) is https://foolang.org, containing
syntax, design notes, aspirations, etc.

"Hello world" looks like this:

``` foolang
class Main {}
    direct method run: command in: system
        system output println: "Hello world"!
end
```

0.1.0 release pronounced the bootstrap interpreter good enough. Current work is
focused on a self-hosted interpreter and C transpiler&mdash;the Rust
implementation likely to go the way of the dodo by 0.2.0.

See [CONTRIBUTING.md](CONTRIBUTING.md) for information on how to contribute.
You'll be the first, so it'll be interesting to see how that works out!

Repository organization:

```
docs/      Markdown files for the https://foolang.org website
elisp/     Emacs mode for Foolang
foo/       Foolang code, including prelude, self hosting, tests, and examples
host/      Scaffolding for transpiled-to-C code
src/       Rust code for the bootstrap interpreter
tests/     Rust code for integration tests
