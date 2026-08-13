---
id: SPEC-FIX-one-half
doc_type: design_spec
status: current
status_since: 2026-01-05
summary: One half of a pair, and an edge of a relation that requires no reciprocity.
relations:
  cites_evidence:
    - EVAL-FIX-beta
  assesses:
    - EVAL-FIX-alpha
---

# One half

`beta.md` writes nothing back, so the inverse half of this pair is the one nobody wrote. The finding reports here, at the entry this document wrote, and its remediation names `beta.md`.

The `assesses` entry is here to prove two things. That relation declares no `reciprocal`, so it produces no instance of the reciprocity check at all. Its endpoints are both permitted, so it is the passing fixture for the endpoint rule: a `design_spec` at the source end, and an `evaluation` at the target end of a set that names `governed_document`.
