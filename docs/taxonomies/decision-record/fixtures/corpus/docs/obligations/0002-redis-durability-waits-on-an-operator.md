---
id: OBL-repo-0002
title: Redis durability under a node loss is untested
status: current
status_since: 2026-06-01
last_verified: 2026-07-01
summary: Nothing states what Beacon loses when a Redis node goes, and no deployment large enough to find out exists.
provenance:
  warrant: accepted
  agency: human
  accepted_by: fixture-acceptor
  evidence_basis: unevidenced
relations:
  traces_to:
    - DR-repo-0006
---

# Redis durability under a node loss is untested

## Context

Hot attempt state lives in Redis for the length of the retry window. A node loss inside that window drops state that Postgres does not yet hold.

## Obligation

Beacon owes a statement of what a node loss costs, and a test that produces it.

## Discharge

This waits on a deployment with more than one node, and no operator runs one. There is no window in which this becomes overdue, which is why the taxonomy declares no expectation over it.
