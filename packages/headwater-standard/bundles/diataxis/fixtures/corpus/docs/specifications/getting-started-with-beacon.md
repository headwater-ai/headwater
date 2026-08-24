---
status: current
status_since: 2026-08-01
last_verified: 2026-08-20
summary: The first hour with Beacon, from an empty account to one delivered attempt.
reader_mode: tutorial
provenance:
  warrant: accepted
  agency: mixed
  accepted_by: fixture-acceptor
  evidence_basis: unevidenced
---

# Getting started with Beacon

## Scope

A reader who has never seen Beacon, and who wants one message delivered to one destination. It assumes an account and nothing else.

## Behavior

Create a destination, send one message to it, and read the attempt back.

1. Register the destination `https://example.test/hooks/first`. Beacon answers with a destination identifier.
2. Send one message to that identifier with the body `{"hello": "beacon"}`.
3. Read the attempt. It shows one delivery, its outcome, and the attempt identifier.

You should now see one destination, one message and one attempt, and the attempt outcome is `delivered`.
