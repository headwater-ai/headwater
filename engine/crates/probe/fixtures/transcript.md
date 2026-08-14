---
id: RUN-FIX-one
status: current
status_since: 2026-08-14
summary: One recorded run of the fixture selection, standing for what a recorder writes.
tier: regression
arm: present
---

# One run of the fixture selection

This document stands for what a recorder writes. Nothing in this engine produced it, and nothing in this engine can: a transcript is observed from outside the session that produced it, and the prose around the blocks is what a person wrote about the run afterwards.

## Run identity

```yaml
model: a-model
served_version: a-model-20260701
tree: sha256:fixture-tree
lock: sha256:fixture
selection: sha256:fixture-selection
seed: 0
harness: 0.1.0
tier: regression
arm: present
at: 2026-08-14
cost_cents: 41
```

## Events

```yaml
- probe: PROBE-FIX-opened
  session: 1
  calls:
    - tool: read
      argument: corpus/probes/0002-answered.md
      result: sha256:a
  produced: []
  answer: null
- probe: PROBE-FIX-answered
  session: 1
  calls:
    - tool: read
      argument: corpus/probes/0001-opened.md
      result: sha256:b
  produced: []
  answer: "no"
- probe: PROBE-FIX-not-declared
  session: 1
  calls: []
```
