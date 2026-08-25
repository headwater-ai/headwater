---
id: BCN-DR-0001
title: Sign every payload, and publish the key set
status: current
status_since: 2026-08-01
last_verified: 2026-08-20
summary: Beacon signs every outbound payload and publishes the verifying key set, so a receiver can refuse a forgery.
provenance:
  warrant: accepted
  agency: mixed
  accepted_by: fixture-acceptor
  evidence_basis: unevidenced
---

# Sign every payload, and publish the key set

## Context

A receiver reads a payload from a public URL, and any sender can reach that URL. Without a signature the receiver cannot tell a Beacon delivery from a forgery.

## Decision

Beacon signs every payload with the newest key of a set, and it publishes the whole set. A receiver verifies against the set.

## Consequences

Every receiver has to verify, and a receiver that skips the check gains nothing from the signature. The key set has to be reachable without authentication, which is what makes a rotation a two-step operation rather than a swap.

This document carries no `reader_mode`. It is one of the kinds the theoretical-foundations evaluation names as distorting under the four modes: it is a record of a choice and its authority, and no reader arrives at it in a reader mode.
