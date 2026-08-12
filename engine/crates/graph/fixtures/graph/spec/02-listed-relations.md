---
id: SPEC-FIX-listed
doc_type: design_spec
relations:
  - cites_evidence: EVAL-FIX-alpha
---

# A relations block that is a sequence

Q4 puts every relation instance under one `relations:` key, and the block is a mapping of relation names. A sequence is a shape nothing can read, and it is reported once against the block rather than guessed at.
