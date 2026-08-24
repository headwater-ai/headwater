---
id: HW-OBL-0108
title: "An agent writes the acceptance stamp of every document in this corpus"
status: discharged
status_since: 2026-08-24
waiting_on: ruling
last_verified: 2026-08-24
summary: "Stop rule 5 forbids an agent to write `accepted_by`, and 161 of the 161 agent-drafted documents here carry one that the drafting agent typed."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-ai-integration
    - HW-SPEC-authoring-and-lifecycle
    - HW-DR-0034
---

# An agent writes the acceptance stamp of every document in this corpus

## Context

[Stop rule 5](../spec/05-ai-integration.md#the-stop-rules) states the bar. An agent never writes `accepted_by`, and it never moves a warrant to `accepted`. It drafts, and it marks what it drafted as `asserted` where no human read the result. [Spec 11](../spec/11-adjacent-work.md#l5-onboarding-ships-the-synthesized-tier-and-marks-nothing) names the failure the rule prevents: a stamp that an agent applies to its own output, with a false name attached.

A count over the corpus root, excluding the package content that the descriptor excludes:

| reading | value |
|---|---|
| documents with front matter | 164 |
| documents with a provenance block | 163 |
| blocks that name an agent in `drafted_by` | 161 |
| of those, blocks that also carry `accepted_by` | **161** |
| of those, blocks that carry `warrant: asserted` instead | **0** |

The agent that drafted each document typed `accepted_by: j.baxter` and `warrant: accepted` into it, before any human read a word of it. That is every case where the rule could apply, and the rate is one.

This record does the same thing, in its own front matter, for the reason the next section gives.

## Obligation

The corpus owes a ruling, and the two available answers differ in what they cost.

**The rule is right and the practice is wrong.** An agent writes `warrant: asserted` and no `accepted_by`. A human moves both at review. Nothing in this engine moves a warrant, so the human moves it by hand on every document, and 161 documents need the correction.

**The practice is right and the rule is worded wrong.** A pull request is the acceptance, the block is part of what the human reviews, and a merge is the act that `accepted_by` names. Under this reading the stamp is a proposal until the merge and a fact after it.

The second reading is defensible and the bytes do not carry it. A document that a human merged and a document that an agent stamped and nobody read are the same file. So the corpus cannot answer the question that [spec 3](../spec/03-authoring-and-lifecycle.md#provenance-is-recorded-not-assumed) says provenance exists to make a query rather than an archaeology exercise.

Nothing reports any of this. [HW-OBL-0030](0030-the-provenance-block-belongs-to-the-engine-and-nothing-states.md) records that no taxonomy declares the block, so no rule reads a field of it. A stop rule with no instrument is a preference. This is the measurement of what that preference costs over 161 documents.

## Discharge

**This record is discharged.** [HW-DR-0034](../decisions/0034-q34-whether-acceptance-means-merged-to-main-and-what-an-agent-may-write-before-that.md) rules that acceptance is the merge onto `main`, and that the second reading below holds. An agent may write `accepted_by` and `warrant: accepted` on a document it drafts. This holds where the document goes into a pull request a named human reviews before the merge. The stamp answers to nothing until the merge decides whether it was true. Stop rule 5 is reworded to say so.

A ruling on which reading holds, in the decision register, because the two answers assign the act to different parties.

Under the second reading, the block owes a form that separates a proposed stamp from a merged one. A warrant of `asserted` at draft time and a promotion at merge is one shape. It needs an actor that this engine does not have. [HW-OBL-0105](0105-nothing-plays-the-hook-role-that-two-relations-name.md) already records that the `hook` actor plays no part anywhere.

Under either reading, the block owes a declaration before a rule can read it, which is `HW-OBL-0030`.

## The practice moved before the ruling arrived, and this is the count

Re-measured on 2026-08-14 over every Markdown document under `docs/`, outside `docs/taxonomies/**`. 182 carry `drafted_by`. 179 carry `accepted_by`. **Six carry `warrant: asserted` and no `accepted_by` at all**, and every one of the six was written on 2026-08-14. They are the three probes, `HW-OBL-0123`, `HW-OBL-0124` and `HW-SPEC-the-recorder-contract`.

So a second practice runs beside the one this record measures, and nothing declared it. An agent drafts, states the warrant it can state, and leaves the stamp to the reader of the proposal. That is what stop rule 5 asks for, and it is a shape rather than a ruling. The ruling this record waits on decides whether the six are the right answer or the start of a second gap.

**A third practice runs beside both, and it is worse than either.** The warrant reading of `taxonomy audit` reports the documents that state `warrant: proposed`, which spec 3's closed set does not hold. Each of them also names an acceptor, so each states a stamp under a warrant that requires nothing. [HW-OBL-0125](0125-nine-documents-state-a-warrant-the-closed-set-does-not-hold-and-no-check-reads-one.md) holds it. The ruling this record waits on now covers three practices rather than two.
