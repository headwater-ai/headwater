---
id: SCR-STD-api-design
status: current
status_since: 2026-03-01
last_verified: 2026-08-20
summary: The rules every Beacon component that exposes an HTTP interface answers to.
provenance:
  warrant: accepted
  agency: human
  accepted_by: fixture-acceptor
  evidence_basis: unevidenced
relations:
  regulates:
    - SCR-FS-delivery
    - SCR-FS-signing
---

# API design standard

## Scope

Every Beacon component that exposes an HTTP interface to a caller outside its own process. The internal queue protocol between the delivery worker and the signer is out of scope, and the technical specification of each component states its own wire format.

## Requirements

### 1. Every error response carries a stable code

A caller reads a code and never a message. A message is written for a person and it changes, and a caller that branches on a message breaks when somebody improves the wording.

### 2. Every collection response is paginated

A collection with no page limit is a collection whose response size is decided by the caller's data rather than by this component.

### 3. Every request carries an idempotency key

A caller that retries a request has to be able to retry it safely, and a component that cannot tell a retry from a second request makes the caller decide between a lost write and a duplicate one.

## Conformance

A component conforms when its functional specification names each requirement above under its `Acceptance` heading and states the test that establishes it. The review that admits a new component reads that heading against this document.

A deviation is registered with the platform group, with a named owner and a date it expires. Nothing in this corpus holds that register, because no relation in this taxonomy carries a deviation.
