---
id: HW-OBL-0074
title: "A check version is raised by hand, and nothing catches a stale one"
status: current
status_since: 2026-08-12
waiting_on: build
last_verified: 2026-09-26
summary: "Two editions of one rule differ only in code that no digest covers, so a stale version reads the verdicts of the rule before the change."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-check-layer
---

# A check version is raised by hand, and nothing catches a stale one

## Context

The [key](../spec/12-check-layer.md#determinism-concretely) carries the version of the check that reached the verdict, which is what invalidates the entries an earlier edition wrote. Nothing derives that number. Two editions of one rule read the same documents and the same lock, and they differ only in code that no digest covers.

## Obligation

So an author who changes what a rule decides and leaves the version alone reads the verdicts of the rule before the change. A digest over the compiled rule is the instrument that fits, and no part of this specification asks for one.

## Discharge

[The fixture question for a correctness root that is a type](0079-a-correctness-root-that-is-a-type-has-no-failing-fixture.md) is the same class of gap. Both name a correctness root whose defect no fixture can reach.

**Another version was raised by hand, and again nothing checked it.** `link.fragment.unresolved` went from 2 to 3 when it widened to read a fragment on a resolving path ([HW-OBL-0077](0077-a-fragment-on-a-path-needs-a-grain-that-no-scope-supplies.md)). The raise is necessary. A warm cache holds edition-2 verdicts about same-document fragments alone, and it would serve them over a corpus whose cross-document fragments were never examined. An author who forgot it would ship a rule that reads more and reports the same, and every gate would stay green.

**A verdict ledger now catches a version left alone, where a recorded corpus shows the change ([#1105](https://github.com/headwater-ai/headwater/issues/1105)).** The test `engine/crates/check/tests/editions.rs` keeps one row for each rule over each recorded corpus that it reaches. The row holds the version, a fingerprint of the corpus and a digest of every verdict that the cache would store. A digest that moves while the version and the corpus stay fails the test, and the test names the rule. `HEADWATER_BLESS` does not record that row again, so the bless that `DEVELOPING.md` prescribes cannot hide the defect. This does not discharge the record. A change that no recorded corpus exercises still passes, and so does a rule change in the same commit as an edit to its corpus. Seven rules have no instance on any recorded corpus. The digest over the compiled rule that this record asks for is still unbuilt.
