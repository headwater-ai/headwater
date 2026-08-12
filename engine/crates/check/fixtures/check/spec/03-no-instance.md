---
id: SPEC-FIX-no-instance
doc_type: note
---

# No instance

Classified as a `note`, which forbids every facet the two Shape rules read, on a
heterogeneous shelf, and named by no edge. So no rule generates an instance over
it at all.

That is the OB-COV-2 finding: a classified document that no rule looked at. It
is also why the enum rule generates per kind rather than per document — a rule
that instantiated over every typed document would make this finding unreachable.
