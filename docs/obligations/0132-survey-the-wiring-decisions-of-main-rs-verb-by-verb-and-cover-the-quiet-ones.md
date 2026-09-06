---
id: HW-OBL-0132
status: current
status_since: 2026-09-06
last_verified: 2026-08-26
title: "Survey the wiring decisions of main.rs, verb by verb, and cover the quiet ones"
summary: "Two more wiring decisions in main.rs went uncovered by hand alone, and the other twenty-seven verbs have never been surveyed for the same failure."
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: draft
  accepted_by: j.baxter
  evidence_basis: evidenced
waiting_on: adopter
---

# Survey the wiring decisions of main.rs, verb by verb, and cover the quiet ones

## Context

`engine/crates/cli/tests/wiring.rs` holds one case each for `check --change`, `probe grade`, and the `Loaded::runs()` call site that feeds `generate`, from [#179](https://github.com/headwater-ai/headwater/issues/179). Each case was watched failing against the pre-fix form of a defect a person found only by reading the code after something else broke. None of the three defects moved an exit code, so nothing before that reading would have reported them.

A wiring decision is a place where `main.rs` chooses which library value to trust, rather than a place where a library computes one. The three fixes already made took three shapes. One read a rule at the wrong grain. One parsed an input and did not inject it where it needed to reach. One composed a value and handed it on with part of it dropped. `main.rs` runs to about 3,700 lines across thirty verbs, and nothing has enumerated where else it makes a decision of this shape.

Two further sites are already named. `probe stale` is the third caller of `Plan::gradable()`, and it is the one caller not already covered by type or by test. The `taxonomy vendor` doctrine report, added by [#81](https://github.com/headwater-ai/headwater/issues/81) after this issue was filed, makes two wiring decisions of its own. It re-derives `flattened` from `record.package` where `package::vendor` derives it from `identity`. That is safe only because `identity` refuses a record whose header disagrees with the manifest. It also reads the installed manifest with `manifest_at(&installed).ok()`, so a read error prints nothing at all. Both decisions were confirmed by hand and by nothing else, during construction and again during verification.

## Obligation

The corpus owes a survey of `main.rs`, verb by verb, that records every wiring decision as covered, structurally impossible, or uncovered. A verb that decides nothing is recorded as making none.

Every uncovered decision with a quiet failure, one that moves no exit code, owes a case in `crates/cli/tests/wiring.rs`. Each case is watched failing against the form its defect names.

Where a decision can be made unwritable instead of merely tested, it owes that treatment first. `Plan::gradable()` and `Unbound::bind` are the two idioms already in the tree. A structural guarantee replaces a case that has to remember to run.

`probe stale` owes coverage now, as the third caller of `Plan::gradable()`. Its symptom would be a read set composed over part of a selection a planner abandoned before finishing.

The `taxonomy vendor` doctrine report owes two cases of its own. One covers the `flattened` path, re-derived from `record.package` instead of from `identity`. The other covers the silent read of the installed manifest through `manifest_at(&installed).ok()`, which reports nothing on failure.

The remaining twenty-seven verbs owe the survey itself, and the survey owes a stated cost rather than a discovered one. The existing target runs in 0.05 seconds under `cargo test`, and a spawned case costs about 15 milliseconds. A survey that produces thirty such cases should report that cost rather than discover it.

## Discharge

This is discharged when the survey exists as a table over all thirty verbs. Each row names a verb as covered, structurally impossible, or uncovered, and no verb is left unlisted.

It is discharged further when every uncovered decision whose failure is quiet has a case in `crates/cli/tests/wiring.rs`, each one watched failing first.

Where a decision can be made structurally impossible instead of tested, that treatment discharges it in place of a case. `Plan::gradable()` and `Unbound::bind` set that pattern already.

The two named sites, `probe stale` and the `taxonomy vendor` doctrine report, discharge on the same terms once their cases exist and pass.

This waits on an adopter to run the survey, choose the treatment for each finding, and add the resulting cases without adding a new gate.
