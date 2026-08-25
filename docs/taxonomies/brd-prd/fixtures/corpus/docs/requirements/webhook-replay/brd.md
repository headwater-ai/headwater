---
id: SCR-BRD-webhook-replay
status: current
status_since: 2026-01-15
last_verified: 2026-08-20
summary: Why a Beacon tenant replays events to a destination, and the planted change narration in its prose.
requirement_tier: brd
provenance:
  warrant: accepted
  agency: human
  accepted_by: fixture-acceptor
  evidence_basis: unevidenced
---

# Tenant-driven replay

## Business need

A Beacon destination that is down for an hour loses every event whose retry window closed. The tenant discovers the gap days later, from their own reconciliation, and the only remedy is a support ticket that asks Beacon to replay from the attempt table by hand.

The cost is support time and trust in equal parts. Each replay takes an engineer 90 minutes, and the tenant learns that Beacon holds their events and will not give them back without asking a person.

We changed the retry window from six hours to 24 hours last year, and the gap is smaller than it was. It is not closed, because a destination down for two days is a destination that loses two days.

The boundary of this initiative is replay of events Beacon already accepted. Recovery of events a tenant never sent belongs to the ingest roadmap.

## Business requirements

1. A tenant replays the events for one destination over a time range they choose, with no Beacon employee involved.
2. A replayed event is distinguishable from a first delivery, so that a tenant's own idempotency logic can act on it.
3. Replay is bounded by the retention Beacon already sells, and a request outside that window is refused with the reason.
4. A replay in progress does not delay first delivery for any tenant, including the tenant who asked for it.
5. Beacon records who asked for each replay, for which destination and over which range.

## Success measures

Support tickets tagged `replay` run at 12 per month today, at 90 minutes each. Success is under 1 per month.

The median time from a tenant noticing a gap to the events arriving is 3 working days today. Success is under 10 minutes.

The share of replays that a tenant completes alone is 0% today. Success is above 95%.

First-delivery latency during a replay is the measure of the fourth requirement. The 99th percentile is 800 milliseconds today with no replay running, and success is the same figure while a replay of 100,000 events runs.
