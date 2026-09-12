---
id: NOTE-FIX-cites-retired
status: current
status_since: 2026-08-02
relations:
  cites:
    - NOTE-FIX-retired
summary: a live record that cites a retired one through a relation that marks itself
---

# The live record citing a retired one

`cites` is `evidence`, and no core requirement of this taxonomy names that
family. The relation declares `lifecycle_sensitive: true` on its own body, so
the rule reads this pair. A rule that took the word from the family alone is
silent here.
