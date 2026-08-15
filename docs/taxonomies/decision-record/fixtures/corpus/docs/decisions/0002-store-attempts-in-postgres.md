---
id: HW-DR-0002
title: Store attempts in Postgres rather than in the queue
status: current
status_since: 2026-04-02
last_verified: 2026-07-01
summary: Attempt state lives in a relational store, so an operator can read it without draining a queue.
provenance:
  warrant: accepted
  agency: human
  accepted_by: fixture-acceptor
  evidence_basis: evidenced
relations:
  traces_to:
    - EVAL-repo-retry-ceiling
  conflicts_with:
    - HW-DR-0006
---

# Store attempts in Postgres rather than in the queue

## Context

An operator who asks "what happened to attempt 4812?" has to read attempt state. Queue state is readable only by draining the queue, which destroys it.

## Decision

Beacon holds attempt state in Postgres. The queue carries a reference and nothing else.

## Consequences

Every attempt costs a write to a second store, and the two can disagree. The retry-ceiling measurement bounds how often they do.
