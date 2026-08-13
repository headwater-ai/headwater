---
doc_type: design_spec
status: current
status_since: 2026-01-05
summary: A typed document with no identifier, and a relations block that loses its source.
relations:
  assesses:
    - EVAL-FIX-alpha
---

# A document that nothing can name

An edge is identified by its source, so a document with no identifier declares nothing that can be one. The graph build reports two facts about this file. The identifier index reports that a typed `design_spec` declares no identifier. The edge build reports that the `relations:` block below it has no source.

One rule answers for both, because both name one repair. To mint an identifier for this document restores its own identity and every edge the block declares.
