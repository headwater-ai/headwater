---
status: current
status_since: 2026-02-14
last_verified: 2026-07-01
summary: "The core model: a message, an attempt, and a terminal outcome per destination."
doc_type: design_spec
sequence: 1
provenance:
  warrant: accepted
  agency: mixed
  accepted_by: fixture-acceptor
  evidence_basis: evidenced
---

# 1 — The core model

A **message** is the content and its destinations. An **attempt** is one delivery of one message to one destination. An **outcome** is terminal: delivered, refused, or expired.

## Why the attempt is a node

An operator asks how many times a delivery was tried before it succeeded. A counter cannot answer that, and an attempt with a timestamp can.

## Limits

This part says nothing about ordering between destinations. Spec 4 does.
