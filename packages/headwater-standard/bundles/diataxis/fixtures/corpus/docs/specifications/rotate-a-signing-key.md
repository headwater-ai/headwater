---
status: current
status_since: 2026-08-01
last_verified: 2026-08-20
summary: How to replace a Beacon signing key without a gap in which a receiver refuses a payload.
reader_mode: how-to
provenance:
  warrant: accepted
  agency: mixed
  accepted_by: fixture-acceptor
  evidence_basis: unevidenced
---

# Rotate a signing key

## Scope

An operator who already runs Beacon and who has to replace a signing key. It assumes a receiver that verifies the signature on every payload.

## Behavior

Beacon signs with one key and verifies against a set, so a rotation is an addition followed by a removal.

1. Add the new key. Beacon signs with the newest key and publishes both.
2. Wait for the receiver to read the published set. A receiver holds the set for one hour.
3. Remove the old key. A payload signed with it is refused from that moment.

A removal before step 2 leaves a window in which the receiver holds no key that verifies the signature it is sent.
