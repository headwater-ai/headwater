---
id: OBL-repo-0077
title: "A fragment on a path needs a grain that no scope supplies"
status: current
status_since: 2026-08-13
last_verified: 2026-08-13
summary: "`link.fragment.unresolved` reads a bare fragment and never one on a path, and 1595 of this corpus's fragments carry a path."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - DR-repo-0004
---

# A fragment on a path needs a grain that no scope supplies

## Context

A fragment with no path names a heading of the document that wrote it, and `link.fragment.unresolved` reads it at document grain. A fragment on a path names a heading of another document, and no [scope](../spec/12-check-layer.md#scope--the-declaration-everything-else-rests-on) carries one. `Document` carries one document. `Edge` carries a relation instance, and [Q4](../spec/09-decisions.md#q4--relation-storage) rules that a prose link is not a relation. `Neighbourhood` carries the documents one relation away, and it carries the identity of each rather than the body.

## Obligation

The grain that reaches this is one document and the documents that its prose links reach. This corpus writes 1595 fragments on a path against 208 that carry no path, so the unreached half is the larger one. Both numbers move with every prose edit, and the pair recorded here before was 1519 against 277.

## Discharge

The gap has been priced as well as counted. A script that resolves every fragment on a path by hand found two that reach no heading, and both had stood on `main` unreported. They are repaired, and the instrument that found them is twenty lines, so the cost of this entry is the scope and not the rule.
