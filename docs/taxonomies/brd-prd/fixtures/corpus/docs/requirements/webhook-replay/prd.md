---
id: SCR-PRD-webhook-replay
status: current
status_since: 2026-02-01
last_verified: 2026-08-20
summary: What the Beacon replay console must do, and the planted half-written edge to the business document.
requirement_tier: prd
provenance:
  warrant: accepted
  agency: human
  accepted_by: fixture-acceptor
  evidence_basis: unevidenced
relations:
  elaborates:
    - SCR-BRD-webhook-replay
---

# Replay console

## Problem

A tenant who knows they lost events has nowhere to say so. The dashboard lists deliveries and offers no action over them, so the tenant opens a ticket and an engineer runs a script. The script has no dry run, and it has replayed the wrong range twice.

Out of scope: replay across tenants, replay of events Beacon rejected at ingest, and any change to the retention period.

## Solution requirements

1. The replay console takes one destination and a time range, and it shows the count of matching events before it accepts the request. This answers business requirement 1.
2. A replayed delivery carries the header `Beacon-Replay: true` and the identifier of the replay request. This answers business requirement 2.
3. The console refuses a range that starts before the tenant's retention period and states the earliest range it admits. This answers business requirement 3.
4. Replay deliveries are queued behind first deliveries for the same destination, and the queue depth of first deliveries is unaffected. This answers business requirement 4.
5. Each replay request records the requesting user, the destination, the range and the count, and the record is visible on the console. This answers business requirement 5.
6. A replay request is cancellable while it runs, and cancellation stops the queue with no partial state.

## Release criteria

The console replays a seeded range to a live destination and delivers exactly the events in the range, asserted by an acceptance test.

The count shown before the request matches the count delivered, asserted over three seeded ranges of different sizes.

First-delivery latency at the 99th percentile stays inside 800 milliseconds while a replay of 100,000 events runs, measured in the load environment.

A range outside retention is refused with the earliest admissible range in the message, asserted by a contract test.

Cancellation stops a running replay inside five seconds, asserted by an integration test that cancels mid-queue.
