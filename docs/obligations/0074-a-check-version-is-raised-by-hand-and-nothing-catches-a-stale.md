---
id: OBL-repo-0074
title: "A check version is raised by hand, and nothing catches a stale one"
status: current
status_since: 2026-08-12
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
    - SPEC-HW-check-layer
---

# A check version is raised by hand, and nothing catches a stale one

## Context

The [key](../spec/12-check-layer.md#determinism-concretely) carries the version of the check that reached the verdict, which is what invalidates the entries an earlier edition wrote. Nothing derives that number. Two editions of one rule read the same documents and the same lock, and they differ only in code that no digest covers.

## Obligation

So an author who changes what a rule decides and leaves the version alone reads the verdicts of the rule before the change. A digest over the compiled rule is the instrument that fits, and no part of this specification asks for one.

## Discharge

This is the same class of gap as [the fixture question for a correctness root that is a type](0079-a-correctness-root-that-is-a-type-has-no-failing-fixture.md), which is a correctness root whose defect no fixture can reach.
