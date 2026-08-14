---
id: PROBE-FIX-cited
status: current
status_since: 2026-08-14
summary: A predicate over a produced artifact, which reads an identifier where the two forms above read a path.
probe_category: sufficiency
expectation: cited
oracle: "none"
relations:
  examines:
    - PROBE-FIX-answered
---

# The produced artifact names what licensed it

## Task

Write the artifact the task asks for, and cite what governs it.

## Expectation

`cited` over the identifiers it names. The recorder writes every identifier it finds in the artifact and never the ones this probe declares, so the selecting and the comparing are both the grader's.
