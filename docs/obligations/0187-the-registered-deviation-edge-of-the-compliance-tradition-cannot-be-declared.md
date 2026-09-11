---
id: HW-OBL-0187
status: current
status_since: 2026-09-11
summary: "Spec 2 names a registered-deviation check on `does_not_comply_with`, and no entry can enable or declare that relation."
last_verified: 2026-09-11
title: "The registered-deviation edge of the compliance tradition cannot be declared"
waiting_on: ruling
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
---

# The registered-deviation edge of the compliance tradition cannot be declared

## Context

[Spec 2](../spec/02-taxonomy-model.md#the-decision-relation-vocabulary) names four checks that the decision-relation vocabulary brings. One of them is a `does_not_comply_with` edge that points at a `current` standard. That edge is a registered deviation, and it carries an owner and an expiry.

The [`standards-spec` entry](../taxonomies/standards-spec/doctrine.md#findings) is the first entry to model a compliance tradition, and it could not declare that edge. The relation is defined-but-unenabled package content, reached by `$package.optional.does_not_comply_with`, and `package.optional` holds nothing. The entry cannot enable it. The entry may not declare the name either, because the package collides with that address the day `package.optional` holds the relation. `OB-SS-3` of the entry carries the gap, and the argued form is the first finding at [line 148](https://github.com/headwater-ai/headwater/blob/5fb9518/docs/taxonomies/standards-spec/doctrine.md#L148) of the doctrine.

## Obligation

The central negative edge of the tradition is unreachable, so a check that spec 2 names runs for no adopter. [HW-OBL-0027](0027-what-optional-holds-under-the-package-root.md) holds the root question, which is what `optional` holds under the `package` root. This record is the first measured consequence of that silence rather than a second copy of it.

An adopter meets the gap directly. A team that takes a standards taxonomy to record its deviations takes a taxonomy that cannot say a document deviates.

## Discharge

This record discharges when `package.optional` holds `does_not_comply_with` and an entry may enable it. It discharges the other way when spec 2 stops naming a check that no taxonomy can produce. A ruling on HW-OBL-0027 that leaves `optional` empty discharges this record as well, if it says what an entry declares instead.
