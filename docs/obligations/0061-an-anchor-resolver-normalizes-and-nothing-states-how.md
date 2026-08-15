---
id: HW-OBL-0061
title: "An anchor resolver normalizes, and nothing states how"
status: current
status_since: 2026-08-12
last_verified: 2026-08-13
summary: "Spec 2 requires that anchor strings normalize before comparison, and it states no rule that does it."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-taxonomy-model
    - HW-SPEC-check-layer
---

# An anchor resolver normalizes, and nothing states how

## Context

[Spec 2](../spec/02-taxonomy-model.md#behavior-at-the-limits) requires that anchor strings normalize before comparison, so that two spellings of one target are one node. It states no rule that does it. [Spec 12](../spec/12-check-layer.md#the-correctness-roots) then names the resolver a correctness root, because a resolver that mis-normalizes makes `governs` edges miss with no error anywhere.

## Obligation

The engine had to decide the rules, and a decision of that class belongs to the meta-schema.

## Discharge

`engine/crates/graph/src/anchors.rs` states them. A backslash reads as a separator. A `.` segment goes, and a `..` segment cancels the segment before it. An absolute path, and a path that climbs above the repository, are refused rather than clamped. Every rule is lexical, because a resolver that asks the filesystem follows a symlink out of the repository.

This is a meta-schema decision of the same class as [the shelf order](0060-the-most-specific-shelf-wins-names-no-order.md).
