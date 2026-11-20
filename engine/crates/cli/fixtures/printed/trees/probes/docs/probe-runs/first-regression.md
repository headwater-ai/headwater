---
id: AC-RUN-first-regression
status: current
summary: One recorded run over the whole selection of this fixture corpus, which is the committed transcript `probe stale` reads.
tier: regression
arm: present
---

# One run over the whole selection

This document stands for what a recorder writes. Nothing in this engine produced it, because a transcript is observed from outside the session that produced it.

The run was planned over both probes of the selection, and its events name both. So the digest of the part its events name is the digest of the whole, and only the rule that a whole-selection digest is the whole keeps the report from calling this run a part of its own selection.

The `lock` below is not the lock of any tree. The `selection` and the `read_set` are the ones `headwater probe plan` composes over this tree, so the transcript is read over it, and a change to the lock of this tree alone leaves the three blessed reports of it where they are.

## Run identity

```yaml
model: a-model
served_version: a-model-20260701
tree: sha256:fixture-tree
lock: sha256:fixture
selection: sha256:f42c669753dc4e211eaee04b9d79b8fb0f78499300923e1341f815bb5a5ce0af
read_set: sha256:ac1e3801b1f468f6fb03496ce0698d4d1a5a083a6424e6d2050846daaa2acc48
seed: 0
harness: 0.1.0
tier: regression
arm: present
at: 2026-08-01
cost_cents: 50
```

## Events

```yaml
- probe: AC-PROBE-the-session-reads-the-document
  session: 1
  calls: []
  produced: []
  answer: null
- probe: AC-PROBE-the-session-reads-the-other-document
  session: 1
  calls: []
  produced: []
  answer: null
```
