---
id: NOTE-FIX-k
relations:
  traces_to:
    - document-path-target/notes/b.md
    - ./document-path-target/notes/../notes/b.md
summary: an explanation that names one document by its path in two spellings
---

# an explanation that names one document by its path in two spellings

Both entries normalize to the path of `NOTE-FIX-b`. The first is the finding of `notes/a.md`. The second names the same target twice, and `relation.declaration.unusable` reports the repeat, as it did when both entries bound as one anchor.
