---
status: current
status_since: 2026-08-01
last_verified: 2026-08-20
summary: Why Beacon delivers at least once, and what that pushes onto every receiver.
reader_mode: explanation
provenance:
  warrant: accepted
  agency: mixed
  accepted_by: fixture-acceptor
  evidence_basis: unevidenced
---

# Why delivery is at least once

## Scope

The reasoning behind the delivery guarantee. It teaches no task and it lists no field.

## Behavior

A network drops messages, and a sender cannot tell a lost delivery from a lost acknowledgment. A sender that retries in that state delivers twice. A sender that does not retry loses the message.

Beacon retries, so a receiver sees a duplicate. The alternative is exactly-once delivery, which needs agreement between the sender and the receiver on every message, and that agreement costs a round trip on the path that carries every payload.

So the cost sits with the receiver, which holds the attempt identifier and discards a repeat. That is one lookup on a value Beacon already sends, and it is the reason the attempt identifier is stable across retries.
