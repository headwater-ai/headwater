---
id: HW-OBL-0201
status: current
status_since: 2026-09-26
summary: "1233 tests exist under engine/crates and nobody has measured whether they would catch a defect, with cargo-llvm-cov and cargo-mutants as the named, unrun instruments."
last_verified: 2026-09-22
title: "No coverage or mutation measurement has ever been taken over the engine's test suite"
waiting_on: measurement
---

# No coverage or mutation measurement has ever been taken over the engine's test suite

## Context

[#586](https://github.com/headwater-ai/headwater/issues/586) asked, on 2026-09-06, what this repository uses beyond `clippy` to hold code quality. It found 1233 `#[test]` attributes under `engine/crates`. It found no coverage number, no mutation score and no run of either tool anywhere in the repository or its history. [HW-DR-0023](../decisions/0023-the-engine-lint-floor.md) set the lint floor the same way, by measuring first. This obligation therefore carries the same order for the layer beneath the lints: the test suite itself has never had the question put to it. The product-owner pass of 2026-09-22 closed #586 as recorded rather than planned. It closed under the ruling that a finding with no reader outside this repository belongs here and not on the tracker.

## Obligation

This corpus owes two measurements over `engine/`, taken and written down rather than asserted. A coverage run with `cargo-llvm-cov` reports the line and branch fraction the test suite reaches, per crate. A mutation run with `cargo-mutants` reports, for at least `crates/check`, `crates/graph`, `crates/lock` and `crates/resolve`, the count of mutants caught, missed, unviable and timed out. It also reports the wall-clock cost of the run. Every surviving mutant in those four crates is triaged into one of three verdicts. It is a real hole in the suite, code whose behavior nothing depends on, or a mutant that is not a behavior change. A plain statement that the measurement was taken, the result was acceptable and nothing follows is a legitimate discharge. It does not have to produce a follow-up. What it may not do is stay unmeasured while every other layer of this repository (`.githooks/fixtures.sh`, `.claude/hooks/fixtures.sh`, `.claude/skills/fixtures.sh`, `.claude/tutorial/fixtures.sh`, `tools/id-store-fixtures.sh`) already answers this same question about itself.

## Discharge

The instruments exist and neither has been run: `cargo-llvm-cov` for coverage, `cargo-mutants` for mutation testing. Nothing here proposes a CI gate, a coverage threshold or a required mutation score. A ruling would have to set one, and this obligation buys the number that ruling would rest on. It waits on a session that runs both tools over `engine/` and records the numbers and the triage somewhere durable. That session closes with one of the three outcomes #586 named. The first is a follow-up issue for holes worth closing. The second is a decision record if a tool or a bar is adopted. The third is a plain statement that the measurement was taken and the result was acceptable.
