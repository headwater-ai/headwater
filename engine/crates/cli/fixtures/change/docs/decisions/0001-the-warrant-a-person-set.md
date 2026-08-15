---
id: HW-DR-0001
title: The warrant a person set
status: current
status_since: 2026-08-01
last_verified: 2026-08-01
summary: A document that stands at the accepted warrant, so that a prior version at asserted makes one promotion.
provenance:
  warrant: accepted
  agency: human
  accepted_by: a.person
  evidence_basis: unevidenced
---

# The warrant a person set

## Context

This document exists so that `warrant.promoted` has one instance to report. The prior version beside it stands at `asserted`, and the manifest of the change names this path and that file.

## Decision

The warrant is `accepted` here and `asserted` there, which is the transition the rule counts.

## Consequences

A run that carries the change reports one promotion. A run that carries none reports the instance as skipped, and the two reports are what the test compares.
