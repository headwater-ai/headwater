---
status: current
status_since: 2026-06-20
last_verified: 2026-07-01
summary: How an attempt reaches a destination, and what the delivery guarantee is.
doc_type: design_spec
sequence: 4
supersedes:
  - docs/spec/02-transport.md
provenance:
  warrant: accepted
  agency: mixed
  accepted_by: a.ferreira
  evidence_basis: evidenced
---

# 4 — Transport

## Delivery guarantee

Beacon delivers at least once per destination, and a duplicate is visible to the receiver through the attempt identifier.

## Retry

An attempt that fails is retried on a doubling interval, to a declared ceiling. An attempt past the ceiling is terminal with the outcome `expired`.
