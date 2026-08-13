---
id: OBL-repo-0087
title: "`fixable` has two readings inside one engine"
status: current
status_since: 2026-08-13
last_verified: 2026-08-14
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

**This record is discharged, and the engine carries the second reading.** `headwater check --fix` ships. A finding holds the patch it offers instead of a flag beside one, and `fixable` in a report is read off that value. So the word answers one question: whether this run writes the correction. Two fields could disagree and one cannot.

**The first reading did not go, and the severity carries it.** A defect whose remedy is mechanical and total is an error, whether or not the engine writes it. That is the bar `CLAUDE.md` states for this repository and the bar [spec 12](../spec/12-check-layer.md#fixability) states for a taxonomy. `shelf.placement_is_primary` is the case that separates the two: it reports an error, its remedy is to delete one key, and no patch rides with it.

**The bar reads per finding rather than per rule, and the two rules of this record show it.** `language.controlled.not_met` reports a contraction with a patch where the expansion is one word. It reports one without a patch where a reader of the sentence chooses between `it is` and `it has`. `language.retired_term.used` keeps the reading that [spec 2](../spec/02-taxonomy-model.md#the-language-regime-carries-the-terms-that-the-corpus-retired) gives it, and the declared replacement becomes the patch as well as the remediation prose.

**Two questions about the same flag stay open, and each has a record.** [OBL-repo-0088](0088-correcting-an-identifier-is-mechanical-and-it-is-not-local.md) asks whether a patch may span documents. [OBL-repo-0103](0103-the-front-matter-half-of-a-patch-has-no-writer.md) asks what a read-back over front matter compares.
