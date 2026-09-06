---
id: HW-OBL-0152
status: current
status_since: 2026-09-06
title: "Ledger::render() in adoption.rs has the same zero-count defect suppression.rs had before PR #450, still latent"
summary: "adoption.rs::Ledger::render() drops its own header and count line whenever a corpus declares no adoption debt, the same shape PR #450 fixed in suppression.rs, and nothing reads the dropped text yet."
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: draft
  accepted_by: j.baxter
  evidence_basis: evidenced
last_verified: 2026-08-26
waiting_on: adopter
---

# Ledger::render() in adoption.rs has the same zero-count defect suppression.rs had before PR #450, still latent

## Context

[PR #450](https://github.com/headwater-ai/headwater/pull/450) fixed `engine/crates/check/src/suppression.rs::Inventory::render()`. That function returned `String::new()` whenever a corpus declared zero suppression directives. It dropped the `suppressions` header and its `N findings hidden by M directives` count line from every text report. `tools/refresh-figures.sh` greped that line, and it broke outright once this corpus's suppression count reached zero. The pull request's adjudication stage read `engine/crates/check/src/adoption.rs::Ledger::render()` while it diagnosed the shape of the fix. It noted, without fixing, that this second function carries the identical pattern.

This record reads the two functions side by side, rather than taking that note alone. `Ledger::is_empty()` (`adoption.rs:409-411`) returns true when `tasks`, `refused` and `unread` are all empty. `Ledger::render()` (`adoption.rs:463-468`) returns `String::new()` when `is_empty()` holds, before it ever writes the `"adoption\n"` header line. This is the same early-return-before-the-header shape `Inventory::render()` had. It feeds the same call site: `Run::render` at `lib.rs:932` writes `self.adoption.render()` right after `self.coverage.render()`, unconditionally, in every text-format run. The comment on that line quotes spec 7's "beside coverage" placement.

`Ledger` is empty in two distinct states, and only one of them is rare. The first is a lock whose declared `adoption` block itself lists no tasks, no refused entries and no unread keys. Closing every pair of a task, or letting a task expire, does not produce this state on its own. Neither act removes the task's own entry from the lock, so `tasks` stays non-empty until a person deletes that entry by hand. This corpus's own lock still carries `AD-1`, with its one pair closed, so the corpus does not hit this state today. The second, far more common state is a lock that declares no `adoption` block at all. `lib.rs:727-729` shows that `apply()` runs unconditionally. When `declared.adoption` is `None`, the code hands `apply()` an empty `adoption::Declared::default()`, with all three fields empty, the same as a declared and empty block. This is every adopter before their first `headwater infer --write`, and this corpus itself, right up until it wrote `AD-1`.

## Obligation

Nothing in this repository reads `Ledger::render()`'s text output the way `tools/refresh-figures.sh` reads `Inventory::render()`'s. A search of `tools/`, `.githooks/`, `.github/` and `.claude/` finds no pattern that greps the `adoption` block. None greps its header line, or its `N pairs open, M closed, holding K findings, in J tasks` count line. `refresh-figures.sh` reads the `census` and `suppressions` blocks by name, and never `adoption`. So this defect is not live-broken today, for this corpus or for the one script here that parses `headwater check`'s text output. Reading the actual code, rather than the note alone, confirms it is the same defect class. It sits one call away from live. The day a script or a CI step reads the `adoption` line the way `refresh-figures.sh` reads the `suppressions` line, it inherits the identical silent failure. It will find the line missing in the common case, no debt declared, rather than the rare one.

The principle the suppression fix cited applies here unchanged. "Zero suppressions" was itself news worth stating, not an absence to render as nothing. A reader cannot tell "zero" from "this run did not get that far" when the section is simply missing. The same argument holds for "zero adoption debt". That is the state every adopter is in before their first `headwater infer --write`, and it was this corpus's own state before `AD-1` was written. Spec 4's "no silent passes" principle names exactly this kind of absence as one a report should state, not omit.

## Discharge

This closes the way the suppression fix closed. Delete the `is_empty()` early return in `Ledger::render()`, so the header line and the zero-count summary line print unconditionally. Leave the per-task, per-refused and per-unread loops below it exactly as conditional as they already are, since each already prints nothing over an empty collection. `Ledger::is_empty()` itself stays. [`crate::register`] uses it to attribute an escape to an obligation, which is a different question from whether the report renders a line.

A fixture proves the fix. `engine/crates/check/fixtures/corpus.checks` records a `headwater check` text report, the same fixture the suppression fix re-blessed. Point that fixture at a corpus with no `adoption` block declared, the state this corpus itself carried before `AD-1` was written. Before the fix, the recorded report has no `adoption` block at all. After the fix, it gains the two-line block `adoption\n  0 pairs open, 0 closed, holding 0 findings, in 0 tasks`. That move is the same shape of evidence the suppression fix produced, and it discharges this record.

This waits on an adopter. The fix is mechanical once decided, but nothing here reads the dropped text yet. No live consumer forces the decision the way `refresh-figures.sh` forced the suppression one. This is filed as a corpus record rather than a GitHub issue for that reason. The reader it serves is this engine's own internal correctness, not an outside adopter today.
