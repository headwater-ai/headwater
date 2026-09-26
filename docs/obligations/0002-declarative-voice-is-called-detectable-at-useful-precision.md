---
id: HW-OBL-0002
title: "Declarative voice is called detectable at useful precision, and the sample is unreachable"
status: current
status_since: 2026-08-11
waiting_on: adopter
last_verified: 2026-09-27
summary: "The three voice categories run over this corpus, and no category reaches the adjudicated sample bar of 50 findings."
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

**The zero from the other two categories is saturation rather than a clean corpus.** On that same commit, of the patterns that `future_intent` declares, 0 of 14 occur in the 291 documents whose kind binds the declarative regime. `phased_rollout` stands at 0 of 13 over the 293 documents that bind the declarative or the prospective regime. Both denominators come from the kind-to-regime binding of the lock, applied to the 320 documents of the graph. The corpus holds one occurrence of `will become` and two of `in the first release`, and all three sit under `docs/reviews/` or `docs/evaluations/`, which bind `narrative` and forbid nothing. [HW-OBL-0168](0168-a-saturated-pattern-set-and-a-clean-corpus-are-the-same-zero-and-no-report-separates-them.md) states why a zero from a pattern set carries two readings rather than one.

So this claim is short for all three categories rather than for two, and the reading above is the measurement that says so. A wider corpus is one way to reach the sample, and this record waits on one. A pattern set curated against the prose that the two silent categories are written in is the second way. It asks nothing of an adopter. Neither route is taken here.

**The reading of 2026-09-27, on commit `0ee88a2d`.** The reading of 2026-09-11 counted the body alone. Edition four of the rule also reads the `summary` facet, so this reading counts the body and the `summary` together. The rule reports 23 findings over 376 rule instances. `change_narration` reports 22, `future_intent` reports 1, and `phased_rollout` reports none. So no category reaches the bar of 50, and `change_narration` is further from it than on 2026-09-11.

**All 23 findings were adjudicated one at a time.** 7 are genuine, and each one narrates a change to this repository. 4 stand where the narration is the content a reader wants: 3 in a `## Context` section and 1 in a dated measurement. 12 are false, and no rewrite repairs them. 11 of the 12 are `change_narration`, in the five modes above. The twelfth is the one `future_intent` finding, and it is a sixth mode: a general prediction that states a property of a thing. It is in `docs/taxonomies/diataxis/doctrine.md`. So the first finding that `future_intent` has reported on this corpus is not evidence that the set detects the fault.

Each finding carries one label. `genuine` is a finding to repair. `accepted_deviation` and `false_positive` are the two labels that spec 4 declares, and this reading spends no directive on either.

