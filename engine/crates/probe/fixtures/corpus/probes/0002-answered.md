---
id: PROBE-FIX-answered
status: current
status_since: 2026-08-14
summary: An answered probe, which names no document and is the one form that needs none.
probe_category: consistency
expectation: answered
oracle: "none"
---

# The session answers from the closed set

## Task

Say whether the cache may change a verdict. Answer `yes` or `no`.

## Expectation

`answered`, over the two values this block declares. A probe that named no set would be satisfied by every string a session returned.

```yaml
answers: [yes, no]
```
