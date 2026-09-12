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

Neither reading marks `mentions`: no core requirement declares `association`
lifecycle-sensitive, and the relation declares nothing for itself. So this pair
reaches no instance of this rule at all. A rule that read every edge between a
live and a terminal document reports it, and so does a rule that marked a
relation because some other relation declared the word.
