---
id: SCR-FS-delivery
status: current
status_since: 2026-04-01
last_verified: 2026-08-20
summary: What the Beacon delivery worker does, and what a caller may rely on.
spec_layer: functional_spec
provenance:
  warrant: accepted
  agency: human
  accepted_by: fixture-acceptor
  evidence_basis: unevidenced
relations:
  regulated_by:
    - SCR-STD-api-design
    - SCR-STD-logging-baseline
  realized_by:
    - SCR-TS-delivery
---

# Delivery worker

## Scope

The component that takes an accepted event and delivers it to one destination. Acceptance of the event is the ingest component, and the signature on the payload is the signer.

## Behavior

Beacon delivers an accepted event to each destination at least once. A destination that answers with a status outside the 2xx range is retried, and the worker stops after the retry window closes.

The worker delivers to one destination independently of every other destination. A destination that fails delays nothing for a second destination of the same event.

The worker does not deliver in order. Two events accepted a millisecond apart may arrive at a destination in either order, and a caller that needs an order reads the sequence number in the payload.

## Acceptance

Requirement 1 of `SCR-STD-api-design` is met by the error table in the delivery API reference, and the contract test asserts that every branch returns a code from that table.

Requirement 3 of `SCR-STD-api-design` is met by the idempotency key on the accept endpoint, and the contract test replays one request and asserts one delivery.

Requirement 2 of `SCR-STD-logging-baseline` is met by the tenant and attempt fields on every line, and the log schema test asserts both.
