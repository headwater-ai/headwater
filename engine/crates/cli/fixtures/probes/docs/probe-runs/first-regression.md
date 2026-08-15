---
id: RUN-HW-first-regression
status: current
status_since: 2026-08-01
last_verified: 2026-08-01
summary: One recorded run over this fixture corpus, which is the committed transcript the two verbs below read.
tier: regression
arm: present
provenance:
  warrant: asserted
  agency: agent
  drafted_by: a-recorder
  evidence_basis: evidenced
---

# One run over the fixture corpus

This document stands for what a recorder writes. Nothing in this engine produced it and nothing in this engine can, because a transcript is observed from outside the session that produced it.

Neither verb the test drives reads a verdict out of it. The plan over this corpus stops partway, so what a caller does with the refusal beside the partial selection is the whole of what the test measures, and the events below only make the file a transcript.

## Run identity

```yaml
model: a-model
served_version: a-model-20260701
tree: sha256:fixture-tree
lock: sha256:fixture
selection: sha256:fixture-selection
read_set: sha256:fixture-read-set
seed: 0
harness: 0.1.0
tier: regression
arm: present
at: 2026-08-01
cost_cents: 25
```

## Events

```yaml
- probe: PROBE-HW-the-session-reads-the-document
  session: 1
  calls:
    - tool: read
      argument: docs/probes/0002-the-category-is-outside-the-closed-set.md
      result: sha256:fixture-read
  produced: []
  answer: null
```
