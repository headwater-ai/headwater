---
id: DR-repo-0005
title: Partition attempt storage by tenant
status: draft
status_since: 2026-01-05
last_verified: 2026-01-05
summary: A proposal to partition the attempts table by tenant, written in January and never ruled on.
provenance:
  warrant: accepted
  agency: agent
  accepted_by: fixture-acceptor
  evidence_basis: unevidenced
---

# Partition attempt storage by tenant

## Context

One tenant's retry storm slows every other tenant's reads, because all attempts share one table.

## Decision

Beacon partitions the attempts table by tenant.

## Consequences

Cross-tenant queries need a fan-out. Nobody has accepted or refused this, and it has stood in `draft` since January.
