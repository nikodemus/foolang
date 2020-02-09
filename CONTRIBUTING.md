# Contributing to Foolang

Wonderful that you're interested!

## Existing Plans

Take a look at the [open
issues](https://github.com/nikodemus/foolang/issues?q=is%3Aopen+is%3Aissue+no%3Aassignee),
and if you see something in that interests you, feel free to dive right in.

But a caveat!

So far this has been a one-person project. So most of the issues are rather
terse, just enough for one, but not necessarily descriptive for others.

So unless the issue is reasonably explicit about what it means, it's probably a
best to ask "What's the idea here?" first.

Perusing the [design documents](https://foolang.org/#/design) on the website
might also be useful.

## Your Ideas Outside Existing Plans

Pull requests are definitely welcome, but you might want to open an issue first:
it sucks to work on something and then later learn that it's not wanted.

Roughly:

- Cleanups to the Rust code are welcome even if not planned, particularly
  if they address some a `FIXME` in the source. Go right ahead.
- Library code written in Foolang is very welcome even if not planned. For now
  just stick it under foo/. Go right ahead.
- Library code written in Rust is less welcome right now: the Kiss3D stuff is
  already causing regrets. An import/plugin architecture for classes written
  in Rust needs to be implemented first. Open an issue first.
- Core language contributions are extremely welcome, but far more risky in
  terms of "does it fit in the current plans". Open an issue first.

## Coding Conventions

- Newline is LF, not CRLF. (If a file needs to use CRLF for a specific
  reason, it should mention that.)
- Spaces, not tabs.
- Foolang code should mostly indent like foolang.el wants: if that's obviously
  wrong indent by hand and report an issue against foolang.el.
- Foolang code mostly prefers "square" blocks:
  ``` foolang
  foo bar: { stuff todo.
             moreStuff todo }
  ```
- Rust code should indent the way `cargo fmt` wants.
- Commit messages
  - descriptive first line, preferably identifying the component being worked on
  - body explains the why or the how as appropriate
  - before submitting a pull request clear out egregious noise commits if
    possible: no "fix", "oops", "whitespace", "more fixes" commits please.
