---
id: PROBE-FIX-opened
status: current
status_since: 2026-08-14
summary: An opened probe, whose three recorded sessions are the three outcomes a verdict has.
probe_category: discovery
expectation: opened
oracle: "none"
relations:
  examines:
    - PROBE-FIX-answered
---

# The session reads the document it was pointed at

## Task

Read the document this probe names, and say what it governs.

## Expectation

`opened` over the document it names. The transcript runs this probe three times, and the three sessions are what separate a verdict from a missing one.
