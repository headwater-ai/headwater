---
id: HW-OBL-0001
title: The deduplication window length is unmeasured
status: current
status_since: 2026-03-20
last_verified: 2026-07-01
summary: One hour was chosen as the deduplication window, and no measurement supports it.
provenance:
  warrant: accepted
  agency: mixed
  accepted_by: fixture-acceptor
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0004
  discharged_by:
    - EVAL-repo-retry-ceiling
---

# The deduplication window length is unmeasured

## Context

The exactly-once decision fixes the window at one hour. The number came from a meeting.

## Obligation

Beacon owes a measurement of how far apart two attempts of one delivery fall in production, against the window that the guarantee rests on.

## Discharge

The retry-ceiling measurement is the instrument, and it has run. The edge to it is what closes this record.
