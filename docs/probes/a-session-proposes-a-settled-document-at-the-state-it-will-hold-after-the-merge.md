---
id: HW-PROBE-a-session-proposes-a-settled-document-at-the-state-it-will-hold-after-the-merge
status: current
status_since: 2026-09-27
summary: A session says which status a settled decision carries in its pull request, and only the ruling in docs/decisions answers current rather than the draft the scaffolder writes.
last_verified: 2026-09-27
probe_category: sufficiency
expectation: answered
oracle: "none"
title: "A session proposes a settled document at the state it will hold after the merge"
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  traces_to:
    - HW-OBL-0223
    - HW-DR-0052
---

# A session proposes a settled document at the state it will hold after the merge

## Task

You add a new decision record to this repository. You consider the decision settled, and you open a pull request for it. A person reads the pull request before it merges.

Which value does the `status` field in the front matter of that document carry in the pull request? Answer `draft` or `current`.

Answer with one of those two words and with nothing else. Your whole final message is the word: no sentence around it, no justification before it, and no Markdown emphasis on it.

## Expectation

The terminal answer is `current`, the one value in the closed set below.

```yaml
answers: [current]
```

The task offers two values, and the closed set holds one. So `draft` is an answer outside the set, and the grade records it as not satisfied. An `answered` probe is satisfied by any value in its set, so a set that held both would grade every answer as a pass.

**The convention is [HW-DR-0052](../decisions/0052-a-document-is-proposed-at-the-state-it-will-hold-and-the-merge-activates-it.md).** An author writes the state that the document will hold once the branch lands, and the merge activates it. The default goes the other way. `headwater new` writes `draft`, because the lifecycle regime opens at that state, and "draft until merged" is the common practice elsewhere. A session that never read the ruling takes the default.

**This probe is for the `documentation` tier.** Its answer is stated under `docs/`, and the `campaign` tier's absent arm keeps `docs/`. So the difference between the two absent arms is the part of the rate that the documents cause. One copy of the convention sits outside both ablations: the hand-built tutorial page under `site/` states it in one step. A session that reads that page in the `documentation` absent arm can still answer `current`, so the measured difference is a lower bound.

An `answered` probe names no document through `examines`, so the `documentation` tier admits it.

[HW-OBL-0223](../obligations/0223-no-run-measures-whether-the-documents-under-docs-change-what-an-agent-builds.md) records the unmeasured claim. This probe supplies an instrument for it and does not supply an observed result.
