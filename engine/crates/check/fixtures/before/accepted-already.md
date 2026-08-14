---
id: NOTE-FIX-promoted
status: current
status_since: 2026-08-01
summary: a document a person read and accepted, which is the transition this rule counts
provenance:
  warrant: accepted
  agency: human
  accepted_by: a.person
---

# The promoted record

This is the same document at a version where the warrant already stood at
`accepted`. A run over this prior version counts no promotion, and it is what
tells a cache key that reads the prior version from one that does not.
