---
id: DR-repo-0006
title: Hold hot attempt state in Redis
status: current
status_since: 2026-05-30
last_verified: 2026-07-01
summary: Hot attempt state lives in Redis for the length of the retry window, which contradicts the Postgres decision beside it.
provenance:
  warrant: accepted
  agency: human
  accepted_by: fixture-acceptor
  evidence_basis: unevidenced
relations:
  conflicts_with:
    - DR-repo-0002
---

# Hold hot attempt state in Redis

## Context

Reads of attempt state during a retry storm are hot, and Postgres carries them badly.

## Decision

Beacon holds attempt state in Redis for the length of the retry window, and writes it to Postgres when the window closes.

## Consequences

Two stores hold the same state, and an operator reading during the window reads Redis. This record and `DR-repo-0002` are both current and they contradict each other, which is what the planted conflict is here to report.
