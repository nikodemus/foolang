# Foolang TODO

## Release 0.1.0

High level goal: bootstrap interperter for core language complete.

- [x] one pass over contents of docs/ to collect TODOs here
- [x] allow comments inside classes (between methods)
- [x] rewrite main README.md pointing to docs/
- [x] cleanup stale branchs
- [x] LICENCE file
- [x] rename --ide to --webrepl
- [x] change all Expr internals to structs
- [x] allow arbitrary binary messages
- [x] allow prefix message definitions
- [x] emacs syntax highlighting & indentation
- [x] Record class
- [x] Record syntax
- [x] Dictionary class
- [ ] Dictionary syntax
- [ ] class library documentation
  - [ ] attach docstrings (comments) to methods
  - [ ] attach docstrings between methods to class
  - [ ] placeholder way of gaining access to the docstrings
  - [ ] documentation generation in Foolang
    - [ ] Filesystem class
    - [ ] File class
    - [ ] FileOutput class
- [ ] CONTRIBUTING.md
- [ ] decide between this text file stuff and github tools
- [ ] inheritable Interfaces with default methods and require methods
- [ ] dynamically bound variables
- [ ] distinction between mutable and immutable local bindings
- [ ] distinction between mutable and immutable slots
- [ ] fix links in bibliography
- [ ] organize foo/ -> foo/lang, foo/tests, foo/examples
- [ ] kill bar/
- [ ] refactor the Assert class so it can be imported
- [ ] reserved word for raising an error (possibly placeholder until dx vars)
- [ ] decide on notational conventions for instance and class methods
      ie. is it Class##classMethod and Class#instanceMethod or something else?
- [ ] document `is`

## Later

__Swamp of undifferentiated action points.__

- [ ] Reflection via mirrors
- [ ] narrower layout for examples on foolang.org to make it nicer on mobile
- [ ] cleanup: rename string\_as\_str to as\_str.
- [ ] change Expr into a trait, turn Expr objects into Box<dyn Expr>
- [ ] expose Expr as Object to Foolang code
- [ ] electric return in Foolang mode: go to correct indentation immediately
- [ ] foolang-indent-method
- [ ] foolang-indent-class
- [ ] foolang-indent-buffer
- [ ] syntax change: method bodies end with a dot
      (allows most reserved words to be used as message names)
- [ ] Character class
- [ ] non-pretty but parse-consistent printing for Exprs
- [ ] pretty-printing for Exprs (wadler-style?)
- [ ] --allow-private-sends or --devmode flag required for `foo _method`
- [ ] change string interpolation to use StringStream instead of append
- [ ] error source locations, incl. file
- [ ] reloading of changed imports in --devmode or --allow-reload \
      Question: what to do about changes to layouts?
      CL-style update protocol, warning, what?
- [ ] backtraces
- [ ] implicit `_` argument in blocks
- [ ] Block#apply
- [ ] fork and improve https://github.com/TheGreenToaster/docsify-glossary to
      make it require something like %term% markup, and use it to for docs/
- [ ] prism support for foolang (for highlighting in docs/)
- [ ] vscode support for foolang
- [ ] string generators for property testing
- [ ] array generators for property testing
- [ ] test case saving for property testing
- [ ] test case minimization for property testing
- [ ] `foo[x]` accessor syntax as sugar for `foo at: x`
- [ ] array slicing methods
- [ ] replace Array with Vector class, Array becomes an interface
- [ ] Matrix class
- [ ] NdArray class
