---
id: SCR-BRD-tenant-billing
status: current
status_since: 2026-01-15
last_verified: 2026-08-20
summary: Why Beacon bills for delivery volume, and the planted absence of any product requirements document that answers it.
requirement_tier: brd
provenance:
  warrant: accepted
  agency: human
  accepted_by: fixture-acceptor
  evidence_basis: unevidenced
---

# Usage-based billing

## Business need

Beacon charges one monthly fee whatever a tenant sends. The largest tenant sends 400 times the volume of the median one and pays the same amount, so the largest tenants are carried by the smallest. Two of the three most expensive tenants are also the two least profitable accounts.

The flat fee also stops the sales team selling to anyone small. A tenant who sends 200 events a month is asked for the same fee as a tenant who sends eight million, and the conversation ends there.

The price per delivered event is to be decided with finance. This document states the shape of the charge and never the number.

The boundary of this initiative is the price of delivery volume. Support tiers, storage of the attempt history and the enterprise contract terms are priced elsewhere.

## Business requirements

1. A tenant is charged for the events Beacon accepted on their behalf, measured over a calendar month.
2. A tenant reads their own accrued charge at any point in the month, and the figure matches the invoice at the end of it.
3. A tenant is warned before their accrued charge passes a limit they set.
4. The volume Beacon bills for is the volume the attainment report counts, and no second counter exists.
5. An existing tenant moves to the new price with 90 days of notice and no interruption of delivery.

## Success measures

Revenue per delivered event varies by a factor of 400 across tenants today. Success is a factor under 3.

The share of accounts that are unprofitable at the current fee is 18% today. Success is under 5%.

The smallest tenant the sales team will quote sends 50,000 events a month today. Success is any tenant who wants a destination.

Billing disputes run at 4 per quarter today, all of them about what the fee covers. Success is under 1 per quarter.
