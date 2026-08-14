---
id: PROBE-FIX-not-opened
status: current
status_since: 2026-08-14
summary: A negative predicate, which is the one form that a session doing nothing would satisfy for free.
probe_category: navigability
expectation: not_opened
oracle: "none"
relations:
  examines:
    - PROBE-FIX-opened
---

# The session reaches the answer without the document

## Task

Do the thing, and do it without opening the document this probe names.

## Expectation

`not_opened` over the document it names. A session that recorded no tool call at all satisfies this predicate by doing nothing, so the grader refuses that session rather than counting it.
