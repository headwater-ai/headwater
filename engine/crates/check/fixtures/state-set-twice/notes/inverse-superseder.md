---
id: NOTE-FIX-inverse-superseder
status: current
relations:
  supersedes:
    - NOTE-FIX-gen-inverse
  cites:
    - NOTE-FIX-gen-inverse
summary: the one setter of `NOTE-FIX-gen-inverse`, which also cites it
---

# inverse-superseder

Declares `supersedes` and `cites` at `NOTE-FIX-gen-inverse`. `cites` writes no
state, so it is never half of a clash.
