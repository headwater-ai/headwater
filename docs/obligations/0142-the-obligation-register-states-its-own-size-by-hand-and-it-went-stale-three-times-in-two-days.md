---
id: HW-OBL-0142
status: draft
status_since: 2026-08-26
last_verified: 2026-08-26
title: "The obligation register states its own size by hand, and it went stale three times in two days"
summary: "Spec 13 states its own item and record counts by hand, and the figure drifted three times in two days before anyone noticed."
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: draft
  accepted_by: j.baxter
  evidence_basis: evidenced
waiting_on: adopter
---

# The obligation register states its own size by hand, and it went stale three times in two days

## Context

The build-order run's parent measured [spec 13](../spec/13-open-obligations.md) against its own merge commits and found its stated counts wrong three times in two days. At `9253b97` the file said 117 records against 125 on disk, eight short. At `c4a0ff0` it said 126 records and 122 items against 127 on disk, one short. Each gap stood until an iteration doing unrelated work tripped over it and repaired it, first at #232 and again at #244. Neither repair was a deliberate check for drift. [Spec 6](../spec/06-engine-architecture.md#taxonomy-validate-versus-taxonomy-audit) already names this defect and states the remedy, that a document needing such a figure names the verb producing it instead. An earlier pass rewrote six committed counts on that rule, and spec 13 was not among them.

## Obligation

Spec 13 opens with hand-typed counts of its own population, how many items and how many obligation records the shelf holds. Several of its headings restate narrower counts of the same kind. Nothing reads a number in this prose against a corpus measurement. `headwater generate --check` holds no part of it, because spec 13 is authored rather than generated. So the file sits outside the one mechanism that could catch a stale figure. The only check that has ever caught the drift is a person noticing, and the record above is what that mechanism achieves across two days. The corpus owes spec 13 the same treatment spec 6 already prescribes elsewhere. A sentence naming the verb that produces the figure replaces the number.

A second reading stays open beside it. Spec 13's index of records is already a hand-maintained projection of `docs/obligations/**`. A generated index would carry a correct count for free. It would also bring the file under `generate --check`. That option removes the whole class of drift rather than this one instance, and costs more to build.

A third debt sits behind both. Three hand-typed counts of a corpus population were found here by accident, in one run, by a reader not looking for them. Whether any other governed document carries the same defect has not been measured.

## Discharge

This closes when no hand-typed count of items or records survives in spec 13. Either each such sentence names the verb producing its figure, or the block carrying it becomes a generated projection that `generate --check` holds. A figure that stays hand-written needs a stated reason, placed where a reader meets it, for why that one is exempt from spec 6's rule. Proof is a change that adds an obligation record and shows the figure move, or shows the gate fail when it does not. A claim that it would work is not enough. The sweep for other hand-typed corpus counts across the governed documents is a separate debt. This record only states that debt, and does not close it. Closing it needs its own measurement, and its own record if anything turns up.
