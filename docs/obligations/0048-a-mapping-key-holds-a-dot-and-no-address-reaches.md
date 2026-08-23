---
id: HW-OBL-0048
title: "A mapping key holds a dot, and no address reaches it"
status: current
status_since: 2026-08-12
waiting_on: ruling
last_verified: 2026-08-13
summary: "`facet_values` is keyed by a facet and a value joined by a dot, which the reference sublanguage rules ambiguous."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-taxonomy-model
---

# A mapping key holds a dot, and no address reaches it

## Context

The worked example keys `facet_values` with a facet and one of its values, joined by a dot, as `status.current`. The [sublanguage](../spec/02-taxonomy-model.md#the--reference-sublanguage) rules that a declared key with a dot makes an address ambiguous, and that the meta-schema refuses to declare one. Both clauses belong to [spec 2](../spec/02-taxonomy-model.md#mapping-between-taxonomies).

## Obligation

Either the key becomes two levels of mapping, or the specification states that some declarations are outside the reach of an overlay.

## Discharge

`engine/crates/meta/meta-schema.yml` takes the reading that no overlay reaches inside `facet_values`, and it marks the position. Nothing else can settle it, because the choice is between two shapes and both parse.
