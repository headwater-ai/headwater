---
id: NOTE-FIX-open-listed-outside
evidence_basis: [asserted, bogus]
summary: a document whose list holds one admitted value and one outside the set
---

# The list with one item outside

`asserted` is admitted and `bogus` is not. Each scalar item of a list is read against the set, so this document is reported once, at `bogus` (#1873).
