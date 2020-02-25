# Design Notes: Why and How

**Status**: ADOPTED

**Identifier**: 001-design-notes-why-and-how.md

**References**: none

**Prior Art**:
- X3J13 issues, aka Common Lisp standardization committee issues, see:
  - http://www.lispworks.com/documentation/lw50/CLHS/Issues/I_Alpha.htm
  - http://nhplace.com/kent/Papers/cl-untold-story.html
- Architecture Design Records, aka ADRs, see:
  - https://resources.sei.cmu.edu/asset_files/Presentation/2017_017_001_497746.pdf
  - https://adr.github.io/
  - https://github.com/joelparkerhenderson/architecture_decision_record

**History**:
- 2020-02-25: initial version by nikodemus

## Problem Description

Despite Foolang currently being a one person project I have sometimes trouble
keeping track of decisions I've made months previously and their already-once
considered implications.

Currently some design questions have related notes in the source, some under
docs/, some in github issues, and some completely outside the project
structure.

This problem will only get worse if there are ever more people working on
Foolang&mdash;likelyhood is that they will be similarly distracted and
sporadically available.

### Drivers

- Low overhead, suitable for a one person project, but can obviously work
  with more people as well.
- No external tools required.
- Easy to evolve to higher or lower complexity as required.
- Easy to produce a catalog of open design problems and current decisions.

## Proposal

Keep design notes in a format reminiscent of X3J13 and ADRs (see prior
art.)

Format notes in Markdown, including extensions currently supported under
`docs/`, such as Mermaid.

[`wip-design-note-template.md`](design-notes/wip-design-note-template.md)
is template to be used when adding new design notes, with the fields
and sections described below already in place.

### Fields

Each proposal should have the following fields:

- **Title**: unique, readable, descriptive, and pithy. Humor is fine, it
  makes things more memorable.

- **Status**: One of: _WIP_, _ADOPTED_, _DROPPED_, or _RETIRED_.

- **Identifier**: for _WIP_ notes `wip-` followed by a unique lowercase
  hyphenation of the title. For notes in later statutes the `wip-` is replaced
  by a unique numeric prefix, using three digits.

- **References**: project internal references: other design notes, pull
  requests, etc.

- **Prior Art**: project external prior art: papers, language manuals, blog
  posts, etc.

- **History**: initial version date, status changes, substantive rewrites
  of _WIP_ notes, any amendments to notes in later statuses.

### Sections

Each proposal should have the following sections:

- **Problem Description**: what is the problem that should be addressed.

  Should preferably be explicit about **Drivers**: the accepted
  constraints and forces driving the proposal.

- **Proposal**: proposed solution if any, including a summary containing
  rationale and impact notes on:

  - Safety
  - Ergonomics
  - Performance
  - Uniformity
  - Implementation
  - Users

  These subsections should not be speculative, but focus on direct and knowable
  impacts. (If second or third order effects are knowable, they can and should
  be included, but that's rare in the real world.) Example: this proposal
  _might_ improve ergnomics over a long time by fostering consistent design, but
  that is a speculative and indirect impact, in no way guaranteed&mdash;and as
  such not discussed under _Ergnomics_.

- **Alternatives**: different alternatives solutions. These may be equally
  fully developed as the main proposal, or shorter. WIP proposals and
  dropped proposals might not even have a main proposal.

- **Implementation Notes** (optional): may be added during the transition from
  _WIP_ to _ADOPTED_, particularly any shortcomings or differences, should be
  amended as implementation details change.

- **Discussion** (optional): may be added during any transition, should contain
  the reason for _DROPPED_ transition at least.

### Storage

_WIP_ and _ADOPTED_ notes are stored in `.md` files under `docs/design/`
with name matching each note's identifier. This makes _WIP_ and _ADOPTED_ notes
visible in the same space, but easy to distinguish.

_DROPPED_ and _RETIRED_ notes are moved to `docs/design/dropped/` and
`docs/design/retired/` respectively.

### Statuses

Statuses evolve as follows:

``` mermaid
graph TD
   wip-->adopted
   wip-->dropped
   adopted-->retired
```

Meanings of statuses:

- **WIP**: These notes may be edited freely, but they should still contain
  the require fields and a problem description.

- **ADOPTED**: These notes describe the current design, and hopefully
  the implementation. The proposal section should not be substantively edited,
  but **Implementation notes** may evolve. If proposal (design) changes
  substantively open a new design note that supersedes the old one instead
  of amending the old one.

- **DROPPED**: A note that was thrown out without being adopted. These should
  always contain an explanation as to why&mdash;even if it is just "this is not
  the right time". Dropped design notes shall remain dropped, but almost
  identical ones can be recreated if the situation has changed.

- **RETIRED**: The note was adopted at some point, but is no longer relevant,
  typically because another note has superseded it. The references field of a
  retired note should include a pointer to the superseding note, and vice versa.

### Summary

The worst that can happen is that I look silly. If the overhead is unreasonable
I can just throw out the process.

#### Safety

No impact.

#### Ergonomics

No impact.

#### Performance

No impact.

#### Uniformity

No impact.

#### Implementation

Increased discipline in recording design issues increases overhead, but
the overhead does not seem overly large.

#### Users

No impact.

## Alternatives

- **Ad-Hoc Design Notes**: This is the "keep doing what I've been doing"
  alternative. I'm not happy with current results, so more discipline is
  indicated.

- **Use ADR Model**: ADRs are a close match to the problem, and you could call
  the proposed design note model an ADR version, but I specifically want a WIP
  status that describes and explores a problem without yet offering a specific
  proposed solution, which means "design record" is a bit of a misnomer.

- **Use Github Issues**: I don't like the evolution of github issues for this
  purpose. The states don't match the requirements, and a lot of the design
  notes are expected to be most valuable after the implementation work has been
  done, to document the reasons things were done a certain way.

## Implementation Notes

None.

## Discussion

None.
