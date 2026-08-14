---
id: RUN-FIX-four
status: current
status_since: 2026-08-14
summary: A run the recorder wrote down incompletely, where every form reaches a refusal rather than a verdict.
tier: regression
arm: present
---

# The run nothing watched closely enough

Every event below is well-formed, so the intake accepts the file whole and the five confirmations pass. What is missing is the key each predicate reads. An empty list says the recorder watched and saw nothing, and an absent key says nothing watched, and a grader that read the second as the first would return a passing verdict about a run nobody observed.

`PROBE-FIX-answered` carries an event with no answer key. Remove that event and the probe reaches no verdict at all, which is the sixth refusal and the one no malformed key can produce.

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
cost_cents: 12
```

## Events

```yaml
- probe: PROBE-FIX-opened
  session: 1
  produced: []
  answer: null
- probe: PROBE-FIX-answered
  session: 1
  calls: []
  produced: []
- probe: PROBE-FIX-not-opened
  session: 1
  calls: []
  produced: []
  answer: null
- probe: PROBE-FIX-cited
  session: 1
  calls: []
  answer: null
- probe: PROBE-FIX-patched
  session: 1
  calls: []
  produced:
    - path: out/patch.md
      result: sha256:a
      cites: []
  answer: null
```
