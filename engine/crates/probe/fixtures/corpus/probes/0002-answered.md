---
id: PROBE-FIX-answered
status: current
status_since: 2026-08-14
summary: An answered probe, whose closed set of values lives in the section its kind already requires.
probe_category: sufficiency
expectation: answered
oracle: "none"
---

# The session answers from the closed set

## Task

Say whether the cache may change a verdict. Answer `yes` or `no`.

## Expectation

`answered`, over the two values this block declares, and satisfied by the one it expects. A probe that named no set would be satisfied by every string a session returned, and a probe that expected every value of its set would be satisfied by every answer its task offers.

```yaml
answers: [yes, no]
expected: [no]
```
