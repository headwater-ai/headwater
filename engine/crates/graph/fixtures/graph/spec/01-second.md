---
id: SPEC-FIX-second
doc_type: design_spec
relations:
  invented_relation: SPEC-FIX-first
  traces_to:
    - EVAL-FIX-nowhere
    - SPEC-FIX-unshelved
    - SPEC-FIX-generated
    - graph.taxonomy.yml
  audited_by: 12345
  cites_evidence:
    - to: EVAL-FIX-alpha
      cue: a target with an attribute
    - cue: an entry with no target at all
    - [EVAL-FIX-alpha]
---

# A heading

Every entry under `traces_to` reaches a different outcome, and the relation admits both a document and an anchor, which is what makes the order of resolution visible. `EVAL-FIX-nowhere` is nothing. `SPEC-FIX-unshelved` is a real document that no shelf claims, so the census gave it no kind and it is not an endpoint. `SPEC-FIX-generated` is a file this engine wrote, and it declares a kind, so it is a node like any other document. `graph.taxonomy.yml` is a file, so the `code_path` resolver takes it.

`audited_by` names the `ado_work_item` anchor kind, whose resolver reads a committed snapshot that this run does not have.

The three entries under `cites_evidence` are the three shapes an entry can take. The first is a mapping with `to` and an instance attribute. The second is a mapping with no `to`, so it names no target. The third is a sequence, which is neither a target reference nor a mapping, and an entry nested inside an entry names nothing either.
