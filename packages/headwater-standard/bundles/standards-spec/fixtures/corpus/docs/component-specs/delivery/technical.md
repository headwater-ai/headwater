---
id: SCR-TS-delivery
status: current
status_since: 2026-04-01
last_verified: 2026-08-20
summary: How the Beacon delivery worker is built, and where its state lives.
spec_layer: technical_spec
provenance:
  warrant: accepted
  agency: human
  accepted_by: fixture-acceptor
  evidence_basis: unevidenced
relations:
  realizes:
    - SCR-FS-delivery
---

# Delivery worker realization

## Scope

The realization of `SCR-FS-delivery`. That document is canonical for what the worker does, and this one is canonical for how it does it.

## Design

The worker is a pool of processes that read one queue per tenant. Each process takes one attempt, holds a lease for the length of the request timeout, and returns the attempt to the queue when the lease expires.

Attempt state lives in Redis for the length of the retry window and in Postgres after it closes. Redis holds the hot path because the worker reads the attempt count on every retry, and Postgres holds the record because an operator reads it a week later.

The retry schedule is an exponential backoff with full jitter, capped at one hour, over a window of 24 hours. Full jitter was taken over equal jitter because a destination that fails for every tenant at once is the case that matters, and full jitter spreads that herd widest.

## Conformance

Requirement 3 of `SCR-STD-api-design` reaches this document through the idempotency key, which the worker stores as the primary key of the attempt row. The integration test asserts that a replayed accept produces one row.

Requirement 3 of `SCR-STD-logging-baseline` reaches this document through the log writer, which takes a field allowlist and drops every field outside it. The unit test asserts that a payload field never reaches a line.
