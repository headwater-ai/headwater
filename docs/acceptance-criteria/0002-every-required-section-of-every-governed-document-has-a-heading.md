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
  proven_by:
    - HW-VER-0001
---

# Every required section of every governed document has a heading

## Fit criterion

`headwater check` over this corpus reports no instance of `section.required.missing`.

The rule is an error rather than an advisory, so a strict run refuses a commit that would add one. `.githooks/pre-commit` runs the strict form, and the CI job runs it again on every pull request. So the criterion is re-established on every change rather than at one commit, which is the difference between this criterion and [HW-AC-0001](0001-no-source-file-of-the-engine-names-a-network-api-and-no-locked-dependency-provides-one.md).

One thing the criterion does not state. The rule reads a heading, and it reads no word under one. A document with every required heading and nothing under any of them passes. That is the honest reading of what `sections.require` declares, and the remedy is a different rule rather than a stricter reading of this one.

## Method

Test. `engine/crates/check/src/sections.rs` carries the rule and no case table of its own. The golden fixture in `engine/crates/check/tests/fixtures.rs` proves it, over the tree at `engine/crates/check/fixtures/check/`. `check/spec/08-contract-met.md` writes every heading its kind requires, and `check/spec/09-contract-missing.md` omits one. Both feed the recorded report at `engine/crates/check/fixtures/check.report`, and the test compares that report byte for byte.

Two commands produce what a reader inspects.

    cargo test -p headwater-check --test fixtures --manifest-path engine/Cargo.toml --locked
    headwater check --root . --strict

The first holds the rule to the two fixture documents and the report they produce. That is what makes the method `test` rather than `inspection`: the fixture documents decide the verdict, not a person who reads a tree. The second command runs the rule over this corpus and reports what it found.

The two are one method and not two. A rule that could report nothing would leave the second command silent over any corpus at all. A green run and a clean corpus then look the same. The fixture report is what separates them.
