---
id: HW-OBL-0185
status: current
status_since: 2026-09-11
summary: "An entry the canonical library refuses is required by two entries it admits, and no criterion and no ruling reads that direction"
last_verified: 2026-09-11
title: "Whether an admitted library entry may require a bundle that admission refuses"
waiting_on: ruling
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
---

# Whether an admitted library entry may require a bundle that admission refuses

## Context

The canonical taxonomy library admits an entry when all seven of its [admission criteria](../taxonomies/README.md#admission-criteria) hold. `evidence-and-obligation` is refused by the admission table. Criterion 1 refuses it because it models no tradition, and criterion 4 refuses it because it carries no worked corpus of its own. [HW-DR-0044](../decisions/0044-q44-whether-bundles-decompose-into-capabilities-and-assemblies-compose-practices.md) authorized the split that made it, and it settled nothing about admission.

Two admitted entries declare `requires: [evidence-and-obligation]`. `design-spec` declares it for the `evaluation` kind that left it, and `decision-record` declares it for the `obligation` purpose. The `headwater/starter` recipe also selects it. So a composer who takes either admitted entry takes a refused bundle with it, and no criterion of the seven asks about that direction.

This question was raised as part of [#510](https://github.com/headwater-ai/headwater/issues/510) and the library index recorded it against that issue number. A reader who followed the citation arrived back at the open issue that asked it. This record is the answer to that: the question has a home, and the home is not the issue.

## Obligation

The owner owes a ruling on one question. **May an entry that admission admits require a bundle that admission refuses?** Three outcomes are open, and this record prefers none of them. The criteria may bind the whole dependency closure of an admitted entry, which would refuse the two entries above as they stand. The criteria may bind the authored entry alone, which makes the present shape correct and needs criterion 6 to say so. Or a third status may exist between admitted and refused, for a capability split that no adopter selects on its own.

**Criterion 7 measured against the entry is the evidence that the question is more than a matter of order.** The [meta-schema](../spec/02-taxonomy-model.md#the-meta-schema) asks that every declared purpose is served by a concrete kind. `evidence-and-obligation` declares `purposes.evidence` and `purposes.obligation`. Its one kind, `evaluation`, serves `evidence`. Nothing in the entry serves `obligation`. The two kinds that serve it, `obligation_register` and `obligation_record`, live in the two entries that require this one. The rule therefore holds over the closure of an entry that requires this bundle, and over no closure this bundle can name. `requires: []` is not an omission that a later change repairs, because a requirement in the other direction makes a cycle.

**Nothing measures that rule today.** Purpose completeness is one of the seventeen entries of `headwater_meta::validate::SKIPPED` in `engine/crates/meta/src/validate.rs`, because it needs a resolved tree. No verb runs it after resolution. `headwater-resolve` asks whether a concrete kind serves a *core* purpose, in `engine/crates/resolve/src/core.rs`, and it asks nothing about the other declared purposes. The library index says that criteria 3, 6 and 7 become mechanical the day the resolver exists. That day arrived for criterion 3 and for criterion 6, and it has not arrived for this rule of criterion 7.

The wording of criterion 5 for the same bundle is a separate open item, carried by [#520](https://github.com/headwater-ai/headwater/issues/520). This record does not cover it.

## Discharge

This record discharges when the owner rules, in a decision record that states the reason and the condition that reopens the ruling. A ruling that the present shape is correct discharges it as fully as a ruling that refuses the shape. What stands open is the silence rather than either answer.

A ruling that the criteria bind the closure needs one more thing. It needs a sentence in criterion 6 or criterion 7 of `docs/taxonomies/README.md` that says so, because a criterion nobody wrote down binds nothing.

The unmeasured meta-schema rule is a second and smaller item. It discharges when a verb reads purpose completeness over a resolved tree, or when the library index stops promising that criterion 7 becomes mechanical. Neither is a ruling, and this record states the gap rather than holds it.
