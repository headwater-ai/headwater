---
"headwater:generated": "shelf_sections. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: SPEC-FIX-generated
doc_type: design_spec
relations:
  traces_to:
    - SPEC-FIX-first
---

# A generated document that declares an identity

The census reports this file as generated, so no check reads it. It resolves a kind, so it is a node: `01-second.md` names `SPEC-FIX-generated` under `traces_to` and that edge binds to a document rather than to nothing.

It is a source as well as a target. The entry above is a fact its emitter took from the corpus, and a wrong one is repaired in the declaration that writes this file rather than here.
