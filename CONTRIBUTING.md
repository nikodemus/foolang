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

- Library code written in Foolang is very welcome even if not planned. For now
  just stick it under `foo/lib/` or `foo/lang/` as appropriate. Go right ahead.
- Examples and toys written in Foolang are very welcome! Stick them under
  `foo/examples/` for now.
- Core language contributions are extremely welcome, but far more risky in terms
  of "does it fit in the current plans". Opening an issue first is a good idea.
- Work on the Rust code is welcome, but you should be aware that it is destined
  to go away by 0.2.0 when self hosting is done. (Rust will remain the preferred
  language for writing foreign plugins -- once we get to supporting them.)

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
    possible: preference is to have no "fix", "oops", "whitespace", "more fixes"
    commits. (Squash your entire PR to one commit explaining the whole if that's
    what seems best.)
