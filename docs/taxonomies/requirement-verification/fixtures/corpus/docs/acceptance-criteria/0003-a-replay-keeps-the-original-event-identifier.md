---
id: SCR-AC-0003
status: current
status_since: 2026-06-01
summary: "The replayed request carries the event identifier of the failed delivery, so a customer that removes duplicates by that identifier sees one event."
last_verified: 2026-06-01
title: "A replay keeps the original event identifier"
verification_method: test
provenance:
  warrant: accepted
  accepted_by: fixture-acceptor
relations:
  verifies:
    - SCR-REQ-0001
---

# A replay keeps the original event identifier

## Fit criterion

The `Beacon-Event-Id` header of the replayed request is equal to the header of the failed attempt.

## Method

A test fails one delivery, replays it, and compares the two headers that the local endpoint recorded.
