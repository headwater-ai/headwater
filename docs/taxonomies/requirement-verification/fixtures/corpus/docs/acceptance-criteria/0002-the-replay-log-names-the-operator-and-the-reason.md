---
id: SCR-AC-0002
status: current
status_since: 2026-06-01
summary: "Each replay writes one audit line that names the operator, the delivery and the reason that the operator typed."
last_verified: 2026-06-01
title: "The replay log names the operator and the reason"
verification_method: inspection
provenance:
  warrant: accepted
  accepted_by: fixture-acceptor
relations:
  verifies:
    - SCR-REQ-0001
---

# The replay log names the operator and the reason

## Fit criterion

The audit log holds one line for each replay, and the line names the operator, the delivery identifier and the reason text.

## Method

A reviewer reads the audit log after three replays in the staging console and compares each line with the replay that caused it.
