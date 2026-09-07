---
id: HW-OBL-0178
status: current
status_since: 2026-09-07
summary: "The check runner declares a report detail that prints every check instance, and only four test targets ever select it, because no flag and no library consumer can."
last_verified: 2026-09-07
title: "The every-instance report detail reaches four test files and no caller outside them"
waiting_on: ruling
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - engine/crates/check/src/lib.rs
---

# The every-instance report detail reaches four test files and no caller outside them

## Context

`headwater_check::Detail` names how much of a run a report accounts for, and `Detail::EveryInstance` is the widest of its values. It renders one line per check instance rather than one line per finding.

Every file that names the value was enumerated from the tree on 2026-09-07, and the declaration in `engine/crates/check/src/lib.rs` was excluded. Four files remain: `engine/crates/check/tests/cache.rs`, `engine/crates/check/tests/fixtures.rs`, `engine/crates/import/tests/drift.rs` and `engine/crates/scaffold/tests/pipeline.rs`. All four are test targets.

Two routes could carry the value to somebody else, and neither does. `headwater check --help` on the built binary names no detail option. `[workspace.package]` in `engine/Cargo.toml` declares `publish = false`, and `cargo metadata --locked --no-deps` reports it on all 23 members. So no outside author can link this crate and select the value in code.

The question the value answers for a reader is which documents a run decided nothing about. `headwater check --format json` answers that question at a grain a machine reads. Every entry of `coverage.skips` names the census rows the class was routed to, and the instances it routed to none. One terminal line for every instance of a corpus this size answers the same question worse.

## Obligation

A variant no caller outside a test suite can reach is a claim with one beneficiary, and that beneficiary is this repository. The corpus owes a ruling on whether the value is a fixture affordance or an unshipped feature. It owes the record of that ruling where a reader of the crate will meet it.

The corpus does not owe a command-line flag. Naming one here would state a decision that nobody has made.

Deletion is not free either, and the ruling has to price it. `engine/crates/check/tests/cache.rs` compares a warm run against a cold run at this detail, because it is the widest account the type offers. A comparison at `Detail::Findings` stops reading a cache defect that moves the outcome of an instance without moving a finding.

## Discharge

A ruling states which of the two the value is. Where it is a fixture affordance, the doc comment on the variant says so and names the one test that needs the width. This record then closes with nothing else to write.

Where it is an unshipped feature, the ruling names the reader outside this repository who selects it and the surface they select it through. A new option on `headwater check` is also an edit to the interface contract at `docs/interfaces/headwater-check.md`, which is the artifact that reader holds the binary to.
