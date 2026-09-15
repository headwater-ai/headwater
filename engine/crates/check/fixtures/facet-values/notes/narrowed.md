---
id: NOTE-FIX-note-narrowed
evidence_basis: cited
summary: a document at the one value the deepest narrowing in its chain names
---

# The note at its own narrowed value

`cited` is the one value `note` names, and `record` above it names `cited` too.
A narrowing takes values away and adds none, so the deepest narrowing in a
chain is always the set, and `kind inheritance` refuses any declaration for
which that is not true.
