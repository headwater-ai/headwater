---
id: SCR-REQ-0001
status: current
status_since: 2026-06-01
summary: "An operator replays one failed webhook delivery from the console, and Beacon sends the stored payload again to the same endpoint."
last_verified: 2026-06-01
title: "An operator can replay a failed delivery"
provenance:
  warrant: accepted
  accepted_by: fixture-acceptor
relations:
  verified_by:
    - SCR-AC-0001
    - SCR-AC-0002
    - SCR-AC-0003
---

# An operator can replay a failed delivery

## Context

Beacon delivers webhooks to customer endpoints. A delivery fails when the endpoint is down or answers with a server error, and Beacon stops after five attempts. Support then asks the customer to wait for the next event, and an order that the customer never received stays lost. Support raised this in ticket BCN-412.

## Requirement

When a delivery has failed, the console shall let an operator replay it. The replay shall send the stored payload, with its original event identifier, to the endpoint that the delivery named.

The replay will reuse the signing key of the original delivery, so that the customer verifies it with the code that it already runs.

## Verification

Three acceptance criteria settle this requirement. `SCR-AC-0001` measures how fast a replay arrives, and a test proves it. `SCR-AC-0002` states what the replay log records, and a reviewer inspects it. `SCR-AC-0003` states that a replay keeps the event identifier, and a test is to prove it.
