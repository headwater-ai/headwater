---
id: NOTE-FIX-rests-on-stub
status: current
status_since: 2026-08-02
relations:
  revises:
    - NOTE-FIX-quiet
summary: a live record resting on a document that declares no state at all
---

# The live record resting on a stateless document

There is no state at the far end to hold against anything, so the instance
skips with a reason rather than passing.
