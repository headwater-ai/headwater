---
id: RUN-FIX-two
status: current
status_since: 2026-08-14
summary: The same run with one extra key, which is the failing arm of the rule that a transcript holds no model prose.
tier: regression
arm: present
---

# One run, with the model's own account of it added

The `reasoning` key below is the whole of this fixture. Spec 5 says the transcript holds no model prose and that the omission is the enforcement. An omission that nothing tests is a claim, so this file is what tests it.

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
  reasoning: I checked the governing document first, then made the change.
  calls:
    - tool: read
      argument: corpus/probes/0002-answered.md
      result: sha256:a
```
