---
id: OBL-repo-0001
title: "The promotion fix has no reading of the assisted fraction"
status: current
status_since: 2026-08-10
last_verified: 2026-08-14
summary: "Q4 claims that the promotion fix raises author-attributable edges without more hand entry, and no run supports it."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - DR-repo-0004
---

# The promotion fix has no reading of the assisted fraction

## Context

[Q4](../spec/09-decisions.md#q4--relation-storage) rules that front matter is authoritative and that a relation instance is an object. It carries a claim that no run supports: the promotion fix should raise author-attributable edges without a rise in hand entry.

## Obligation

The assisted fraction and the audit report are the instruments, and this corpus owes a reading of both.

## Discharge

The first typing of this corpus supplies the other end of that scale. An agent typed 36 documents and 203 edge halves with no scaffolder and no engine, so the assisted fraction of the run is zero. [What the first typing found](../spec/13-open-obligations.md#what-the-first-typing-of-this-corpus-found) records where the cost fell.

**A scaffolder exists, and it reports the fraction of the run it just did.** `headwater new` counts what it supplied against the four terms [spec 3](../spec/03-authoring-and-lifecycle.md#capture-cost-is-a-tracked-metric) names. They are the required front matter, the required sections, the identifier, and the halves of each proposed edge. This record carried a sentence that the treatment had never been applied to anything, and that sentence is false. What stays open is the claim itself, and the paragraphs below say why the verb does not settle it.

**One run is not a trend, and the metric that Q4 tests is a trend.** The claim is that author-attributable edges rise without a rise in hand entry. That compares two populations over time, and a verb that prints one number stores nothing. Capture-cost telemetry ([#74](https://github.com/headwater-ai/headwater/issues/74)) is the store, and it does not exist.

**No reading of a committed corpus can supply the other half either.** [Q4](../spec/09-decisions.md#q4--relation-storage) keeps `created_by` on the relation type, so the value states who a taxonomy expects to pay for an edge and never who wrote one. A scaffolded `supersedes` and a hand-typed `supersedes` are one string on disk. `taxonomy audit` counts edges by declared creator, which measures the intent of a declaration against a corpus. It cannot count what a scaffolder did. So the assisted fraction has to be taken at the moment the work happens, and every later reader of the corpus is blind to it.

**The one number this repository holds is from a run over a copy of its own tree.** `headwater new obligation_record` supplied 8 of 9 units: four of five front-matter fields, three of three section headings, and the identifier. The summary was the hand entry. A run that proposed one succession edge supplied 9 of 11. The two extra units are the halves of that edge, and the near half is the target that a person named. Both readings cover one document each, and neither is a baseline for a corpus.
