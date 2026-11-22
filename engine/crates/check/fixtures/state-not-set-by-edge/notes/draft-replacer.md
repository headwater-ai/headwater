---
id: NOTE-FIX-draft-replacer
status: draft
status_since: 2026-08-07
relations:
  supersedes:
    - NOTE-FIX-draft-replaced
    - NOTE-FIX-already-retired
    - NOTE-FIX-deprecated-early
    - NOTE-FIX-drafted-page
summary: a draft successor of four targets, each a case the rule must tell apart
---

# draft-replacer

A draft over a draft target, over a target already at the set state, over a generated page, and over a target terminal at another state. Only the last is pending, so lifecycle.state.set_after_initial warns once (#1973).
