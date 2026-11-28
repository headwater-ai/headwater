---
id: HW-OBL-0272
status: current
status_since: 2026-11-25
summary: "`headwater check` counts the governed scope with nothing ignored, and `taxonomy validate` leaves out what git ignores. So a pattern that matches only ignored files passes one verb and fails the other."
last_verified: 2026-11-25
title: "A governed scope pattern whose only matches git ignores passes check and fails taxonomy validate"
waiting_on: ruling
provenance:
  warrant: asserted
relations:
  traces_to:
    - engine/crates/graph/src/scope.rs
    - engine/crates/check/src/scope_unmatched.rs
---

# A governed scope pattern whose only matches git ignores passes check and fails taxonomy validate

## Context

[#2099](https://github.com/headwater-ai/headwater/issues/2099) added the rule `relation.scope.unmatched`. It reports each governed scope pattern that matches no entry of the tree, as an error. The rule and `headwater taxonomy validate` call one function, `Scope::coverage` in `engine/crates/graph/src/scope.rs`. The two verbs pass that function different sets of ignored paths.

- `taxonomy validate` passes what git ignores. So an ignored build output or cache on one host is not an entry, which an owner ruled on 2026-09-25 for #951.
- `headwater check` passes nothing. [Spec 12](../spec/check-layer.md) keeps every version control command off the check loop, so the check cannot ask git.

## Obligation

Two answers differ, and no finding names the difference.

- A pattern whose only matches are files that git ignores passes `check --strict` and fails `taxonomy validate`.
- A top directory that git ignores counts as a tree in the check and not in `validate`. So at a root whose only top directory is ignored, the check counts the scope and `validate` skips it with the no-tree notice.

The adopter case of #2099 is not affected. A pattern that opens with a wildcard is refused before any walk and before any ignore rule. The `headwater check` contract states this limit.

## Discharge

This record discharges when the two verbs give one answer for an ignored path. It also discharges when a ruling accepts the difference and the contract cites the ruling. One fix is a check input that names the ignored paths with no git command on the loop. A verb outside the loop can write that list. That way needs a ruling against the line spec 12 draws.
