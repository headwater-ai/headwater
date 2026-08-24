---
id: SCR-FS-signing
status: current
status_since: 2026-01-15
last_verified: 2026-08-20
summary: What the Beacon signer does, and the planted absence of the technical specification that realizes it.
spec_layer: functional_spec
provenance:
  warrant: accepted
  agency: human
  accepted_by: fixture-acceptor
  evidence_basis: unevidenced
relations:
  regulated_by:
    - SCR-STD-api-design
---

# Signer

## Scope

The component that signs an outgoing payload so that a destination can establish that Beacon sent it. Key storage is the platform key service and is out of scope.

## Behavior

The signer takes a payload and a tenant, and it returns a detached signature over the payload bytes and a key identifier. A destination verifies the signature against the public key the identifier names.

The signer signs with the tenant's current key. A key that was rotated stays valid for verification for 30 days, so a destination that caches a key has a window in which to notice the rotation.

The signer does not sign a payload larger than one megabyte. A caller that presents one gets an error code and no signature.

## Acceptance

Requirement 1 of `SCR-STD-api-design` is met by the error table in the signer API reference, and the contract test asserts that the oversize case returns a code from that table.

Requirement 3 of `SCR-STD-api-design` is met by the key identifier, which makes a repeated sign request return the same signature for the same payload and key.

This document is the far end of one planted defect of this corpus. `SCR-TS-signing` declares `realizes: SCR-FS-signing` and this document does not name that edge back, so a run reports the missing half against the technical specification.

No participation finding fires here, and the reason is worth stating. The `functional-realized` expectation is satisfied by the half either document wrote, and `SCR-TS-signing` wrote one. An old `status_since` therefore costs this document nothing. `docs/component-specs/webhooks/functional.md` is where that finding lands, because neither half of a `realizes` pair exists anywhere for it.
