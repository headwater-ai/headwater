---
id: NOTE-FIX-mentions-retired
status: current
status_since: 2026-08-02
relations:
  mentions:
    - NOTE-FIX-retired
summary: a live record that mentions a retired one through an association
---

# The live record mentioning a retired one

`association` carries no `lifecycle_sensitive` flag, so this pair reaches no
instance of this rule at all. A rule that read every edge between a live and a
terminal document reports it.
