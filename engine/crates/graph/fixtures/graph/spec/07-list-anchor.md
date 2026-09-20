---
id: SPEC-FIX-list-anchor
doc_type: design_spec
relations:
  governs:
    - [graph.taxonomy.yml, graph.report]
    - [graph/no-such-file.rs, graph.report]
---

# A list is one anchor, and a dead member is reported for itself

The first entry under `governs` is a list of two real files, so this is one
edge over the union: one anchor node, reached by both members. The second
entry names one real file and one that is not there, so the whole entry is
unresolved, and the report names the dead member rather than the file
beside it that does exist. HW-DR-0074 rules both.
