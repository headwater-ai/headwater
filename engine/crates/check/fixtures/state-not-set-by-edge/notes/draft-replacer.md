---
id: NOTE-FIX-draft-replacer
status: draft
status_since: 2026-08-07
relations:
  supersedes:
    - NOTE-FIX-draft-replaced
    - NOTE-FIX-already-retired
    - NOTE-FIX-deprecated-early
summary: a draft successor of three targets, one at each state the rule must tell apart
---

# draft-replacer

A draft over a draft target, over a target already at the set state, and over a target terminal at another state. Only the last is pending, so lifecycle.state.set_after_initial warns once (#1973).
