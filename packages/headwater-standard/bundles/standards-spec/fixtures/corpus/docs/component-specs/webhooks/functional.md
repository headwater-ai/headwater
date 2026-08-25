---
id: SCR-FS-webhooks
status: current
status_since: 2026-01-15
last_verified: 2026-08-20
summary: What the Beacon webhook registry does, and the planted absence of any technical specification for it.
spec_layer: functional_spec
provenance:
  warrant: accepted
  agency: human
  accepted_by: fixture-acceptor
  evidence_basis: unevidenced
---

# Webhook registry

## Scope

The component that holds the destinations a tenant registered, and the rules that decide which events reach which destination.

## Behavior

A tenant registers a destination with a URL, a secret and an event filter. The registry returns an identifier, and the delivery worker reads the registry to decide where an accepted event goes.

A destination that fails every delivery for seven days is disabled, and the tenant is told. A disabled destination stays in the registry so that a tenant can see why it stopped.

The registry does not validate that a URL answers. A destination that never answered and a destination that stopped answering look the same to a tenant until the first delivery.

## Acceptance

The page limit on the list endpoint is asserted by a contract test that registers 500 destinations and reads one page.

This document is a planted defect of this corpus. It is `current`, its `status_since` is more than 90 days before the recorded run, and no technical specification realizes it. Neither half of the `realizes` pair exists anywhere in the corpus, so the `functional-realized` expectation has nothing to satisfy it.
