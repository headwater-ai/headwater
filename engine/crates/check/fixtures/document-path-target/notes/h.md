---
id: NOTE-FIX-h
relations:
  cites:
    - document-path-target/notes/b.md
summary: an explanation that cites an item in another system whose name spells a document's path
---

# an explanation that cites an item in another system whose name spells a document's path

`cites` reaches a `snapshot_item`, whose resolver is not `source-tree`. The string names an item in that system and not a file in this tree, so the entry is an anchor and no finding.
