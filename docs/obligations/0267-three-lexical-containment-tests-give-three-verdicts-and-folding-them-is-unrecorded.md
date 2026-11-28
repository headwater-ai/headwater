---
id: HW-OBL-0267
status: current
status_since: 2026-11-24
summary: "leaves_the_repository in census, stays_inside_the_tree in census and escapes in resolve each decide whether a path leaves its root, by different rules."
last_verified: 2026-11-24
title: "Three lexical containment tests give three verdicts, and folding them is unrecorded"
waiting_on: build
provenance:
  warrant: asserted
---

# Three lexical containment tests give three verdicts, and folding them is unrecorded

## Context

[The register of open obligations](../spec/open-obligations.md) holds a finding that has no reader outside this repository. Run `20261122-0114` wrote intake line 29, from the builder of #1967. The product owner ruled it RECORD into the group that its end-of-run report calls S4. This record found the three functions again on `0cdd96298`.

## Obligation

#1967 added `escapes` in `engine/crates/resolve/src/containment.rs`. Two more lexical tests of containment stand beside it, and each gives its own verdict.

- `leaves_the_repository` in `engine/crates/census/src/outside.rs` reports a path that holds any `..`.
- `stays_inside_the_tree` in `engine/crates/census/src/derived.rs` keeps a running depth. It fails when the depth goes below 0, and also when the path ends at the root, because it asks for a depth above 0.

Nothing records whether the two should call `escapes`, or why their verdicts differ.

## Discharge

This record discharges when the two census functions call `headwater_resolve::containment::escapes`, or when a change records why each keeps its own rule.
