---
id: SPEC-FIX-not-a-mapping
doc_type: design_spec
status: current
status_since: 2026-01-05
summary: A relations block written as a sequence, which names no relation type.
relations:
  - cites_evidence: EVAL-FIX-alpha
---

# A relations block that is not a mapping

Q4 puts every relation instance under one `relations:` key, and the keys of that block are relation names. A sequence names none. The whole block declares no edge, and the build reports it once against the block rather than guessing at it.

This is the first of the four ways a block can fail. It has a file of its own because it stops the build before it reads an entry. No other defect of a block is visible beside it.
