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
read_set: sha256:9cd0571c32a18081b2fd2f031a69d16f7260bd9a9c77e157884ec0d5ea6968fb
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
      result: sha256:ab543ee59d5a46fd2fa8f4e1ce81e2de3a68926e0cdb53eacacf2a8b3a4adf76
  produced: []
  answer: null
- probe: PROBE-FIX-answered
  session: 1
  calls:
    - tool: read
      argument: corpus/probes/0001-opened.md
      result: sha256:d096faf5e06e2dc870c3bf0fdb47fa2dd9c8c7883fd1f81d8fabaff7b36ccd9a
  produced: []
  answer: "no"
- probe: PROBE-FIX-not-opened
  session: 1
  calls:
    - tool: read
      argument: corpus/probes/0002-answered.md
      result: sha256:ab543ee59d5a46fd2fa8f4e1ce81e2de3a68926e0cdb53eacacf2a8b3a4adf76
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
