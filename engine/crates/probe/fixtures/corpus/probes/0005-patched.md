---
id: PROBE-FIX-patched
status: current
status_since: 2026-08-14
summary: The oracle route, and the one form whose expectation names a check this engine carries.
probe_category: sufficiency
expectation: patched
oracle: section.required.missing
---

# The produced patch passes the check it is graded against

## Task

Repair the document the task names.

## Expectation

`patched` against the oracle in the front matter. An artifact that carries no `findings` key was never checked, and the grader refuses that session rather than reading an absent key as a clean one.
