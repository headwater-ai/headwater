---
id: SCR-PRD-self-serve-onboarding
status: current
status_since: 2026-02-01
last_verified: 2026-08-20
summary: What the Beacon signup flow must do so that a tenant reaches a first delivered event alone.
requirement_tier: prd
provenance:
  warrant: accepted
  agency: human
  accepted_by: fixture-acceptor
  evidence_basis: unevidenced
relations:
  elaborates:
    - SCR-BRD-self-serve-onboarding
---

# Guided signup

## Problem

A tenant who signs up for Beacon lands on an empty dashboard. Nothing on it says what to do next, and the destination form asks for a signing secret that the tenant does not have yet. The tenant opens a support ticket, and the ticket is the onboarding flow.

Out of scope: the billing plan chooser, the team-invite flow and the destination registry beyond the first destination. A tenant who wants a second destination uses the registry that already exists.

## Solution requirements

1. The dashboard shows a three-step checklist to a tenant with no delivered event: register a destination, send a test event, confirm receipt. This answers business requirement 1.
2. Each step records its completion against the tenant, and the checklist resumes at the first incomplete step on any session. This answers business requirement 2.
3. The destination form generates the signing secret and shows it once, so that the tenant is never asked for a value Beacon owns. This answers business requirement 1.
4. The form sends a reachability probe to the destination before it accepts the value, and it refuses a destination that answers outside the 2xx range. This answers business requirement 4.
5. Every step transition emits an analytics event carrying the tenant, the step and the outcome. This answers business requirement 3.
6. A test event travels the same delivery path as a production event, so that the guarantee a tenant sees in onboarding is the guarantee they get. This answers business requirement 5.

## Release criteria

The checklist renders for a tenant with no delivered event, and the acceptance test drives all three steps against a live destination.

The reachability probe refuses an unreachable destination inside two seconds, measured against a destination that never answers.

The resume path restores a half-finished checklist after a session ends, asserted by an integration test that drops the session between step one and step two.

Five tenants from the design partner list complete the flow with no support contact, observed by the product manager.

The analytics events for all three steps appear in the warehouse, confirmed by a query the data team runs before release.

The copy for the third step is to be decided with the content designer, and it does not block the criteria above.
