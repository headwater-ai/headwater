---
id: HW-DR-0108
status: current
status_since: 2026-10-04
summary: "The site sidebar lists the documents of a shelf in path order, which is numeric order on a numbered shelf. The shelf index keeps the derived reading order."
last_verified: 2026-10-04
title: "108 — The site sidebar lists a shelf in path order, and the shelf index keeps the reading order"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-sonnet-5.5
  activity: draft
  evidence_basis: evidenced
relations:
  constrains:
    - HW-DR-0036
  governs:
    - engine/crates/generate/src/site_nav.rs
---

# 108 — The site sidebar lists a shelf in path order, and the shelf index keeps the reading order

## Context

[HW-DR-0036](0036-q36-which-of-mkdocs-docusaurus-or-astro-this-corpus-emits-navigation-for-and-why.md) has the `site_nav` emitter list each shelf in the order `by_precedence` derives. That is the order a shelf index prints, so the two never disagreed.

The owner reviewed the published site on 2026-10-04 and found that the decisions sidebar read 101, then 86, then 4. Precedence follows relations and the nucleus, and it gives a reader no rule for finding a decision they already know by number.

A sidebar and a shelf index do different jobs. A shelf index is the page a reader opens to learn what to read first. A sidebar is the list a reader uses to find a document by name or number.

## Decision

The `site_nav` emitter lists the documents of a shelf in path order. A shelf of numbered documents carries a zero-padded prefix, so path order is numeric order. A shelf with no prefix reads alphabetically by file name.

The shelf index keeps the order `by_precedence` derives. `route` keeps it too.

The generated index of a shelf stays the first entry of its group, as before.

## Consequences

- The decisions sidebar reads 1, 2, 3 and so on up to 108.
- The specification series sidebar reads by part number, with the glossary last. The shelf index still puts the documents with the highest precedence first.
- The previous and next links at the foot of a page follow the sidebar, because MkDocs builds them from the nav.
- A corpus that wants the reading order in its sidebar needs a declared option. This decision adds none, because no adopter has asked for one.
- The test in `engine/crates/generate/tests/fixtures.rs` now holds the order against the path of each document. It held the old order against `by_precedence`.
