---
id: OBL-repo-0087
title: "`fixable` has two readings inside one engine"
status: current
status_since: 2026-08-13
last_verified: 2026-08-13
summary: "One reading says what is true of the defect and the other says what the engine will do about it."
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
    - SPEC-HW-taxonomy-model
---

# `fixable` has two readings inside one engine

## Context

[Spec 12](../spec/12-check-layer.md#fixability) makes the flag a property of the finding: whether the fix is mechanical and total. [Spec 2](../spec/02-taxonomy-model.md#the-language-regime-carries-the-terms-that-the-corpus-retired) reads it that way for a retired term, where the presence of a replacement decides it, and `language.retired_term.used` implements exactly that.

`language.controlled.not_met` sets it false for a contraction, whose expansion is as mechanical as any substitution. The stated ground is that no patch rides along, and that a flag with nothing behind it claims a capability the engine lacks.

## Obligation

Both readings are defensible and they are not the same reading. One says what is true of the defect and the other says what the engine will do about it.

## Discharge

`check --fix` is where the two meet, and until it exists a reader of the report cannot tell which sense a `fixable` flag carries.
