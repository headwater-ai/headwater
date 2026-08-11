---
status: current
status_since: 2026-02-03
last_verified: 2026-07-01
summary: Why Beacon exists, who it serves, and what it refuses to do.
doc_type: design_spec
sequence: 0
provenance:
  warrant: accepted
  agency: human
  accepted_by: a.ferreira
  evidence_basis: evidenced
---

# 0 — Motivation and scope

Notifications in the estate are sent by nine services, each with its own retry rule. An operator who asks "did the customer get it?" reads nine logs. Beacon is one delivery path with one record of what happened.

## Scope

Beacon accepts a message and a destination, and it reports the outcome of each attempt.

## Out of scope

Beacon does not compose message content, and it does not hold a customer profile.
