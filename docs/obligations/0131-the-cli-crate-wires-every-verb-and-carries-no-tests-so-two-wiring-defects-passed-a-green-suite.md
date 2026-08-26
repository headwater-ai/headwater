---
id: HW-OBL-0131
status: draft
status_since: 2026-08-26
last_verified: 2026-08-26
title: "The CLI crate wires every verb and carries no tests, so two wiring defects passed a green suite"
summary: "Two wiring defects in the CLI crate's verb dispatch were found by hand, and reverting either fix still leaves the test suite green."
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: draft
  accepted_by: j.baxter
  evidence_basis: evidenced
waiting_on: adopter
---

# The CLI crate wires every verb and carries no tests, so two wiring defects passed a green suite

## Context

`engine/crates/cli/src/main.rs` runs to 3,668 lines and carries no tests of its own. It is the crate that wires every verb to the library behind it. That makes it the one place where a correct library and a correct emitter can still produce a wrong answer. Two wiring defects surfaced there in three iterations, found by hand rather than by the suite. Both were in verb wiring, not in a library.

## Obligation

The first defect took `Plan::over(..).selected` in `Loaded::runs()` and dropped the `refusal` beside it. A corpus whose probe plan refused still published a rate through a green gate, over a silently reduced denominator. The second tested `plan.selected.is_empty()` in the `probe grade` arm, where the rule is `Refusal::stops_a_grade()`. A refusal that left a partial selection still graded against probes a planner had already given up on. Both defects are fixed, but reverting either one alone still leaves `cargo test --release` reporting 71 suites, 627 passed, 0 failed. A check that fires on no input is indistinguishable from one that does not work, and by that standard both fixes were unverified.

A later pass added a test target, `engine/crates/cli/tests/wiring.rs`, driving the built binary over a fixture root. It gave three of roughly thirty verb decisions a case shown to fail against its pre-fix form. The three are `check --change`, `probe grade`, and the `Loaded::runs()` call site. The corpus still owes a case for every other verb whose behavior turns on reading one library value rather than another.

## Discharge

What closes this is the enumeration that has never been done. Every decision point in the CLI crate needs a name, wherever a correct library value sits beside a wrong one the wiring could read instead. Each point then needs a case shown to fail against its pre-fix form. No new gate discharges any part of it, since this is coverage rather than a rule. The work is recorded rather than planned, so it waits on an adopter to make the remaining decisions worth enumerating.
