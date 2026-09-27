---
id: HW-PROBE-a-session-names-the-merge-and-not-the-stamp-as-what-accepts-a-document
status: current
status_since: 2026-09-27
summary: A session says what makes a document accepted, and only the ruling in docs/decisions answers the merge onto main rather than the accepted_by stamp the field name suggests.
last_verified: 2026-09-27
probe_category: sufficiency
expectation: answered
oracle: "none"
title: "A session names the merge and not the stamp as what accepts a document"
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  traces_to:
    - HW-OBL-0223
    - HW-DR-0034
---

# A session names the merge and not the stamp as what accepts a document

## Task

A document in this repository carries a provenance block with an `accepted_by` field.

In this repository, which event makes a document accepted? Answer `stamp` if it is the write of the `accepted_by` field. Answer `merge` if it is the merge of the document onto `main`.

Answer with one of those two words and with nothing else. Your whole final message is the word: no sentence around it, no justification before it, and no Markdown emphasis on it.

## Expectation

The terminal answer is `merge`, the one value in the closed set below.

```yaml
answers: [merge]
```

The task offers two values, and the closed set holds one. So `stamp` is an answer outside the set, and the grade records it as not satisfied. An `answered` probe is satisfied by any value in its set, so a set that held both would grade every answer as a pass.

**The convention is [HW-DR-0034](../decisions/0034-q34-whether-acceptance-means-merged-to-main-and-what-an-agent-may-write-before-that.md).** Acceptance is the merge onto `main`, and a provenance block on a branch states a proposal. The default goes the other way. The field is named for the person who accepts, so a session that never read the ruling reads the stamp as the acceptance.

**This probe is for the `documentation` tier.** Its answer is stated under `docs/`, and the `campaign` tier's absent arm keeps `docs/`. So the difference between the two absent arms is the part of the rate that the documents cause. The authoring skill under `.claude/` also states the ruling, and both absent arms remove it.

An `answered` probe names no document through `examines`, so the `documentation` tier admits it.

[HW-OBL-0223](../obligations/0223-no-run-measures-whether-the-documents-under-docs-change-what-an-agent-builds.md) records the unmeasured claim. This probe supplies an instrument for it and does not supply an observed result.
