---
id: HW-OBL-0002
title: "Declarative voice is called detectable at useful precision, and the sample is unreachable"
status: current
status_since: 2026-08-11
waiting_on: adopter
last_verified: 2026-09-11
summary: "The three voice categories run over this corpus, and the largest reading is 36 findings against a sample bar of 50."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0005
    - HW-EVAL-what-a-check-can-know
---

# Declarative voice is called detectable at useful precision, and the sample is unreachable

## Context

[Spec 8](../spec/08-design-departures.md) calls declarative voice "mechanically detectable at useful precision". The measurement that closed [Q5](../spec/09-decisions.md#q5--voice-checking-depth) covered three ASD-STE100 structural rules, used as proxies, and none of the three survives. The passive, progressive and auxiliary detectors lived in `tools/ste-lint.py` and retired with it. Q5 measured their false-positive rates at 8%, 83% and 56%, and no rule blocked on any of the three.

So the measurement that closed Q5 does not run here, and the instrument that produced it is recoverable rather than lost. `tools/ste-lint.py` stands intact at `a39e3fb^`, which is commit `9f0d1d8`. [The evaluation](../evaluations/what-a-check-can-know.md) records the method beside it. The seed is `20260811`, and the samples are 60 of 449 passive findings, 25 of 54 auxiliary, and all 12 progressive. So a reader can repeat that measurement.

A rebuild of the four detectors adds nothing to this claim. The evaluation states that it did not measure `future_intent`, `change_narration` or `phased_rollout`, which are the categories that the declarative regime forbids. A detector nobody acts on is a warning count, and its reading is already recorded.

**The edge to that evaluation is `traces_to` and not `discharges`, and the difference matters.** The [decision-record entry](../taxonomies/decision-record/doctrine.md) declares `discharges` for the event that closes an entry, which is an instrument that ran and paid the debt. This record is open, and the evaluation says itself that it did not measure the three categories above. So `discharges` would state a closure that no run reached. What the taxonomy has no relation for is the honest link, which is an open record and the evidence it rests on. `cites_evidence` says that and runs from the three register kinds rather than from a record.

## Obligation

The instrument is a run of the three categories over this corpus, with an adjudicated sample of at least 50 findings each.

## Discharge

`voice.forbidden_construction` implements `future_intent`, `change_narration` and `phased_rollout`, which are the categories that the declarative regime forbids. So the implementation exists, and what this record holds open is the sample.

**The reading of 2026-09-11, on commit `58f46a8d`.** `change_narration` reports 36 findings, and the other two categories report none. So no category reaches the bar of 50. Edition two of the `change_narration` set reported 60 findings on the day it landed. This record then carried that figure through two merged rewrites that removed most of them. Write a count here as a dated reading that names the commit. A bare count goes stale on the next rewrite, and nothing reports it.

**All 36 findings were adjudicated one at a time, under the two labels spec 4 declares.** 13 are genuine. 5 stand in a section where the narration is the content a reader wants, such as the `## Context` of a decision record. 18 are false for good, and each one is a way that English states something other than a change. The five modes are a definition, a conditional inside a hypothetical, an inline quotation, another system's behavior, and a heading. The loosest pattern of the set carries 24 of the 36 findings, and 9 of those 24 are genuine. That pattern stays, and the argument is the absolute count rather than the rate. The rate moves in both directions. Over genuine findings alone it falls from 13 of 36 to 4 of 12. Over the findings that are not permanently false it rises from 18 of 36 to 9 of 12. The yield does not move, because 9 genuine findings go and no numerator returns them. `engine/crates/check/src/voice.rs` holds the ruling, one sentence for each of the five modes, and the trend that would retire the pattern. None of the 18 carries a `headwater allow` directive, because a directive requires an `until` and none of these sentences becomes wrong.

**The zero from the other two categories is saturation rather than a clean corpus.** Of the patterns that `future_intent` declares, 0 of 14 occur in the 291 documents whose kind binds the declarative regime. `phased_rollout` stands at 0 of 13 over the 293 documents that bind the declarative or the prospective regime. Both denominators come from the kind-to-regime binding of the lock, applied to the 320 documents of the graph. The corpus holds one occurrence of `will become` and two of `in the first release`, and all three sit under `docs/reviews/` or `docs/evaluations/`, which bind `narrative` and forbid nothing. [HW-OBL-0168](0168-a-saturated-pattern-set-and-a-clean-corpus-are-the-same-zero-and-no-report-separates-them.md) states why a zero from a pattern set carries two readings rather than one.

So this claim is short for all three categories rather than for two, and the reading above is the measurement that says so. A wider corpus is one way to reach the sample, and this record waits on one. A pattern set curated against the prose that the two silent categories are written in is the second way. It asks nothing of an adopter. Neither route is taken here.
