---
id: SPEC-FIX-first
doc_type: design_spec
---

# Two documents, one identifier

Spec 3 mints an identifier once and never reuses it. Where two documents claim one, a target that names it resolves to two documents, and no rule downstream can pick between them.
