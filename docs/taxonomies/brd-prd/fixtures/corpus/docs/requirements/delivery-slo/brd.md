---
id: SCR-BRD-delivery-slo
status: current
status_since: 2026-01-15
last_verified: 2026-08-20
summary: Why Beacon publishes a delivery service level objective, and the planted absence of the heading that carries the measures.
requirement_tier: brd
provenance:
  warrant: accepted
  agency: human
  accepted_by: fixture-acceptor
  evidence_basis: unevidenced
relations:
  elaborated_by:
    - SCR-PRD-delivery-slo
---

# Published delivery objective

## Business need

Beacon sells at-least-once delivery and publishes no number beside it. Every enterprise procurement review asks for one, and the answer is a sentence an account manager writes by hand. Two deals stalled last quarter on the difference between two such sentences.

An unpublished objective also costs the delivery team. Nobody inside Beacon agrees on what counts as an incident, so a destination that is slow for one tenant is escalated by one engineer and ignored by another.

The boundary of this initiative is the objective and the report of it. The changes to the delivery worker that a missed objective would ask for belong to the delivery roadmap.

## Business requirements

1. Beacon publishes one delivery objective that every contract cites, and no account manager writes a second one.
2. A tenant reads their own attainment against the objective, for the current month and for the twelve months before it.
3. The delivery team is alerted when attainment for any tenant drops below the objective, before the tenant notices.
4. A missed objective produces a credit that is calculated from the published number rather than negotiated per tenant.
5. The objective covers the path Beacon controls, and it states plainly that a destination that refuses an event is outside it.

Requirement 4 is planned for the quarter after the objective is published, and the credit schedule is to be decided with finance.
