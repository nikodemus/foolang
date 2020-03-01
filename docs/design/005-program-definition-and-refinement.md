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
- 2020-03-01: initial version by Nikodemus

## Problem Description

What is a Foolang program, how is one defined?

## Proposal

### Program Environment

A program is defined by an environment object containing a graph of named &
immutable objects. Objects in the program environment do not have access to the
environment object itself.

Executing a program is equivalent to sending the message `run:in:` to the `Main`
object in the program environment.

### Construction of Program Environment

Source code can be both purely declarative as well as constructive.

Declarative source code uses `class` and `interface` syntaxes to define global
objects by describing their behaviour and layout.

Constructive source code uses `define` syntax to define global objects by
describing their construction using arbitrary Foolang code, but without
access to a System object or ability to cause non-local side-effects.

The program environment is constructed by the compiler by _loading_ the
associated source code, and constructing the objects it describes:

1. The compiler initializes an empty program environment.

2. The compiler processes all associated source code, creating the declaratively
   defined objects in the environment under their own names, adding the
   constructively defined names to the environment as placeholders containing
   their associted constructive definitions without executing them yet.

3. The compiler executes all constructive definitions in the environment,
   replacing the plaholder defitions with the constructed objects. Accessing a
   constructive placeholder definition during execution of another triggers the
   recursive execution of the first.

   During execution of recursive constructive definitions the compiler must
   monitor this process to detect cyclic dependencies, and raise an exception on
   such.

   After a constructive definition has been executed the compiler must verify
   that the resulting object is immutable. (This may be relaxed in development
   mode, see below.)

?> Because `define` operations have no access to operating system and cannot
mutate global objects, they cannot cause arbitrary effects: they can construct
arbitrarily complex objects, including cyclic ones, but they cannot do anything
that would change the meaning of other parts of the program any more than a
declarative class definition could.

### Separate Compilation

The compiler can identify sections of the program text defined purely
declaratively and use precompiled versions of them.

Construtive definitions can also be precompiled, but only if all their
dependencies are part of the same precompilation unit.

The definition given here does not provide a facility for delivering a
precompiled part separately, but neither does it contradict the ability to do
so.

### Refinement in Development Mode

In development mode the immutability constraint of a program environment is
relaxed, allowing its refinement via patching:
- Mutation of global objects to add, remove, and redefine methods, and change
  layouts of classes.
- Addition of new gobal objects.
- Removal of global objects.

Patching a program is equivalent to sending a message to the program environment
itself, and as such not possible to regular programs, though in development mode
is would be possible to define a global giving access to the environment to rest
of the program.

!> **DETAILS TO BE DETERMINED** (1) How are changes to class layouts handled? (2)
When is patching an environment possible, can the program be running? (3) How
does opening a previously immutable class for mutation work? (4) How does class
patching relate to class extensions?

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

#### Uniformity

None.

#### Implementation

Minor: the proposed algorithm is simple.

#### Users

No users, no impact. If there were users, this model would be hard
to restrict later.

## Alternatives

- Disallowing constructive definitions.
- Restricting constructive definitions further, such as:

  - only allow references to builtins
  - only allow references to globals in the same module
  - only allow references to declarative globals

  Relaxing restrictions later would be a backwards compatible step, unlike
  adding them to the proposed model.

  Implementation burden would be increased however, since enforing the
  restrictions would increase complexity.

## Implementation Notes

None.

## Discussion

None.
