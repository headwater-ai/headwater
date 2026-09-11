---
id: HW-OBL-0005
title: "A closed-set expectation has no reproducibility reading under grading"
status: current
status_since: 2026-08-11
waiting_on: build
last_verified: 2026-08-13
summary: "Q8 claims that a closed-set expectation makes a probe verdict reproducible, and the one closed-set probe this corpus declares has now returned a verdict."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0008
---

# A closed-set expectation has no reproducibility reading under grading

## Context

[Q8](../spec/09-decisions.md#q8--probe-cost-and-cadence) rules that a probe is a document with a declared expectation. A closed-set expectation should make a probe verdict reproducible under grading.

## Obligation

`generate --check` over a result document is the instrument, and this corpus owes a reading of it.

## Discharge

`headwater generate --check` exists and it now holds a result document of this repository. [The result of 2026-09-11](../probe-results/regression-probe-transcript-for-2026-09-11.md) carries four verdicts of four, and one of the four probes declares a closed answer set. So the instrument has a reading at last.

**The reading does not support the claim, and it does not refute it either.** The closed-set session of that run ended with the word `present` on the harness result line, and `tools/probe/probe-record.sh` writes no `answer` key value for any probe, so the transcript records `answer: null` and the verdict grades the recorder. One run is also one run: reproducibility is a comparison of two verdicts over one expectation, and this corpus holds one. What this record now waits on is a second run over a recorder that observes the answer.
