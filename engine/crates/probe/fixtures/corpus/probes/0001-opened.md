---
id: PROBE-FIX-opened
status: current
status_since: 2026-08-14
summary: A well-formed probe that names one document, which is the arm every refusal below is measured against.
probe_category: discovery
expectation: opened
oracle: "none"
relations:
  examines:
    - PROBE-FIX-answered
---

# The session reads the document

## Task

Do the thing the document governs.

## Expectation

`opened` over the document it names.
