# Foolang Design

## Design Priorities

1. **Safety**: No memory errors. No race conditions, kinda. No ambient
   authority. No undefined behaviour. Fault-tolerant applications.

2. **Ergonomics**: Code should be a pleasure to read and write.

3. **Performance**: Code with type annotations should run on par with -O0
   compiled "equivalent" C or C++. As long as the compiler backend doesn't
   exist this is by necessity based on handwaving: _"that should be easy
   enough to handle!"_

4. **Uniformity**: Built-in code should not be privileged over user code.
   This is sometimes called growability: most of the language should
   be implemented in libraries, not the core.

If one of these is violated, that violation should be driven by a higher
priority concern.

!> **Race conditions**: that's not actually true in general &mdash; Foolang
only gives you freedom from race conditions relating to object state. You can
still create filesystem races and have actors that are badly behaved unless they
receive messages in specific order, etc.

## Design Notes

Reorganization of design notes into more coherent and future proof form
in in progress, this is the current status:

- 001 - [Design Notes: Why and How](design/001-design-notes-why-and-how.md)
- 002 - [No Class Inheritance](design/002-no-class-inheritance.md)
- 003 - [Sorting Floats](design/003-sorting-floats.md)
- 004 - [Reified Types Without Reflection](design/004-reified-types-without-reflection.md)
- 005 - [Program Definition and Refinement](design/005-program-definition-and-refinement.md)
- 006 - [Integer Types](design/006-integer-types.md)
- 007 - [Definitions and Expressions](design/007-definitions-and-expressions.md)
- 008 - [Indexed Classes](design/008-indexed-classes.md)
- 009 - [Self Hosting](design/009-self-hosting.md)
- 010 - [Docstrings and Comments](design/010-docstrings-and-comments.md)
- WIP - [Tower of Babel](design/wip-tower-of-babel.md)

### Old Design Notes

Most of the notes haven't been migrated yet:

- [Arrays](Arrays.md)
- [Booleans](Booleans.md)
- [Comprehensions](Comprehensions.md)
- [Iterators](iterators.md)
- [Enums](Enums.md)
- [Extension Methods](Extension_Methods.md)
- [Finalization](Finalization.md)
- [Functions](Functions.md)
- [IDE](IDE.md)
- [IRs](IR.md)
- [Interfaces](Interfaces.md)
- [Miscellany](Miscellany.md)
- [Modules](Modules.md)
- [Supervisors](Supervisors.md)
- [System](system.md)
- [Testing](Testing.md)
- [Tokenization](Tokenization.md)

## Design Compromises

### No Optional Arguments

**Problem**: occasionally questionable ergnomics, needing to create multiple
methods instead of just one.

**Excuse**: not critical right now, and Smalltalk world doesn't seem to really
miss them.

One possibility would be to use commas for this:

```
method readline, onEof: defaultAtEof, blocking: True
       ...
```

allowing:

```
system output readline, onEof: { return False }
```

### No Variable Argument Methods or Blocks

**Problem**: occasionally questionable ergnomics.

**Excuse**: not critical right now, and neither Smalltalk nor Rust worlds seem
to really miss them.

### Non-Local Returns Can Be Unsound

**Problem**:

``` foolang
class Foo {}
  direct method bad
     { return 42 }
  direct method bang
     self bad value
end
```


ie. same as eg. in Common Lisp, one can construct blocks that try to return from
frames that have already returned.

**Excuse**: having `return` inside a block do a non-local return from the
lexically enclosing method allows implementing unwinding control structures and
exception handling in user code. (Esp. when combined with dynamic bindings.)

**Possible palliative**: `<sound-returns>` pragma could ensure that all
returns are statically known to be safe, and signal a compile-time
error if not.

**Possible alternatives**:
- Implement exceptions and control flows using magic primitives
- Error values instead of exceptions
