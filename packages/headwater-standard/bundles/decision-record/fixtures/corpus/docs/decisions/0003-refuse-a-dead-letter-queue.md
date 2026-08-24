---
id: HW-DR-0003
title: Refuse a dead-letter queue
status: deprecated
status_since: 2026-02-20
last_verified: 2026-02-20
summary: A proposal for a dead-letter queue that the team considered and refused, kept so that nobody proposes it twice.
provenance:
  warrant: accepted
  agency: human
  accepted_by: fixture-acceptor
  evidence_basis: unevidenced
---

# Refuse a dead-letter queue

## Context

Attempts past the retry ceiling are terminal with the outcome `expired`. A dead-letter queue was proposed as the place to hold them.

## Decision

Beacon ships no dead-letter queue. An expired attempt stays in Postgres with its outcome, and the operator reads it there.

## Consequences

This record was refused rather than accepted, and nothing ever relied on it. The corpus cannot say that. Its state is `deprecated`, which is the same state the abandoned decision beside it carries.
