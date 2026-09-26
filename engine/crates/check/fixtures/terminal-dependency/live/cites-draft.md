---
id: NOTE-FIX-cites-draft
status: current
status_since: 2026-08-02
relations:
  cites:
    - NOTE-FIX-argued-over
summary: a live record that cites a draft one through a relation that marks itself
---

# The live record citing a draft

`cites` declares `lifecycle_sensitive: true` for itself. The initial reading
does not read that word, so this pair is reported for the same reason as the
unmarked one beside it.
