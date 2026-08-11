---
status: current
status_since: 2026-01-28
last_verified: 2026-07-01
summary: Recovery after a forced restart on both broker candidates, and what each lost.
cited_by:
  - docs/spec/09-decisions.md
provenance:
  warrant: accepted
  agency: mixed
  accepted_by: fixture-acceptor
  evidence_basis: evidenced
---

# Queue durability

## The question

Q1 asked which broker carries the queue. The two candidates differ in what a restart costs.

## The instrument

Ten thousand messages were enqueued against each candidate, the process was killed at a random point, and the count that survived the restart was compared with the count acknowledged.

## What it found

The in-memory broker lost every message that was not yet consumed, which was 41 percent of the run at the point of the kill. The durable log lost none, and it took 90 seconds longer to drain the backlog afterwards.

## The decision it supports

The durable log. The drain time is a latency cost that the design can absorb, and a lost attempt is a record that no operator can reconstruct.

## What it did not settle

The retention window, which is a cost question rather than a durability one.
