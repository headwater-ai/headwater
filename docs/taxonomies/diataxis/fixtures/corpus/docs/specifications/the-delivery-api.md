---
status: current
status_since: 2026-08-01
last_verified: 2026-08-20
summary: Every field of the Beacon delivery API, with its type and its admitted values.
reader_mode: reference
provenance:
  warrant: accepted
  agency: mixed
  accepted_by: fixture-acceptor
  evidence_basis: unevidenced
---

# The delivery API

## Scope

Every field the delivery API reads and writes. It states what each field is and never why the field is there.

## Behavior

A message carries three fields.

| Field | Type | Admitted values |
|---|---|---|
| `destination` | string | a destination identifier |
| `body` | object | any JSON object under 256 KiB |
| `idempotency_key` | string | 1 to 128 characters, or absent |

An attempt carries four.

| Field | Type | Admitted values |
|---|---|---|
| `attempt` | string | an attempt identifier |
| `message` | string | the message identifier |
| `outcome` | string | `delivered`, `refused`, `expired` |
| `at` | timestamp | RFC 3339, in UTC |
