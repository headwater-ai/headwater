---
id: HW-OBL-0171
status: current
status_since: 2026-09-07
summary: "A facet declares `type: date`, `type: integer` or `type: string`, and no check reads a document's value against that declaration. A mapping where a date belongs reaches a strict run at exit 0."
last_verified: 2026-09-07
title: "No rule reads a document facet value against the type its declaration states"
waiting_on: ruling
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0058
    - engine/crates/check/src/facet_blank.rs
---

# No rule reads a document facet value against the type its declaration states

## Context

A facet declaration in the resolved taxonomy of this corpus carries a `type` member. Five facets declare `type: string`, two declare `type: date`, and one declares `type: integer`. The member is read by the meta-schema, which owns the shape of a taxonomy, and by one rule of the check layer.

That one rule is `facet.value.blank`, and it reads the member to decide its own population rather than to type a value. It reports a sequence or a mapping where the declaration says `string`, and it reports nothing at all about the two other type names. So `status_since` carrying a mapping, and `sequence` carrying a list, both reach `check --strict` at exit 0.

The engine takes a stated posture toward this. `facet_value.rs` says that a facet whose value is a mapping or a list produces nothing there, where the taxonomy declared a set of scalars. The reason it gives is that the meta-schema owns shape, and that a second report of one defect sends an author to two places. That posture is right about a taxonomy, which the meta-schema validates. It leaves a document unread, because no schema validates a document's front matter against the facet declarations.

[HW-DR-0058](../decisions/0058-a-blank-facet-value-is-a-rule-of-its-own-and-it-reads-every-string-facet.md) settles the narrow case and states this one as its residue. The question is not which rule reports a wrongly typed value. The question is whether the check layer types a document facet value at all. The other answer is that an emitted JSON Schema owns it.

## Obligation

The corpus owes a ruling on where a document facet value meets the type its declaration states. Three answers are open, and each one carries a different cost.

The first widens `facet.value.blank` into a rule about types. That makes one rule carry two subjects, and it needs a parser for each type name a taxonomy may write. The second adds a Shape-origin rule for each declared type, which is the shape the existing facet rules take. The third rules that the emitted JSON Schema owns it, which reaches only the adopters who run an emitter and leaves `headwater check` silent.

A ruling settles which one holds, and it also settles what an unknown type name decides. The meta-schema owns the set of names, and a rule that refused a name it did not know would refuse a taxonomy the meta-schema admits.

## Discharge

A decision record states which component reads a document facet value against its declared type, and the reasoning behind the choice. Where the answer is a rule of this engine, the rule ships with a fixture it fails and a fixture it passes. That is the bar [spec 12](../spec/12-check-layer.md#testing-a-check-without-a-failing-fixture-does-not-ship) sets.
