---
id: HW-DR-0001
title: Deliver at least once, and expose the attempt identifier
status: superseded
status_since: 2026-03-14
last_verified: 2026-03-14
summary: The first delivery guarantee Beacon shipped, replaced when per-destination deduplication became cheap.
provenance:
  warrant: accepted
  agency: mixed
  accepted_by: fixture-acceptor
  evidence_basis: unevidenced
relations:
  superseded_by:
    - HW-DR-0004
---

# Deliver at least once, and expose the attempt identifier

## Context

Beacon delivers an attempt to a destination over a network that drops messages. A receiver that sees an attempt twice can tell the two apart only if something on the attempt is stable across retries.

## Decision

Beacon delivers at least once per destination. Every attempt carries an attempt identifier, and a retry of one attempt carries the same one.

## Consequences

A receiver holds the deduplication. That is work Beacon pushes onto every integrator, and the integration guide has to say so on its first page.
