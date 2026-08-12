---
id: SPEC-FIX-one-half
doc_type: design_spec
relations:
  cites_evidence:
    - EVAL-FIX-beta
  assesses:
    - EVAL-FIX-alpha
---

# One half

`beta.md` writes nothing back, so the inverse half of this pair is the one
nobody wrote. The finding reports here, at the entry this document wrote, and
its remediation names `beta.md`.

The `assesses` entry is here to prove the generation step. That relation
declares no `reciprocal`, so it produces no instance of the check at all.
