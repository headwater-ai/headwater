---
id: HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer
status: draft
status_since: 2026-08-27
summary: A session classifies a recovery word as present, withheld, or absent from a served corpus with a counted tombstone.
last_verified: 2026-08-27
probe_category: sufficiency
expectation: answered
oracle: "none"
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: codex
  activity: draft
  evidence_basis: unevidenced
relations:
  traces_to:
    - HW-OBL-0013
---

# A counted tombstone separates a withheld answer from an absent answer

## Task

Inspect the served corpus for the recovery word.

Answer only `present` if the corpus states `amber`.

Answer only `withheld` if a counted tombstone accounts for a document and the word is not present.

Answer only `absent` if neither condition holds.

## Expectation

The terminal answer is one value from the closed set below. The three task conditions make these values mutually exclusive.

```yaml
answers: [withheld, absent, present]
```

An `answered` probe names no document through `examines`. Its answer set defines the predicate domain.

[HW-OBL-0013](../obligations/0013-no-probe-tests-whether-a-counted-tombstone-stops-a-confident.md) records the unmeasured claim. This probe supplies its instrument and does not supply an observed result.
