---
id: NOTE-FIX-blank
status: current
status_since: 2026-09-30
summary: a record that writes the warrant key and no value
provenance:
  warrant:
  agency: human
---

# The record with a blank warrant

`warrant:` with nothing after it is null in the core schema, which is an absence and not a fifth value. The instance onto it skips as it does onto a record with no warrant.
