---
id: SPEC-FIX-dangling
doc_type: design_spec
status: current
status_since: 2026-01-05
summary: Two edges whose targets bind to nothing, in the two ways a document target can.
relations:
  assesses:
    - SPEC-FIX-no-such-document
    - SPEC-FIX-untyped
---

# Dangling

The failing fixture for `relation.target.unresolved`, and it holds both defects that a document target has. The first entry names an identifier that nothing in this tree carries. The second names `04-untyped.md`, which declares that identifier and no kind, so it is a near miss rather than a node.

Each one reports a separate finding, because one instance covers one entry rather than one Q4 pair. A pair needs a target document, and neither of these targets is one.

The relation is `assesses`, which declares no `reciprocal`. So neither entry produces an instance of the reciprocity rule, and an unbound target produces none of the endpoint rule either. This document therefore tests one rule at a time.
