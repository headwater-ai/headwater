---
id: SCR-PRD-delivery-slo
status: current
status_since: 2026-02-01
last_verified: 2026-08-20
summary: What the Beacon attainment report must do so that a tenant reads their own delivery objective.
requirement_tier: prd
provenance:
  warrant: accepted
  agency: human
  accepted_by: fixture-acceptor
  evidence_basis: unevidenced
relations:
  elaborates:
    - SCR-BRD-delivery-slo
---

# Attainment report

## Problem

A tenant who asks whether Beacon met its objective has no page to read. The delivery team answers with a query against the attempt table, and the query takes an engineer an hour. Two tenants have asked for the same month and received two different numbers.

Out of scope: the credit calculation and the alerting path. The credit belongs to billing and the alert belongs to the delivery team's own runbook.

## Solution requirements

1. Beacon computes attainment for each tenant as the share of accepted events delivered inside the objective window, over a calendar month. This answers business requirement 2.
2. The attainment page shows the current month to date and the twelve months before it, per tenant. This answers business requirement 2.
3. The page states the published objective beside the attainment, taken from one configuration value that no page hard-codes. This answers business requirement 1.
4. An event whose destination refused delivery is counted as delivered for the purpose of attainment, and the page says so in one line. This answers business requirement 5.
5. The computed monthly attainment is exposed through the reporting API, so that the alerting path and the credit calculation read one number. This answers business requirements 3 and 4.

## Release criteria

The attainment figure for a seeded month matches a hand calculation over the same attempt rows, asserted by an acceptance test.

The page renders for a tenant with no delivered events and states that attainment is undefined rather than 0%.

The reporting API returns the same figure the page shows, asserted by a contract test that reads both.

The objective value on the page changes when the configuration value changes, with no deployment, confirmed by the delivery team.

A tenant from the design partner list reads the page and states the number without asking what it means.
