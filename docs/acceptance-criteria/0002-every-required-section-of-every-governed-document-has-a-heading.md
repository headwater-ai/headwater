---
id: HW-AC-0002
status: current
status_since: 2026-09-12
summary: "The rule reports one finding for each required section a document does not carry, and it reads the heading and never the prose under it."
last_verified: 2026-09-12
title: "Every required section of every governed document has a heading"
verification_method: test
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: draft
  evidence_basis: evidenced
relations:
  verifies:
    - HW-REQ-0002
  traces_to:
    - engine/crates/check/src/sections.rs
---

# Every required section of every governed document has a heading

## Fit criterion

`headwater check` over this corpus reports no instance of `section.required.missing`.

The rule is an error rather than an advisory, so a strict run refuses a commit that would add one. `.githooks/pre-commit` runs the strict form, and the CI job runs it again on every pull request. So the criterion is re-established on every change rather than at one commit, which is the difference between this criterion and [HW-AC-0001](0001-no-source-file-of-the-engine-names-a-network-api-and-no-locked-dependency-provides-one.md).

One thing the criterion does not state. The rule reads a heading, and it reads no word under one. A document with every required heading and nothing under any of them passes. That is the honest reading of what `sections.require` declares, and the remedy is a different rule rather than a stricter reading of this one.

## Method

Test. The rule has a case table in `engine/crates/check/src/sections.rs`, and the workspace suite runs it on every change to the engine.

Two commands produce what a reader inspects.

    cargo test -p headwater-check --lib sections --manifest-path engine/Cargo.toml --locked
    headwater check --root . --strict

The first holds the rule to its own behavior over inputs the case table states. It is what makes the method `test` rather than `inspection`: nobody reads a tree and decides. The second runs the rule over this corpus and reports what it found.

The two are one method and not two. A rule that could report nothing would leave the second command silent over any corpus at all. A green run and a clean corpus then look the same. The case table is what separates them.
