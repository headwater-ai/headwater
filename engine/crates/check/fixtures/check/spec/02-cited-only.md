---
id: SPEC-FIX-cited-only
doc_type: design_spec
---

# Cited only

`delta.md` writes `cited_by: SPEC-FIX-cited-only`, and this document writes no
`cites_evidence`. So the missing half is the one under the relation's *own*
name, which is the other of the two cases. This document declares no edge and it
is still checked, because the instance over that pair reads both endpoints.
