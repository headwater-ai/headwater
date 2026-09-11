---
id: NOTE-FIX-placeholder-basis
status: current
status_since: 2026-08-02
summary: a document whose evidence basis is still the template placeholder
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-opus-5
  evidence_basis: "{{evidenced | reconstructed | unevidenced}}"
relations:
  traces_to:
    - NOTE-FIX-asserted
---

# The unfilled template placeholder

Three templates of the base package ship this string, and documents of this
repository carry it unfilled. It is not `evidenced`, so the rule passes it, and
the case is here because a reader that split the string would report it.
