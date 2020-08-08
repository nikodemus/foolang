# Program Definition and Refinement

**Status**: ADOPTED (not implemented)

**Identifier**: 005-program-definition-and-refinement

**References**: none

**Prior Art**:
- 1997 - [A Declarative Model for Defining Smalltalk
  Programs](https://web.archive.org/web/20200301140324/https://www.instantiations.com/vast/files/archive/Smalltalk-Solutions97/SSDCL1.HTM)
  by Allen Wirfs-Brock.
- 1997 - [Smalltalk ANSI Standard, Chapter 3.
  (draft)](https://web.archive.org/web/20200301135818/http://www.math.sfedu.ru/smalltalk/standard/chapter3.html.en)

**History**:
- 2020-03-01: Initial version by Nikodemus.
- 2020-08-08: Separate resolution and finalizations steps allow cyclic
  references. Marginally improved specification of separate compilation.

## Problem Description

What is a Foolang program, how is one defined?

## Proposal

### Program

A program is defined by a _program graph_ of immutable objects rooted in an
object called `Main`.

Executing a program is equivalent to sending the message `Main #run:in:`.

### Construction of a Program

The compiler constructs a program graph by:

1. Initializing an empty _program environment_. The program environment will map
   global names to program components.

2. Loading all associated source code into the environment as program components,
   without resolving references to globals.

3. Resolving the name `Main` in the environment, and finalizing the
   corresponding component.

   To _resolve_ a name find the corresponding program component in environment:
   recursively resolve all names in subcomponents, store the results, and return
   the component.

   To _finalize_ a program component, recursively finalize the resolved results
   saved during the resolution stage, and construct the final object.

   If resolution enters a program component already being resolved, return
   the in-progress component.

4. The resolved and finalized `Main` is the root of the program graph.

### Separate Compilation

Modules can be precompiled by pre-resolving all definitions contained in the module,
and pre-finalizing them in the partial program environment:

- Pre-resolution is like resolution, except imported names produces fixups.
- Pre-finalization is like finalization, except fixups are allowed.

To link a precompiled to module fixups are resolved and finalized as if during
normal program construction.

Precompiled modules are assumed to be non-portable between different machines,
and non-compatible between different versions of Foo.

### Refinement in Development Mode

In development mode the immutability constraint of a program graph is relaxed
and access to the program environment is provided:

- Definitions can be changed, added, and removed.

- Methods can be redefined, added and removed from existing classes and interface.

- Class layouts can be changed.

However: programs graphs still cannot modify themselves.

!> Without significant compiler and runtime cleverness development mode is
heavily pessimized. This is an acceptable initial tradeoff.

### Summary

A halfway point between purely declarative and wildly constructive approaches:
the power of constructive definitions is too great to ignore, see eg. the
definition of `Prefix` class in `lib/si.foo`: at the time of writing it is very
repetitive, but a constructive definition could build the same class from
information contained in a simple table.

#### Safety

None.

#### Ergonomics

Good: constructive definitions are convenient, declarative definitions are
clean.

#### Performance

None.

#### Uniformity

None.

#### Implementation

Minor: the proposed algorithm is simple.

#### Users

No users, no impact. If there were users, this model would be hard
to restrict later.

## Alternatives

- Precompilation of modules linking in all dependencies early. Smaller
  precompilation units seem more preferable, they can always be linked
  into larger ones as well.

- Disallowing cyclic dependencies. This is an interesting question: cycles are
  often smell, but they're also often quite convenient. A further alternative
  would be to allow _explicit_ cyclic dependencies:

  ```
  class A { b::(cyclic B) }
  end

  class B { a::(cyclic A) }
  end
  ```

  ...but that seems like pointless pedantry?

- Disallowing constructive definitions.

- Restricting constructive definitions further, such as:

  - only allow references to builtins
  - only allow references to globals in the same module
  - only allow references to declarative globals

  Relaxing restrictions later would be a backwards compatible step, unlike
  adding them to the proposed model.

  Implementation burden would be increased however, since enforing the
  restrictions would increase complexity - and generality of the
  language would be reduced.

- Specifying portability of compatibility requirements.

## Implementation Notes

None.

## Discussion

None.
