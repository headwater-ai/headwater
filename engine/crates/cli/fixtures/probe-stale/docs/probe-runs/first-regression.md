---
id: HW-RUN-first-regression
status: current
status_since: 2026-08-01
last_verified: 2026-08-01
summary: One recorded run over the whole selection of this fixture corpus, which is the committed transcript `probe stale` reads.
tier: regression
arm: present
provenance:
  warrant: asserted
  agency: agent
  drafted_by: a-recorder
  evidence_basis: evidenced
---

# One run over the whole selection

This document stands for what a recorder writes. Nothing in this engine produced it, because a transcript is observed from outside the session that produced it.

The run was planned over both probes of the selection, and its events name both. So the digest of the part its events name is the digest of the whole, and only the rule that a whole-selection digest is the whole keeps the report from calling this run a part of its own selection.

The `lock` below is not the lock of any tree. The test copies this repository's lock beside the corpus, so a fixed value would move with every taxonomy change, and a lock that moved while the read set held still leaves a transcript readable.

## Run identity

```yaml
model: a-model
served_version: a-model-20260701
tree: sha256:fixture-tree
lock: sha256:fixture
selection: sha256:eb7ca2fb2297d11f1d0b84a9fa56f1b09986076f09136e8b5dd6c38e1583c9ba
read_set: sha256:209ccb1b85fc4e2cd81e52611ac85a4c4900ad08af1d8ef1c9a6ed845e3318d9
seed: 0
harness: 0.1.0
tier: regression
arm: present
at: 2026-08-01
cost_cents: 50
```

## Events

```yaml
- probe: HW-PROBE-the-session-reads-the-document
  session: 1
  calls: []
  produced: []
  answer: null
- probe: HW-PROBE-the-session-reads-the-other-document
  session: 1
  calls: []
  produced: []
  answer: null
```
