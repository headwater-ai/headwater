---
id: HW-VER-0001
status: current
status_since: 2026-09-19
summary: "A case table in engine/crates/check/src/sections.rs is what the workspace suite runs, on every change, to prove HW-AC-0002."
last_verified: 2026-09-19
title: "The section-heading rule has a case table, and the workspace suite runs it on every change"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: draft
  evidence_basis: evidenced
relations:
  proves:
    - HW-AC-0002
---

# The section-heading rule has a case table, and the workspace suite runs it on every change

## Approach

A case table in `engine/crates/check/src/sections.rs` holds one row per shape the rule has to tell apart, and the workspace suite runs every row on each change to the engine.

    cargo test -p headwater-check --lib sections --manifest-path engine/Cargo.toml --locked

That command is what makes the method `test` rather than `inspection`: the case table decides the verdict, and nobody reads a tree to settle it. `headwater check --root . --strict` runs the same rule over this corpus and reports what it finds there, which is the second half of [HW-AC-0002](../acceptance-criteria/0002-every-required-section-of-every-governed-document-has-a-heading.md)'s own Method section.

## What this does not cover

This document carries no anchor into the case table it names. [#936](https://github.com/headwater-ai/headwater/issues/936) adds the anchor kind and the resolver that would bind `engine/crates/check/src/sections.rs` as something the graph can read, and until it lands a rename of that file breaks no edge here, because no edge reaches it yet. The identity this document carries is its own, which is the property [HW-DR-0073](../decisions/0073-a-verification-is-a-kind-and-its-identity-is-minted-rather-than-found-in-the-code-that-cites-it.md) asked for; the case table itself is still found by reading the source rather than by following a link.

This document also carries no observation. [#937](https://github.com/headwater-ai/headwater/issues/937) adds the snapshot that would name the commit this suite last ran clean against. Today a reader has only the workspace suite's own exit status, taken fresh on every run.
