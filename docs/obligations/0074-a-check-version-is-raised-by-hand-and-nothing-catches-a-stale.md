---
id: HW-OBL-0074
title: "A check version is raised by hand, and nothing catches a stale one"
status: current
status_since: 2026-08-12
waiting_on: ruling
last_verified: 2026-11-20
summary: "A verdict ledger catches a check version left alone over a recorded corpus, and two instance rules and two shapes of change stay outside it."
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-check-layer
---

# A check version is raised by hand, and nothing catches a stale one

## Context

The [key](../spec/check-layer.md#determinism-concretely) carries the version of the check that reached the verdict, and that version is what makes the entries of an earlier edition stale. Nothing derives the number. Two editions of one rule read the same documents and the same lock, and they differ only in code that no digest covers. So an author who changes what a rule decides and leaves the version alone reads the verdicts of the rule before the change.

**A verdict ledger now catches a version left alone, where a recorded corpus shows the change ([#1105](https://github.com/headwater-ai/headwater/issues/1105)).** The test `engine/crates/check/tests/editions.rs` keeps one row in `engine/crates/check/fixtures/editions.ledger` for each rule over each recorded corpus that it reaches. The row holds the version, a fingerprint of the corpus and a digest of every verdict that the cache would store. A digest that moves while the version and the corpus stay fails the test, and the test names the rule. `HEADWATER_BLESS` does not record that row again, so a bless cannot hide the defect.

A raise that the ledger sees is re-recorded by a bless. For example, `identifier.claim.stale` went from version 2 to 3 in `f910f601` ([#1502](https://github.com/headwater-ai/headwater/pull/1502), Refs [#1366](https://github.com/headwater-ai/headwater/issues/1366)), and the bless re-recorded its 7 rows at that time.

`link.fragment.unresolved` shows why a raise matters. It went from version 2 to 3 when it widened to read a fragment on a resolving path ([HW-OBL-0077](0077-a-fragment-on-a-path-needs-a-grain-that-no-scope-supplies.md)). A warm cache held edition-2 verdicts about same-document fragments alone, and it would serve them over a corpus whose cross-document fragments nobody examined. Nothing checked that raise when it landed.

## Obligation

Each rule in `RULES` either has a ledger row or is named in `UNCOVERED` in `editions.rs` with a reason. The test `every_rule_is_ledgered_or_named_as_uncovered` holds that partition in both directions. The rules in `UNCOVERED` are of two classes, and only one class is a gap.

- **Rules that write no cache entry.** Twelve rules reach a finding through a direct call in `headwater_check::run`, and not through `Cache::outcome`. Six of them are `coverage.document_unchecked`, `language.outside_root.refused`, `taxonomy.pin.diverged`, `taxonomy.lock.stale`, `projection.budget.exceeded` and `relation.scope.unmatched`. The other six are `harvest.pin.unread`, `import.pin.unread`, `obligation.disposition.not_one`, `control.mechanism.unimplemented`, `control.observation.invalid` and `adoption.task.expired`. No cache entry holds their verdict, so a stale version cannot serve an old verdict for them.
- **Instance rules that no recorded corpus reaches.** `surface.local_path.instructed` and `surface.command.undeclared` go through the cache, and no recorded corpus declares a `surface` and an adopter shelf. Their cases are unit tests only. A change to what either rule decides, with no version raise, passes every test.

Two shapes of change are outside the ledger for every rule:

- A change that no recorded corpus exercises.
- A change that shows only against a warm cache that an older binary wrote.

A third shape, a rule change in the same commit as an edit to the corpus that the ledger reads it over, is closed. The bless now replays the corpus as the ledger recorded it and refuses the row when the verdicts differ ([HW-OBL-0222](0222-ten-rules-of-the-editions-ledger-have-one-recorded-corpus-so-a-rule-change-blessed-with-an-edit-to-that-corpus-passes.md)).

A digest over the compiled rule closes both, and no part of the specification asks for one. `headwater_check::rules_digest` ([#1029](https://github.com/headwater-ai/headwater/issues/1029)) hashes the list of rule names. It moves when a rule enters or leaves the set, and not when the code of a rule changes.

[HW-OBL-0079](0079-a-correctness-root-that-is-a-type-has-no-failing-fixture.md) is the same class of gap. Both name a correctness root whose defect no fixture can reach.

## Discharge

The question for a person: **does the engine build a digest over the code of each rule, or accept the ledger and its named gaps?** One shape of the digest is a hash of the source of each rule at build time, which joins the cache key.

If the ruling is to build the digest, this record is discharged when the digest joins the key. A fixture then changes the code of one rule and leaves its version alone, and a warm cache from the older binary serves nothing.

If the ruling is to accept the ledger, this record is discharged when [the check layer](../spec/check-layer.md) states the accepted gaps. The two instance rules above each gain a recorded corpus that reaches them, or the specification names them as accepted.
