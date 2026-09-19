---
id: HW-VER-0001
status: current
status_since: 2026-09-19
summary: "A golden fixture report in engine/crates/check/tests/fixtures.rs is what proves HW-AC-0002, not a per-shape case table."
last_verified: 2026-09-19
title: "The section-heading rule has no case table, and a golden fixture report is what proves it runs"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: draft+revise
  evidence_basis: evidenced
relations:
  proves:
    - HW-AC-0002
---

# The section-heading rule has no case table, and a golden fixture report is what proves it runs

## Approach

`engine/crates/check/src/sections.rs` carries the rule and no test of its own: it has no `#[cfg(test)]` module and no per-shape table. What proves it is the golden fixture in `engine/crates/check/tests/fixtures.rs`, over the tree at `engine/crates/check/fixtures/check/`. `check/spec/08-contract-met.md` writes every heading its kind's contract requires, and `check/spec/09-contract-missing.md` omits one; both are read into the recorded report at `engine/crates/check/fixtures/check.report`, and the test compares that report byte for byte.

    cargo test -p headwater-check --test fixtures --manifest-path engine/Cargo.toml --locked

That command is what makes the method `test` rather than `inspection`: the two fixture documents decide the verdict, and a byte comparison against a recorded report is what would fail if the rule stopped telling them apart. `headwater check --root . --strict` runs the same rule over this corpus and reports what it finds there, which is the second half of [HW-AC-0002](../acceptance-criteria/0002-every-required-section-of-every-governed-document-has-a-heading.md)'s own Method section.

## What this does not cover

**No per-shape case table exists for this rule**, and this document does not claim one. An earlier draft of this document and of HW-AC-0002's own Method section named `cargo test -p headwater-check --lib sections`, a command that runs zero tests against an empty module; that claim was wrong, inherited from a Method section that was already wrong, and this revision corrects both.

This document carries no anchor into the source it names. [#936](https://github.com/headwater-ai/headwater/issues/936) adds the anchor kind and the resolver that would bind `engine/crates/check/src/sections.rs` as something the graph can read, and until it lands a rename of that file breaks no edge here, because no edge reaches it yet. The identity this document carries is its own, which is the property [HW-DR-0073](../decisions/0073-a-verification-is-a-kind-and-its-identity-is-minted-rather-than-found-in-the-code-that-cites-it.md) asked for; the rule's own behavior is still found by reading the source and the fixture tree rather than by following a link.

This document also carries no observation. [#937](https://github.com/headwater-ai/headwater/issues/937) adds the snapshot that would name the commit this suite last ran clean against. Today a reader has only the workspace suite's own exit status, taken fresh on every run.
