---
id: SCR-TS-signing
status: current
status_since: 2026-01-15
last_verified: 2026-08-20
summary: How the Beacon signer is built, and the planted half of a reciprocal edge.
spec_layer: technical_spec
provenance:
  warrant: accepted
  agency: human
  accepted_by: fixture-acceptor
  evidence_basis: unevidenced
relations:
  realizes:
    - SCR-FS-signing
---

# Signer realization

## Scope

The realization of `SCR-FS-signing`. This document declares the `realizes` edge and the functional specification does not name it back, which is the planted defect.

## Design

The signer runs in the delivery worker process rather than as a service of its own. A round trip per payload would double the delivery latency, and the key material is already in the worker's address space for the length of one attempt.

Signatures are Ed25519 over the raw payload bytes. The key identifier is the first eight bytes of the public key, hex encoded, which is short enough for a header and wide enough that a collision across one tenant's keys is not a case anybody plans for.

The rotation window is held as two active keys per tenant. The signer signs with the newer one and the verifier accepts both, and the older one is dropped 30 days after the rotation.

## Conformance

Requirement 1 of `SCR-STD-api-design` reaches this document through the error mapping table, which turns each signer failure into one code. The unit test walks the table.

The 30-day rotation window is asserted by an integration test that rotates a key, verifies with the old public key, advances the clock past the window and asserts the failure.
