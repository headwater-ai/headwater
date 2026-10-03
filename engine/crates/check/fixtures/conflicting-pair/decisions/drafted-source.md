---
id: DEC-FIX-drafted-source
status: current
relations:
  conflicts_with:
    - DEC-FIX-drafted-target
summary: a current decision in conflict with a draft one
---

# drafted-source

Declares `conflicts_with` at `DEC-FIX-drafted-target`, which is a draft. Moving an end to the initial state trades this rule for `lifecycle.dependency.on_initial`, whose remedy is to promote the draft.
