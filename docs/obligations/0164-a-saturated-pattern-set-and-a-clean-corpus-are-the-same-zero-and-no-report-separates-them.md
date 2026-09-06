---
id: HW-OBL-0164
status: current
status_since: 2026-09-06
summary: "The voice rule reports zero because not one of its forty patterns occurs here, and that reads the same as a clean corpus."
last_verified: 2026-09-06
title: "A saturated pattern set and a clean corpus are the same zero, and no report separates them"
waiting_on: build
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-OBL-0002
    - engine/crates/check/src/voice.rs
---

# A saturated pattern set and a clean corpus are the same zero, and no report separates them

## Context

`voice.forbidden_construction` reads a curated pattern set for each category that a voice regime forbids. Edition one of that set holds forty patterns across three categories. Over this corpus it generates 258 instances and reports nothing.

A zero from a lexical rule carries two readings, and the report states neither. The first reading is a corpus that holds none of the fault. The second is a pattern set whose entries are absent from the language the corpus writes. A direct count separates them here: not one of the forty patterns of edition one occurs in a declarative document of this corpus.

[HW-OBL-0002](0002-declarative-voice-is-called-detectable-at-useful-precision.md) took the first reading. It asks for an adjudicated sample of at least 50 findings per category, and it records the count as an order of magnitude short. It concludes that no larger corpus under the declarative regime exists to run against, so the claim waits on an adopter.

Edition two of the `change_narration` set reports 60 findings over the same documents. The corpus that HW-OBL-0002 calls too small for the sample supplies it. So the bar that record waits on is a property of the pattern set rather than of the corpus.

**Curation against a corpus rejects candidates that intuition admits.** Three entries measured badly enough to stay out, and each failed for a reason a reader of the phrase alone would miss. `the retired` matches four sentences of this corpus and none is a narration, because "retired term" is the vocabulary of a rule here. `at one point` matches the positional sense in "worth making at one point in a text". `was replaced` matches three sentences that state a measurement procedure rather than a change to the system.

## Obligation

The corpus owes a reading, per lexical rule, of how many of the patterns of that rule match at least one sentence. A rule whose whole set matches nothing is reported as such on the run that reports its zero.

This reaches past voice. `language.retired_term.used` reads a list from the taxonomy and carries the same shape. A term retired long ago, and edited out of every document, reports zero on every run after that. The report says only that the rule passed.

## Discharge

A coverage line that names, for each rule with a pattern set, the count of patterns that matched and the count declared. Nothing here asks for a verdict on that ratio. A set with dead entries is legitimate, and a set with none alive is the finding.
