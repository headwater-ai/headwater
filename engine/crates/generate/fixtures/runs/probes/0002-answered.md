---
id: PROBE-FIX-answered
status: current
status_since: 2026-08-14
summary: An answered probe, which is also the document the opened probe examines.
probe_category: sufficiency
expectation: answered
oracle: "none"
---

# The session answers from the closed set

## Task

Say whether the cache may change a verdict. Answer `yes` or `no`.

## Target

The document that the key reads.

## Key

`answered`, over the two values this block declares. A probe that named no set would be satisfied by every string a session returned.

```yaml
answers: [yes, no]
expected: [no]
```
