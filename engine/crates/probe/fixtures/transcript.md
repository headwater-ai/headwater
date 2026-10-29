---
id: RUN-FIX-one
status: current
status_since: 2026-08-14
summary: One recorded run of the fixture selection, in which every predicate form is satisfied.
tier: regression
arm: present
---

# One run of the fixture selection

This document stands for what a recorder writes. Nothing in this engine produced it, and nothing in this engine can: a transcript is observed from outside the session that produced it, and the prose around the blocks is what a person wrote about the run afterwards.

Every one of the five predicate forms is satisfied here, and `transcript-missed.md` is the same run with every one of them refuted. Two files rather than one, because a grader that returned `satisfied` for everything would pass the first on its own.

## Run identity

```yaml
model: a-model
served_version: a-model-20260701
tree: sha256:fixture-tree
lock: sha256:fixture
selection: sha256:fixture-selection
read_set: sha256:662f7a5c4812c1f57bce8561be7eb24471042a7be1d472c72ca3b56ce5fad446
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
      argument: corpus/probes/0002-answered.md.bak
      result: sha256:a0
    - tool: read
      argument: /home/runner/repo/corpus/probes/0002-answered.md
      result: sha256:be177fea7c3cb22bbc0419a39601ae4010234b283d8fabf6300995690a396630
  produced: []
  answer: null
- probe: PROBE-FIX-answered
  session: 1
  calls:
    - tool: read
      argument: corpus/probes/0001-opened.md
      result: sha256:2afd22625f685b4b8165b714eb4cbe00b1df1c64bba4718687dc62858fe1c974
  produced: []
  answer: "no"
- probe: PROBE-FIX-not-opened
  session: 1
  calls:
    - tool: read
      argument: corpus/probes/0002-answered.md
      result: sha256:be177fea7c3cb22bbc0419a39601ae4010234b283d8fabf6300995690a396630
  produced: []
  answer: null
- probe: PROBE-FIX-cited
  session: 1
  calls: []
  produced:
    - path: out/report.md
      result: sha256:d
      cites: [PROBE-FIX-answered, PROBE-FIX-not-opened]
  answer: null
- probe: PROBE-FIX-patched
  session: 1
  calls: []
  produced:
    - path: out/patch.md
      result: sha256:e
      cites: []
      findings: []
  answer: null
- probe: PROBE-FIX-not-declared
  session: 1
  calls: []
```
