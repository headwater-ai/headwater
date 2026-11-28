---
id: HW-OBL-0268
status: current
status_since: 2026-11-24
summary: "Four rulings live only in issues, contracts and comments: the blocking posture, the seventh facet role, the identifier step of route and the baseline marker."
last_verified: 2026-11-24
title: "Four rulings of run 20261122-0114 that no decision record carries"
waiting_on: ruling
provenance:
  warrant: asserted
---

# Four rulings of run 20261122-0114 that no decision record carries

## Context

[The register of open obligations](../spec/open-obligations.md) holds a finding that has no reader outside this repository. Run `20261122-0114` wrote intake lines 88, 114, 137 and 149. The product owner ruled each one RECORD into the group that its end-of-run report calls S5, a reason with no decision record. That group held eight lines. A record that bundles gaps is split when it passes five, so this record holds the rulings. This record searched `docs/decisions/` again on `0cdd96298`. Each line named an outside reader, and the product owner found that a contract or spec page already states the ruling for that reader.

## Obligation

- The owner ruled on #2015, on 2026-11-22, that posture is the adopter's to configure and that a blocking control fails `--strict` at any severity. The ruling lives in the issue, a taxonomy comment, [the check layer](../spec/check-layer.md) and the `headwater check` contract. No decision record names `posture: blocking` or `blocked_by`.
- The seventh facet role and edition 2 of `identifier.claimed_twice`, from #2025, have no decision record. [The taxonomy model](../spec/taxonomy-model.md), [authoring and lifecycle](../spec/authoring-and-lifecycle.md) and the issue state them. The first half of line 114 does not hold, because `command-surface.md` governs `engine/crates/api/src/**`.
- The identifier step of `headwater route` names the document of a live or former identifier, outside the purpose gate and the budget. Only `docs/interfaces/headwater-route.md` states it, and it cites #2025.
- The marker `verified_as: baseline` on `governs`, from headwater/standard 4.24.0 and the owner ruling of 2026-11-22 on #2016, has no decision or evaluation. The `--baseline` paragraph of `docs/interfaces/headwater-check.md` and a taxonomy comment state it.

## Discharge

Each item discharges alone, when a decision record carries the ruling and the documents that state it cite that record. The record discharges when every item has.
