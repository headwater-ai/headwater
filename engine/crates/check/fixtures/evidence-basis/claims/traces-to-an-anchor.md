---
id: NOTE-FIX-traces-to-an-anchor
status: current
status_since: 2026-08-02
summary: an evidenced claim whose pointer reaches source code rather than a document
provenance:
  warrant: accepted
  agency: human
  accepted_by: a.person
  evidence_basis: evidenced
relations:
  traces_to:
    - evidence-basis.taxonomy.yml
---

# The evidenced claim tracing to an anchor

An anchor is not a document and it carries no warrant. `EdgeUnit::Pair` never
groups a half whose far end is not a document, so this reaches no instance.
Nothing else in the suite asserts that, and a rule at `EdgeUnit::Entry` would
read an end that is not there.
