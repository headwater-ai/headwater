---
id: RUN-FIX-first-regression
status: current
status_since: 2026-08-14
summary: One recorded run of the fixture selection, which stands for what a recorder writes.
tier: regression
arm: present
---

# One recorded run of the fixture selection

Nothing in this engine produced this file and nothing in it can. A transcript is observed from outside the session it records, and the prose around the blocks is what a person wrote about the run afterwards. Only the two fenced blocks are read.

The three sessions of `PROBE-FIX-opened` are the reason this file has three of them. One recorded a call that named the examined document, one recorded that it watched and saw no call, and one recorded no `calls` key at all. The first is satisfied, the second is not satisfied, and the third reaches no verdict and leaves the denominator of the rate.

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
  session: watched
  calls:
    - tool: read
      argument: /home/runner/repo/runs/probes/0002-answered.md
      result: sha256:a
  produced: []
  answer: null
- probe: PROBE-FIX-opened
  session: watched-and-saw-nothing
  calls: []
  produced: []
  answer: null
- probe: PROBE-FIX-opened
  session: nothing-watched
  produced: []
  answer: null
- probe: PROBE-FIX-answered
  session: watched
  calls: []
  produced: []
  answer: "no"
```
