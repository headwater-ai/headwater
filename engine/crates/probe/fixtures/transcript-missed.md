---
id: RUN-FIX-three
status: current
status_since: 2026-08-14
summary: The same selection, recorded in a run where every predicate form is refuted.
tier: regression
arm: present
---

# The same run, with every expectation refuted

Spec 12 asks a check for one fixture it fails and one it passes, and a correctness root owes the same. This file is the failing half for all five predicate forms, and `transcript.md` is the passing half. A grader that returned one verdict for everything would pass exactly one of the two.

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
arm: absent
at: 2026-08-14
cost_cents: 39
```

## Events

```yaml
- probe: PROBE-FIX-opened
  session: 1
  calls:
    - tool: read
      argument: corpus/probes/0002-answered.md.bak
      result: sha256:a
  produced: []
  answer: null
- probe: PROBE-FIX-answered
  session: 1
  calls: []
  produced: []
  answer: maybe
- probe: PROBE-FIX-not-opened
  session: 1
  calls:
    - tool: read
      argument: corpus/probes/0001-opened.md
      result: sha256:b
  produced: []
  answer: null
- probe: PROBE-FIX-cited
  session: 1
  calls: []
  produced:
    - path: out/report.md
      result: sha256:c
      cites: [PROBE-FIX-patched]
  answer: null
- probe: PROBE-FIX-patched
  session: 1
  calls: []
  produced:
    - path: out/patch.md
      result: sha256:d
      cites: []
      findings: [section.required.missing]
  answer: null
```
