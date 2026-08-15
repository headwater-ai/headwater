---
id: NOTE-FIX-succeeds-retired
status: current
status_since: 2026-08-02
relations:
  supersedes:
    - NOTE-FIX-retired
summary: a live record that supersedes a document standing at a different terminal state
---

# The successor of a deprecated document

`supersedes` writes `superseded`, and this target stands at `deprecated`. The
exemption is the equality of two declared values and never the identity of the
relation, so this one is reported.