| document and line | pattern | label | reason |
|---|---|---|---|
| `docs/decisions/0008-probe-cost-and-cadence.md:26` | `there was no` | `accepted_deviation` | history in the `## Context` section |
| `docs/decisions/0012-migration-path-for-an-existing-corpus.md:42` | `no longer` | `false_positive` | the present behavior of another tool |
| `docs/decisions/0017-governed-access-and-the-solution-layer.md:94` | `did not exist` | genuine | tells the change that a ruling made |
| `docs/decisions/0022-q22-the-integrity-posture-of-a-published-package.md:26` | `did not exist` | `accepted_deviation` | history in the `## Context` section |
| `docs/decisions/0024-q24-readability-and-what-a-sweep-can-be-asked-about.md:54` | `no longer` | `false_positive` | a conditional inside a hypothetical |
| `docs/decisions/0039-q39-how-a-figure-reaches-a-hand-built-page.md:54` | `no longer` | `false_positive` | the condition that makes a script fail |
| `docs/decisions/0051-q51-what-licenses-a-term-into-the-public-glossary-and-what-licenses-a-sixteenth.md:48` | `did not exist` | `accepted_deviation` | a dated measurement in the `## Consequences` section |
| `docs/decisions/0054-the-upper-bound-of-a-reconcile-first-allocator-is-the-corpus-and-a-claim-store.md:49` | `no longer` | `false_positive` | a definition |
| `docs/interfaces/headwater-probe.md:33` | `no longer` | `false_positive` | a definition |
| `docs/obligations/0129-spec-12-calls-two-phase-a-outcomes-structural-findings-and-the-engine-emits-none.md:29` | `no longer` | `false_positive` | a definition, in the `## Context` section |
| `docs/obligations/0130-the-publication-date-of-this-repository-is-unset-and-the-owner-alone-sets-it.md:28` | `there was no` | `accepted_deviation` | history in the `## Context` section |
| `docs/spec/07-distribution-and-federation.md:159` | `no longer` | `false_positive` | a conditional inside a hypothetical |
| `docs/spec/glossary.md:592` | `no longer` | `false_positive` | the definition of a retired term |
| `docs/spec/glossary.md:811` | `used to` | `false_positive` | a heading |
| `docs/taxonomies/README.md:225` | `used to` | genuine | tells what a fixture README said before |
| `docs/taxonomies/README.md:233` | `has since` | genuine | tells the history of a milestone |
| `docs/taxonomies/decision-record/doctrine.md:69` | `no longer` | genuine | tells a move of a facet to the base |
| `docs/taxonomies/decision-record/doctrine.md:100` | `no longer` | `false_positive` | a table row that defines a state |
| `docs/taxonomies/design-spec/doctrine.md:137` | `has since` | genuine | tells a change to spec 2 |
| `docs/taxonomies/diataxis/doctrine.md:46` | `will eventually` | `false_positive` | a general prediction that states a property |
| `docs/taxonomies/diataxis/doctrine.md:137` | `has since` | `false_positive` | a conditional inside a hypothetical |
| `docs/taxonomies/diataxis/doctrine.md:141` | `has since` | genuine | tells that a demonstration ran |
| `docs/taxonomies/evidence-and-obligation/doctrine.md:28` | `used to` | genuine | tells what an older kind carried |

The rate of each pattern is its genuine findings over its findings. `no longer` is 1 of 10. `has since` is 3 of 4. `used to` is 2 of 3. `did not exist` is 1 of 3. `there was no` is 0 of 2, and both are `accepted_deviation`. `will eventually` is 0 of 1. Two labels are close calls. The line of `0051` could be genuine, because it tells a change to the site. The line of the `design-spec` doctrine could be `accepted_deviation`, because its section records what a draft assumed. Neither call moves a ruling below.

**The loosest pattern now yields 1 genuine finding of 10.** `no longer` carries 10 of the 22 `change_narration` findings. Since 2026-09-11 its genuine count fell from 9 to 1, and its false count fell too. The quotation exclusion of #783 removed one false finding, and rewrites made for other reasons changed the words of others. So the false count is not a floor that no rewrite reaches. The ruling in `engine/crates/check/src/voice.rs` retires the pattern when a reading finds the genuine count at zero and the false count unmoved. This reading sets the baseline for that ruling: 1 genuine and 9 false. A later reading compares its false count with 9, not with the figures of 2026-09-11.

**The two quiet categories are still at saturation.** Of the patterns that `future_intent` declares, 2 of 14 occur in the 378 documents whose kind binds the declarative regime. Both occurrences of `will be` are inside a quotation, which the rule does not read. The other pattern is the false finding above. Of the patterns that `phased_rollout` declares, 0 of 13 occur in the 382 documents that bind the declarative or the prospective regime. `change_narration` has 5 of its 23 patterns in the same 382. Both denominators come from the kind-to-regime binding of the lock, applied to the 421 documents of the graph. 39 of those documents bind `narrative`.

**No voice regime can name the sections that it reads, and this record prices that ruling.** A decision record keeps history in its `## Context` section, and the rule reads that section as it reads every other section. 4 of the 23 findings stand in a `## Context` section. 3 of them are narration that a reader wants there, and 1 is a false definition. A section scope clears those 4 findings. It does not clear the fourth legitimate narration, which stands in a `## Consequences` section. So a schema change buys 4 findings, and it is declined. The convention for an author is a block directive with `reason=accepted_deviation`, an `until` and a note. The convention costs 4 directives today, and this corpus spends none. A directive requires an `until`, and a sentence about history does not become wrong on a date.

**This record still waits on an adopter.** The second route is a curated pattern set, and `build` would say that its answer is known. No candidate pattern for `future_intent` or `phased_rollout` has been measured against this corpus. So that route waits on a measurement that nobody has designed, and the wider corpus is still the route with a known shape.
