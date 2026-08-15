---
id: HW-DR-0004
title: Deduplicate per destination, so delivery is exactly once
status: current
status_since: 2026-03-14
last_verified: 2026-07-01
summary: Beacon holds a per-destination window of attempt identifiers, so a receiver no longer carries the deduplication.
provenance:
  warrant: accepted
  agency: mixed
  accepted_by: fixture-acceptor
  evidence_basis: unevidenced
relations:
  supersedes:
    - HW-DR-0001
---

# Deduplicate per destination, so delivery is exactly once

## Context

The at-least-once guarantee pushed deduplication onto every integrator. A per-destination window of attempt identifiers became affordable when attempt state moved to Postgres.

## Decision

Beacon deduplicates per destination over a window of one hour. Delivery is exactly once inside that window and at least once outside it.

## Consequences

The guarantee is now bounded by a window, which is a weaker claim than the one the marketing page made. The window length is an operational parameter, and every change to it changes the guarantee.
