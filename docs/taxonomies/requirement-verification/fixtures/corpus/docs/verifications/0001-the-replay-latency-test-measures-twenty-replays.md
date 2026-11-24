---
id: SCR-VER-0001
status: current
status_since: 2026-06-01
summary: "The replay latency test runs twenty replays against a local endpoint and fails when the slowest one takes longer than sixty seconds."
last_verified: 2026-06-01
title: "The replay latency test measures twenty replays"
provenance:
  warrant: accepted
  accepted_by: fixture-acceptor
relations:
  proves:
    - SCR-AC-0001
  cited_in:
    - src/replay_latency_test.rs
---

# The replay latency test measures twenty replays

## Approach

`src/replay_latency_test.rs` starts a local endpoint, fails five attempts of one delivery, and requests a replay through the console API. It records the time from the request to the first byte that the endpoint receives. It does this 20 times and asserts that the largest time is below 60 seconds. A comment in the test cites this document by its identifier.

## What this does not cover

The endpoint is local, so the test measures Beacon and not the network to a customer. It does not cover a replay to an endpoint that is still down.
