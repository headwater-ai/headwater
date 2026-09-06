---
id: HW-OBL-0143
status: current
status_since: 2026-09-06
last_verified: 2026-08-26
title: "A specification sentence closes a set with a count the source exceeds, and 115 of 127 candidates are unchecked"
summary: "Three sentences that closed a set with a wrong count were found by accident, and 115 more candidates were never checked."
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: draft
  accepted_by: j.baxter
  evidence_basis: evidenced
waiting_on: adopter
---

# A specification sentence closes a set with a count the source exceeds, and 115 of 127 candidates are unchecked

## Context

Pull request #262, commit 7772252, fixed three sentences of one defect class. A specification sentence closes a set with a count, with "both", or with "either", and the source it describes has one more member. The three sentences were in `docs/spec/12-check-layer.md`, `docs/spec/01-conceptual-model.md`, and `docs/spec/02-taxonomy-model.md`. The find came from writing an interface_contract, because a contract forces an exhaustive enumeration where a specification wrote a summary. A grep of `docs/spec/` for a sentence closing a set with a number word, "both", or "either" returns 1109 candidates. Restricting to a definite determiner or a pronoun subject gives 497. Restricting to a noun a source can enumerate — exits, verbs, crates, rules, kinds, facets, relations, shelves, formats, streams, severities, scopes, emitters, arms — gives 127.

## Obligation

Twelve of the 127 candidates are checked, and nine passed while three failed. The three failures are the sentences #262 already corrected. A further 115 candidates remain unchecked against the source each one describes. No rule reads a count in prose against a source, and none could without knowing which source a sentence describes. The gap is not a cost the check layer has priced. It is a class the check layer's own subject matter excludes.

## Discharge

Discharge requires every one of the 127 candidates checked against its source, or the population re-derived with a stated filter and checked again. A sentence found wrong needs correction, or its count replaced by a reference to the verb or file that produces it. Spec 6 states that a count copied into prose is a claim no run re-derives. This waits on the sweep running to completion. No rule traces a sentence to the source it describes, and none has been designed.
