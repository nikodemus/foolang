# Definitions and Expressions

**Status**: ADOPTED (implemented'ish)

**Identifier**: 007-definitions-and-expressions

**References**:
- 005 - [Program Definition and Refinement](005-program-definition-and-refinement.md)

**Prior Art**: none

**History**:
- 2020-03-15: initial version by Nikodemus
- 2020-03-19: implementation notes by Nikodemus
- 2020-06-06: impact clarified by Nikodemus

## Problem Description

Foolang toplevel syntax is intended to be declarative: even constructive
definitions are declarative if you squint hard enough: this name is associated
with this value.

Having definitions in the same syntactic category as expressions causes issues:

``` foolang
class X {}
end
```

If this is an expression it needs to be followed by a dot&mdash;otherwise a
following `class` token would be a message to the object, or the syntax would
need exceptional rules along the lines of "class expressions are terminated by
'end' instead of a dot", etc.

## Proposal

Separate the syntax into definitions and expressions. Expressions are always
contained in definitions, except in sessions/workspaces, where they can be
mixed.

(The following grammar elides several details.)

```
<module> := <definition>+

<definition> := import <importSpecification>
              | export <exportSpecification>
              | class <name> <classSpecification> end
              | interface <name> <interfaceSpecification> end
              | extend <name> <extensionSpecification> end
              | define <name> <expr> end

<classSpecification> := { <slotSpecification>* } <methodSpecification>*

<methodSpecification> := method <selector> <expr>+

<expr> := <simpleExpr> . <expr>?
        | let <name> = <simpleExpr> . <expr>
        | return <simpleExpr> . <deadExpr>?
        | panic <simpleExpr> . <deadExpr>?

Allowed, but cause for a warning:

<deadExpr> := <expr>

<simpleExpr> := <literal>
              | <variable>
              | <simpleExpr> <messages>
              | <simpleExpr> :: <type>
              | ( <expr> )
              | { ... }
              | [ ... ]

<variable> := <alphabetic-or-underscore><alphanumeric-or-underscore>*
           !! <reservedWord>

<reservedWord> := let
                | return
                | panic
                | required
                | method
                | import
                | export
                | define
                | class
                | end

<messages> := <chain>
           | ; <cascade>

<cascade> := <chain>
           | <chain> ; <cascade>
```

### Summary

Treat toplevel code as non-expressions. This means workspaces are special, but
clarifies syntax otherwise.

#### Safety

None.

#### Ergonomics

"Absolutely everything is an expression" elegance is lost, but separate
syntactic categories otherwise clarify parsing.

#### Performance

None.

#### Uniformity

None.

#### Implementation

Minimal.

#### Users

No users, no issues.

## Alternatives

- `<anything> end` are all declarations, not separated by dots.
- `class X ... end` is an expression, which as a side effect creates the
  class in the current scope. (Alternatives: creates class in toplevel scope,
  allowed only in toplevel scope.)
- `class X ... end` is a declaration, which instructs the compiler to create
  the class in toplevel scope.
- `class X ... end` is equivalent to `define X ... end` where the body containts
  a constructuve expression for creating the class object.

## Implementation Notes

At the moment parser is aware of contexts in which definitions are not allowed,
but not of contexts where expressions are not allowed -- this is handled in
parser call sites.

