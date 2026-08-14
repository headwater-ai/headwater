---
id: RUN-FIX-committed
status: current
status_since: 2026-08-14
summary: A transcript that lives on the shelf of this fixture corpus, so that a read set and a corpus tree are two different sets.
tier: regression
arm: present
---

# A committed run of the fixture selection

This document is on the corpus and `fixtures/transcript.md` is not, and the difference is the whole reason it is here. The read set of a run covers the probes of the selection and the documents they examine. The corpus tree digest covers every classified document. A fixture corpus in which those two sets are one set cannot tell a sharp instrument from a blunt one, and every test of the narrowing would pass under a whole-tree comparison.

So this file is a classified document that no probe examines. An edit to the prose here moves the tree digest and leaves the read set alone, which is the negative direction that the read-set tests assert.

The second event carries no `calls` key. That session is the one nothing watched, and a read set that read it as a session which opened no document would be a read set that no change could ever void.

## Run identity

```yaml
model: a-model
served_version: a-model-20260701
tree: sha256:fixture-tree
lock: sha256:fixture
selection: sha256:acb39c7d95c364df7aa7c4fd5a084010f3fbe9aacbf0586dc93126281a77084e
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
      argument: corpus/probes/0002-answered.md
      result: sha256:ab543ee59d5a46fd2fa8f4e1ce81e2de3a68926e0cdb53eacacf2a8b3a4adf76
  produced: []
  answer: null
- probe: PROBE-FIX-not-opened
  session: 1
  produced: []
  answer: null
```
