---
id: SPEC-FIX-unusable-entries
doc_type: design_spec
status: current
status_since: 2026-01-05
summary: The four entry defects that leave a relations block readable and its entries unusable.
relations:
  invented_relation: EVAL-FIX-alpha
  assesses:
    - EVAL-FIX-alpha
    - EVAL-FIX-alpha
    - [EVAL-FIX-alpha]
    - cue: an entry that names no target
---

# Entries a relations block cannot use

The block is a mapping and four of its entries declare no edge. `invented_relation` is neither a declared relation nor a declared inverse. The second `EVAL-FIX-alpha` repeats a triple that the first already declared, so it adds no edge. The third entry is a sequence, which is neither a target reference nor a mapping with `to`. The fourth is a mapping with no `to`, so it names no target at all.

The first entry is the passing half in the same file. It declares one `assesses` edge to an evaluation, and the endpoint rule and the target rule both read it.

`assesses` carries the four defects rather than `cites_evidence`, because `assesses` requires no reciprocity. A relation that required one would put a second finding on this document about the half that no evaluation wrote back.
