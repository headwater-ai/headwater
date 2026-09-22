---
id: SPEC-FIX-orphan
---

# Orphan

No shelf of this fixture tree claims `query/notes/**`, so this document is
untyped: it carries an identifier and no kind. It exists for one read alone —
`resolve_identifier("SPEC-FIX-orphan")` — which is the near-miss case
`query.reads` records: a typed lookup finds nothing, and the untyped index
finds this file by the same identifier, exactly.
