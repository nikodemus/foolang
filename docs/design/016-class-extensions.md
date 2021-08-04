# 016 - Class Extensions

**Status**: WIP (not implemented)

**Identifier**: 016-class-extensions

**References**:
- none

**History**:
- 2021-08-04: initial version

## Problem Description

In the "dynamic development version" of Foolang (currently) the interpreter, the
system needs to be able to extend existing classes.

The semantics I'm aiming for right now are those provided by the bootstrap
interpreter, as the mininum viable design:

- Original class definition to be able to be extended with interfaces and new
  methods via extensions.
  
- Class redefinition is not allowed.

- Methods can be added.

- Interfaces can be added.

- Not in scope:
  - Replacing already defined methods: such definitions are rejected.
  - Removing interfaces.
  - Removing methods.
  - Issues of authority.

- At the same time, the system should not make it explicitly hard for
  later development to:
  - Allow changing class layout.
  - Redefine existing methods.
  - Remove existing methods.
  - Require appropriate authority through system object and mirrors.

## Decision Drivers

- Efficiency: this should not have an notable impact on method dispatch
  performance, not bloat size required by classes.
  
- Ease of implementation. This is a silly roadblock that I want to
  clear without compromising on future design, but I don't need to
  implement all the future needs now either.

## Proposal

### Review of Current Implementation

Current class layout looks like this:

```C
struct FooClass {
  struct FooHeader header;
  struct FooBytes* name;
  struct FooClass* metaclass;
  struct FooClassList* inherited;
  struct FooLayout* layout;
  FooMarkFunction mark;
  size_t size;
  struct FooMethod methods[];
};
```

and methods:

```C
struct FooMethod {
  struct FooClass* home;
  struct FooSelector* selector;
  size_t argCount;
  size_t frameSize;
  // Native method functions directly implement the method
  // Object method functions send #invoke:inContext: to the object
  FooMethodFunction function;
  struct Foo object;
};
```

Interfaces can already (in principle) be added and removed by replacing the
`inherited`-member, but number of methods is fixed when the class object is
emitted.

There is information about which methods originates from the class, and
which from inherited interfaces via the `home`.

There is no information which method originates from an extension.

There are planned refactorings to method calling convention which would
likely remove the `argCount` and `frameSize` from the method struct.

Currently `FooMethod` is slightly uncomfortably between a method object,
and a method table entry: the `home` entry is only used for printing
backtraces and debug messages.

### Plan

1. Complete the already planned changes first:
   - https://github.com/nikodemus/foolang/issues/815
   - https://github.com/nikodemus/foolang/issues/622

2. Clean up `FooMethod` into `FooMethodTableEntry`:

```C
struct FooMethodTableEntry {
    struct FooSelector* selector;
    FooMethodFunction function;
    struct Foo methodObject;
};
```

Where the `methodObject` responds to messages: `#arity`, `#home`,
`#isExtension`, and `#invoke:on:`.

The `function` is the fast way to invoke the methodObject in question, in case
of compiled methods the function directly implementing the method, in case of
non-compiled methods it will delegate to the object's `#invoke:on:`.

3. Replace `FooClass.methods` by a pointer to a `FooMethodTable`.

```C
struct FooMethodTable {
    size_t size;
    FooMethodTableEntry data[];
}
```

which can be grown when necessary.

!> In compiled code the table is can still be allocated as part of the class
object, and compiled could first do their lookups in the tail-table, which would
be empty for interpreter classes before following the pointer. This seems like
an unnecessary complication right now, but worth keeping in mind for the future.

Given the above, all of `Class#__addInterface:,` `#__addDirectMethod:`, and
`#__addInstanceMethod:` should be easy to implement, and sufficient for current
needs: they still must check the `FooClass.header` for the class not being
static, but given that changes are allowed.

Once actors enter the picture, classes will have a reference count for number of
actors they're visible to. Reference count 1 actors can be dealt with directly,
others need to wait for other executors to stop. (No locking on the class or
method table.)

### Summary

Despite the immutability of compiled classes, for dynamic development environment
classes do need to be modified.

This proposal arranges the related structures in such a way that those modifications
become feasible, without introducing other big design changes, and allowing
the facilities to be later encapsulated by mirrors, and extended for finer
control.

#### Safety

Impact on safety: none. Since this proposal doesn't allow removal of
methods or interfaces there should not be any type issues. Otherwise
the same overriding and extension facilities were already available
at compile-time.

#### Ergonomics

No impact when compared to bootstrap interpreter, clear positive impact when
compared to self-hosted implementation as REPL becomes more useful.

#### Performance

Assumed none, though additional indirection to method table may turn
out to be an issue. If that's the case we should see it almost immediately,
and there's already a plan for mitigation.

#### Uniformity

No impact. While this doesn't directly allow extending built-in classes
as they are statically allocated, this does allow interpreter to provide
wrappers that _can_ be extended.

#### Implementation

Minor impact. Required changes are relatively minor and do no increase
complexity in a substantial way - and additional layer of indirection,
but no new interactions.

#### Users

No users, no impact.

## Alternatives

- Represent classes in the interpreter with non-class objects.
  ```class InterpreterClass { name layout interfaces methods } ... end

     define InterpreterInteger
          InterpreterClass name: "Integer" ...!
  ```
  - Pro: interpreter becomes more indenpendent of the host.
  - Con: ...not sure there is one, at least not at the moment.

## Implementation Notes

Implementation not started, but going to do the planned refactorings
even if the alternative proposal ends up being promoted to the proposal
properl

## Discussion

None.

