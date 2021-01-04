# Docstrings and Comments

**Status**: WIP (not implemented)

**Identifier**: 010-docstrings-and-comments

**References**: none

**History**:
- 2020-08-06: initial version by Nikodemus
- 2021-01-05: updated to current format

## Problem Description

1. Use of strings vs comments as docstrings: Smalltalk uses comments as
   docstrings, Common Lisp uses strings. What are the tradeoffs?

2. How do arbirary comments at toplevel, between methods, etc, relate to
   documentation and ST-style browser-experience (even if based on editor
   views)?

Consider poppping up a view to edit eg. all method implementations of
`AstVisitor#visitConstant:`, or all classes implementing `Application`. Are
there comments outside the methods and classes being shown that should also be
shown?

## Proposal

Comments and docstrings provide two parallel prose views into code: docstrings
are the public facing prose, and comments are the internal prose.

Docstrings are strings, and can appear in following locations:

- Start of module: module-level documentation.
- Start of define: global documentation.
- Start of class (after slots): class documentation
- Start of a method: method documentation.

Comments can appear anywhere, but have conventional meanings which the
parser is aware of:

- Toplevel line comments represent section headers. Block comments taking only a
  single line, fences included, are considered line comments.
- Section headers using longer comment fences are subsections.
- Toplevel block comments are associated with their respective sections.
- Toplevel block comments before first section are module level comments.
- Successive comments of same type are concatenated.
- Comments contained inside definitions are associated with them.

Editors should support code folding following the section headers, and when
displaying a composite view should provide easy access to higher level comments
for each file.

### Summary

The docstring outline seems like a reasonable starting point.

The comment outline is not restrictive, which matches commonplace expectations,
but provides a clear idea of which comments are are associated with which
definitions.

#### Safety

No impact.

#### Ergonomics

Hopefully positive impact despite imposing meaning on comments.

#### Performance

No impact.

#### Uniformity

No impact.

#### Implementation

Some impact: parser is slightly complicated by additional comment semantics.

#### Users

No users, no impact.

## Alternatives

- ST-style solution where comments are docstrings. This seems to mix the two
  views, and creates additional burden to be careful about if the comment
  is intended to be user visible.
- CL-style solution where docstrings are strings, and comments don't have
  meaningful semantics. Allowing comments but not providing semantics
  hurts editor support.

## Implementation Notes

None.

## Discussion

None.
