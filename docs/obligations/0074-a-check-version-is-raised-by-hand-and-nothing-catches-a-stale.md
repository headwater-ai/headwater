---
id: HW-OBL-0074
title: "A check version is raised by hand, and nothing catches a stale one"
status: current
status_since: 2026-08-12
waiting_on: build
last_verified: 2026-08-13
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
