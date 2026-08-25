---
id: SCR-FS-ingest
status: current
status_since: 2026-08-01
last_verified: 2026-08-20
summary: What the Beacon ingest endpoint does, and the planted discriminator value outside the admitted set.
spec_layer: interface_spec
provenance:
  warrant: accepted
  agency: human
  accepted_by: fixture-acceptor
  evidence_basis: unevidenced
---

# Ingest endpoint

## Scope

The component that accepts an event from a tenant and puts it on the delivery queue. Delivery itself is `SCR-FS-delivery`.

## Behavior

The endpoint accepts an event, assigns it an identifier and a sequence number, and returns both. An event that the endpoint accepted is delivered at least once to every destination the registry matches.

The endpoint refuses an event larger than one megabyte, and it refuses an event whose tenant has no destination registered.

The endpoint is idempotent over the idempotency key a caller presents. A repeated key returns the identifier of the first event and queues nothing.

## Acceptance

The idempotency behavior is asserted by a contract test that presents one key twice and reads one queued event.

This document is the planted defect of this corpus, and it is the one that produces no finding. Its `spec_layer` says `interface_spec`, the `component_specs` shelf is heterogeneous and admits `functional_spec` and `technical_spec`, so kind resolution stops and the document is untyped. An untyped document is a row of the census and no rule reads it.
