---
id: NOTE-FIX-skipped
status: current
status_since: 2026-08-01
summary: a record whose prior version stands at a value the state facet does not admit
---

# The skipped record

The version before this change wrote `retired`, which the vocabulary does not
hold. `facet.value.not_permitted` reports that, and this rule declines it.
