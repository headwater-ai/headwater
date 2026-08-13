---
"headwater:generated": "shelf_sections. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
doc_type: decision_register
id: REG-FIX-generated
---

# The register

A generated file that declares an identity. The marker is a member of the block
rather than the first line, because the first line of a document with front
matter is the fence.

The census reports it as generated, so no check reads it. It resolves a kind,
so the identifier index holds it and an edge can name `REG-FIX-generated`. Spec
6 excuses a generated file from checks and says nothing about identity, and
those are the two halves this fixture keeps apart.
