---
id: HW-DR-0101
status: current
status_since: 2026-09-30
summary: "A scaffold edits no target of a symmetric relation. A far half in a live target reads as a live document that rests on a draft. It sets no supersedes state, which check --fix writes."
last_verified: 2026-09-30
title: "new writes no far half of a symmetric relation and no state on a supersedes target"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: evidenced
relations:
  constrains:
    - HW-DR-0086
---

# new writes no far half of a symmetric relation and no state on a supersedes target

## Context

[HW-DR-0086](0086-a-reciprocal-half-is-owed-once-its-writer-leaves-its-initial-state.md) stopped `headwater new` from writing the far half of a `reciprocal: required` relation while the new document is a draft. It left a symmetric relation unchanged, and it gave this reason: "no check reads a symmetric pair".

That reason is false. `relation.reciprocity.missing` reads only required pairs. But `lifecycle.dependency.on_initial` reads every relation between two documents, and it names `conflicts_with`, a symmetric relation, as in scope. So a far half that `new` writes into a live target is read.

[#1187](https://github.com/headwater-ai/headwater/issues/1187) reported the defect. It was measured on `origin/main` at `03021137` on 2026-09-30. An overlay added a symmetric relation `pairs_with` between decisions, with `created_by: scaffold`. Then `headwater new decision --relates pairs_with=HW-DR-0004` opened a draft and edited HW-DR-0004, a live decision. The report said the half went in "because reciprocity is required". The next `headwater check` reported `lifecycle.dependency.on_initial` on HW-DR-0004, because it now wrote `pairs_with` to a draft. That is the warning HW-DR-0086 removed for required pairs.

The same issue asked what `new` does with the state that `supersedes` declares for its target, `on_target: {set_state: superseded}`. `new` writes no state onto the target. Its report said that "no rule of this engine reads a transition". That is also false. Since [#1198](https://github.com/headwater-ai/headwater/issues/1198), `lifecycle.state.not_set_by_edge` reads an authored target that an edge retires, and its patch writes the state with its stamp.

The issue offered three outcomes for the symmetric half. The half is owed and a check reads it, or the half is deferred as a required half is, or `new` does not write it. The run ruled the third outcome, and the supersedes state is left to `check --fix` (run `20260929-2201`).

## Decision

**`headwater new` writes no far half of a symmetric relation, at any opening state.** The engine defines a symmetric relation as its own inverse. It does not define it as a pair that both documents must declare, which is what `reciprocal: required` means. So the half in the new document states the edge. The graph and `headwater explain` show the edge from both ends. The report says that the relation is symmetric and that the target is not edited.

The other two outcomes are refused for these reasons:

- A check that reads symmetric pairs is a product change wider than this record. It needs the graph to pair `(A, r, B)` with `(B, r, A)`, and it needs a new fix. It also turns every one-sided `conflicts_with` in an adopter's corpus into an error.
- A deferral as for a required half has no collector. No check reads a symmetric pair, so `check --fix` never writes the deferred half. Every base regime opens at `draft`, so a deferral means that the half is never written, and the report would say otherwise.

**`headwater new` writes no state onto the target of a relation that declares `on_target.set_state`, at any opening state.** `lifecycle.state.not_set_by_edge` reads the transition once the new document leaves its initial state. Its patch carries the state and its stamp, and `headwater check --fix` writes it. So the rule is the one writer of that state. The report names the rule and the fix.

**What reopens the symmetric ruling.** Two cases reopen it. The first is a symmetric relation whose far half a reader needs in the target's own front matter. The second is a decision to make `relation.reciprocity.missing` read symmetric pairs. Either case moves the ruling to the first outcome, with the deferral on the predicate of HW-DR-0086.

**What reopens the supersedes ruling.** A published regime that opens at a state whose role is not `initial` reopens it. With one, `new` in the base leaves a live target at its old state, and a strict check fails on that tree until `check --fix` runs.

## Consequences

**The warning goes away for a symmetric relation.** A draft that `new` opens with a symmetric edge onto a live document leaves the live document's bytes alone. So `lifecycle.dependency.on_initial` has no line from a live document to a draft to report. `engine/crates/scaffold/tests/opening_state.rs` holds this at a draft and at a live opening state.

**The base is not changed in practice.** The published base and bundles declare one symmetric relation, `conflicts_with`, and `new` refuses it because an agent creates it. Every base regime opens at `draft`, so `new` in the base always defers the far half of `supersedes` and never edits its target. So both rulings are for an overlay author. That author declares a symmetric relation for a scaffold, or a regime that opens at a live state.

**At a live opening state, a supersession leaves the target at its old state.** `new` writes `superseded_by` into the target, and the target still reads `current`. `lifecycle.state.not_set_by_edge` reports that as an error, and `headwater check --fix` writes `superseded` with its stamp.

**The assisted fraction counts the near half of a symmetric edge only.** `new` writes no far half, so the far half is in neither term. [HW-OBL-0001](../obligations/0001-the-promotion-fix-has-no-reading-of-the-assisted-fraction.md) records that no run has measured the fraction yet, and this record does not discharge it.
