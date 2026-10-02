---
id: HW-PROBE-a-session-says-where-a-figure-on-the-site-is-written
status: current
status_since: 2026-10-03
summary: A search by name reaches a replaced ruling before the ruling in force, and the two give different answers, so a session that stops at the first hit answers wrong.
last_verified: 2026-10-03
probe_category: navigability
expectation: answered
oracle: "none"
title: "A session says where a figure on the site is written"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  traces_to:
    - HW-OBL-0014
---

# A session says where a figure on the site is written

## Task

A hand-built page of the website of this project shows a figure, such as the number of documents in this repository. Where is that figure written? Is it written into the page that is committed to the repository, or into the copy of the page that is built when the site is published?

Answer on one line with one word, `committed` or `published`, and with nothing else. Your whole final message is that word.

## Expectation

The terminal answer is `published`, the one expected value of the closed set below.

```yaml
answers: [committed, published]
expected: [published]
```

**This probe has the supersession-trap shape (#1472).** Two decision records rule on the question, and they give different answers. [HW-DR-0039](../decisions/0039-q39-how-a-figure-reaches-a-hand-built-page.md) is `superseded`. It rules that a script writes every figure on a hand-built page from a run, before the commit, so its answer is `committed`. [HW-DR-0097](../decisions/0097-a-figure-on-a-hand-built-page-is-measured-when-the-site-is-published-and-the-committed-page-carries-none.md) is `current` and supersedes it. It rules that the committed page carries no figure, and that a job measures each figure into the assembled copy when the site is published. So its answer is `published`.

**A search by name reaches the replaced ruling first.** Both file names hold `figure` and `hand-built-page`. In a listing of `docs/decisions/`, HW-DR-0039 comes before HW-DR-0097, because the listing is in the order of the number. So a session that lists the shelf, opens the first match and stops answers `committed`. The grade finds that answer wrong, because it is a value of the closed set and not the expected one.

**What a sound session does.** It reads the `status` of the first match, or its `superseded_by` edge, and goes on to HW-DR-0097. Or it reads the committed pages under `site/`, where each figure element is empty.

**The name of the ruling in force states its answer.** A session that reads the two file names side by side can answer from them. This probe does not test a vocabulary mismatch. It tests whether a session stops at the first document it reaches.

[HW-OBL-0014](../obligations/0014-no-probe-tests-whether-an-agent-reaches-the-adjudication-from.md) records that no probe tests whether an agent reaches the ruling from the document that lost it. The pair here is a supersession and not an adjudicated disagreement. So a run of this probe narrows that record and does not close it.
