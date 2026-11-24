---
id: SCR-AC-0001
status: current
status_since: 2026-06-01
summary: "A replay that an operator starts reaches a healthy endpoint within sixty seconds, measured from the click to the first byte the endpoint receives."
last_verified: 2026-06-01
title: "A replay reaches the endpoint within one minute"
verification_method: test
provenance:
  warrant: accepted
  accepted_by: fixture-acceptor
relations:
  verifies:
    - SCR-REQ-0001
  proven_by:
    - SCR-VER-0001
---

# A replay reaches the endpoint within one minute

## Fit criterion

Over 20 replays to a healthy endpoint, the slowest replay arrives within 60 seconds of the request.

## Method

A test starts a local endpoint that fails the first five attempts and then answers 200. It then requests a replay through the console API and measures the time to the first byte that the endpoint receives.
