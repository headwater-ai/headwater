---
id: DR-FIX-0001
status: current
status_since: 2026-09-28
summary: A decision that governs one file and three list anchors.
relations:
  governs:
    - src/c.rs
    - [src/a.rs, src/b.rs]
    - [src/c.rs, "src/a, b.txt"]
    - [src/c.rs, src/b.rs, src/a.rs]
---

# Govern two lists

The first entry is one pattern. The second is a list of two files, which is one anchor over the union of its members. The third is a list whose first member holds a comma and a space, which is the separator of a joined list, so a consumer that splits a joined string reads three members where the export holds two. The fourth is a list of three files, written out of order, so an emitter that writes members only for a list of exactly two drops it.
