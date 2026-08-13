---
id: OBL-repo-0002
title: "Declarative voice is called detectable at useful precision, and the sample is unreachable"
status: current
status_since: 2026-08-11
last_verified: 2026-08-13
summary: "The three voice categories run over this corpus, and each one is an order of magnitude short of the sample the instrument asks for."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - DR-repo-0005
---

# Declarative voice is called detectable at useful precision, and the sample is unreachable

## Context

[Spec 8](../spec/08-design-departures.md) calls declarative voice "mechanically detectable at useful precision". The measurement that closed [Q5](../spec/09-decisions.md#q5--voice-checking-depth) covered three ASD-STE100 structural rules, used as proxies, and none of the three survives. The passive, progressive and auxiliary detectors lived in `tools/ste-lint.py` and retired with it. Q5 measured their false-positive rates at 8%, 83% and 56%, and no rule blocked on any of the three.

So the measurement that closed Q5 cannot be repeated here, and an instrument that produced a closed finding is gone. That is a trade rather than an oversight. A detector nobody acts on is a warning count, and its reading is already recorded.

## Obligation

The instrument is a run of the three categories over this corpus, with an adjudicated sample of at least 50 findings each.

## Discharge

`voice.forbidden_construction` implements `future_intent`, `change_narration` and `phased_rollout`, which are the categories that the declarative regime forbids. The first run reports 21 findings over the three, and the largest category holds 10. So the implementation exists and the sample the instrument asks for does not.

Each category is an order of magnitude short of 50, and no larger corpus under the declarative regime exists to run against. This claim stays unmeasured until the instrument names a reachable sample, a wider corpus, or both.
