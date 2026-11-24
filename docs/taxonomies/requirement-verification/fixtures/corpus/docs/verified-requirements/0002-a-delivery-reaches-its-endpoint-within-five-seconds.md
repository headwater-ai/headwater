---
id: SCR-REQ-0002
status: current
status_since: 2026-06-01
summary: "Beacon sends ninety-nine of every hundred first delivery attempts within five seconds of the event that caused them."
last_verified: 2026-06-01
title: "A delivery reaches its endpoint within five seconds"
provenance:
  warrant: accepted
  accepted_by: fixture-acceptor
---

# A delivery reaches its endpoint within five seconds

## Context

Customers use Beacon webhooks to start a fulfillment job. A slow delivery delays the job, and two customers wrote their own polling code because of it.

## Requirement

Beacon shall send the first attempt of 99 of every 100 deliveries within five seconds of the event, measured over one calendar day.

## Verification

No acceptance criterion states the measure yet. The delivery team owns it.
