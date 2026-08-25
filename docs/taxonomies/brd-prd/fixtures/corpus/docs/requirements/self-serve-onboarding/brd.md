---
id: SCR-BRD-self-serve-onboarding
status: current
status_since: 2026-01-15
last_verified: 2026-08-20
summary: Why Beacon takes a tenant from signup to a first delivered event with no sales call.
requirement_tier: brd
provenance:
  warrant: accepted
  agency: human
  accepted_by: fixture-acceptor
  evidence_basis: unevidenced
relations:
  elaborated_by:
    - SCR-PRD-self-serve-onboarding
---

# Self-serve onboarding

## Business need

Every new Beacon tenant reaches a first delivered event through a person. An account manager creates the tenant, a support engineer registers the first destination, and a second engineer sends a test event. Three people are between a signup and a working integration.

The cost is the reason this initiative exists. The three steps take four working days on average, and a tenant who waits four days has already tried a competitor. The team that runs the steps is the same team that answers support, so growth in signups is growth in support load rather than growth in revenue.

Signup volume will be four times the current one once the paid marketing budget lands, and the three-person path does not carry that.

The boundary of this initiative is signup to first delivered event. Billing, quota and the destination registry beyond the first destination belong to other initiatives.

## Business requirements

1. A tenant registers, configures a destination and receives a delivered event with no Beacon employee involved.
2. A tenant who abandons the flow can resume it from where they stopped, on any device.
3. Beacon records which step each incomplete signup stopped at, so that the team can see where the flow loses people.
4. The flow refuses a destination that Beacon cannot reach, at the moment the tenant enters it, rather than at the first delivery.
5. A tenant on the self-serve path receives the same delivery guarantee as a tenant an account manager configured.

## Success measures

The median time from signup to first delivered event is four working days today. Success is under 15 minutes.

The share of new tenants who reach a first delivered event with no support ticket is 0% today. Success is above 80%.

Support tickets tagged `onboarding` run at 40 per month today. Success is under 10 per month at twice the signup volume.

The measure of the fourth requirement is the share of first deliveries that fail because the destination was wrong. That share is 22% today and success is under 5%.
