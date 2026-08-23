---
id: HW-OBL-0047
title: "The scope of a sub-document identifier"
status: current
status_since: 2026-08-12
waiting_on: build
last_verified: 2026-08-13
summary: "An acceptance criterion receives a stable identifier, and nothing says whether it numbers within its requirement or over the corpus."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-taxonomy-model
---

# The scope of a sub-document identifier

## Context

Acceptance criteria are one of the five artifacts that receive a stable identifier, and a [contract sidecar](../spec/02-taxonomy-model.md#contract-sidecars-the-specification-as-oracle) gives them one. Nothing says what scopes that identifier. Allocation is reconcile-first over the corpus, which is the wrong denominator if a criterion numbers within its requirement.

## Obligation

Nobody has minted a criterion, because no specification of the sidecar exists. This is the other half of [identity below the grain of a document](0036-identity-below-the-grain-of-a-document.md), and the two differ in their remedy. That item converts an entry into a document, and a criterion stays inside a sidecar and needs a parent to number against.

## Discharge

The first half is settled and it settles nothing here. An `obligation_record` is a document, so the corpus is the right denominator for it, and the decision-record entry allocates one reconcile-first over the corpus. A criterion is not a document, and no conversion is available to it.
