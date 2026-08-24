---
status: current
status_since: 2026-08-01
last_verified: 2026-08-20
summary: What Beacon is, what it delivers, and the fields an integrator meets first.
provenance:
  warrant: accepted
  agency: mixed
  accepted_by: fixture-acceptor
  evidence_basis: unevidenced
---

# Beacon, in one page

## Scope

The page an integrator opens first. It argues for the model and it lists the fields of the model in the same file, so it holds two reader modes and it carries no `reader_mode`.

## Behavior

Beacon is a delivery service for outbound webhooks. It exists because a retry loop written by hand inside an application shares the fate of that application, and a delivery that has to survive a deploy cannot live there.

The model has three nodes. A **message** is content plus destinations. An **attempt** is one delivery of one message to one destination. An **outcome** is terminal.

| Node | Identifier | Carries |
|---|---|---|
| message | `msg_…` | body, destinations, idempotency key |
| attempt | `att_…` | message, destination, outcome, timestamp |
| destination | `dst_…` | URL, signing key set |

The first two paragraphs are explanation and the table is reference. Splitting them is the remedy the doctrine states, and this fixture is here to be the document nobody has split yet.
