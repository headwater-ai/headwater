---
id: NOTE-FIX-onto-blank
status: current
status_since: 2026-09-30
summary: an evidenced claim onto a record whose warrant key has no value
provenance:
  warrant: accepted
  agency: human
  accepted_by: a.person
  evidence_basis: evidenced
relations:
  traces_to:
    - NOTE-FIX-blank
---

# The evidenced claim onto a blank warrant

The parser hands `warrant:` back as the text `~`. A reader that took the text would report a value nobody wrote, so this instance skips.
