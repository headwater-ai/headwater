---
id: HW-OBL-0088
title: "Correcting an identifier is mechanical and it is not local"
status: current
status_since: 2026-08-13
waiting_on: ruling
last_verified: 2026-08-13
summary: "Spec 12 lists an identifier correction among four mechanical fixes, and no such repair is confined to one document."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0004
---

# Correcting an identifier is mechanical and it is not local

## Context

[Spec 12](../spec/12-check-layer.md#fixability) lists four mechanical fixes, and one of them is to correct the format of an identifier. Two of the three ways an identifier fails its pattern do have one derivable outcome. A wrong namespace has the declared namespace, and a short sequence has the declared width.

Neither fix is confined to the document that holds the finding. An identifier is the one value that other documents keep copies of, because [Q4](../spec/09-decisions.md#q4--relation-storage) rules that a relation target is an identifier. This corpus names `HW-SPEC-taxonomy-model` from three other documents, so a patch that rewrote it would leave three dangling edges behind. The zero-padding case is worse, because allocation is reconcile-first and the padded value may already belong to another document.

## Obligation

Either spec 12 qualifies its own example, or `check --fix` gains a class of fix that spans documents and reports every file it would touch.

## Discharge

So `identifier.pattern.not_met` offers no fix, and the rule that spec 12 states is per finding while this repair is corpus-wide. [The record on `fixable`](0087-fixable-has-two-readings-inside-one-engine.md) is the same flag read two ways, and this is a third question about it.
