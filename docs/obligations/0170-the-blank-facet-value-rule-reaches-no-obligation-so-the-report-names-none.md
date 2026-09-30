---
id: HW-OBL-0170
status: discharged
status_since: 2026-10-01
summary: "Every other rule this engine carries reaches one obligation. `facet.value.blank` reaches none, because the control that would bind it sits in the vendored package and the edit moves the pin digest."
last_verified: 2026-10-01
title: "The blank facet value rule reaches no obligation so the report names none"
waiting_on: build
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

# The blank facet value rule reaches no obligation so the report names none

## Context

[HW-DR-0058](../decisions/0058-a-blank-facet-value-is-a-rule-of-its-own-and-it-reads-every-string-facet.md) rules that a declared facet carrying no content is a rule of its own. `facet.value.blank` is that rule, and it ships with no control that names it.

The register section of a check report states one line for each rule that reaches no obligation. Over this corpus that section held one sentence of the opposite shape: every rule this engine carries reaches one obligation. The report for this corpus states instead that `facet.value.blank` reaches no obligation, so it names none. The finding the rule raises carries no obligation identifier beside it, where the finding of `facet.required.missing` carries `OB-FACET-1`.

Nothing refuses this. `Bound::Unnamed` is a legal state of a served rule. `control.mechanism.unimplemented` reports the opposite direction only, which is a control naming a mechanism the engine does not carry. So an uncited rule is a quiet gap rather than a finding.

**The reason it ships uncited is the cost of the edit and not a judgment about the rule.** `OB-FACET-1` and `CT-FACET-1` are declared in `headwater/standard`, which this repository consumes as a vendored artifact under a pinned digest. A sibling obligation and control belong beside them, and writing them moves `taxonomy-source/headwater-standard/`, the vendored package, the digest in `.headwater/taxonomy.yml` and the lock. Three recorded fixtures of the engine workspace read the resolved taxonomy and move with it.

## Obligation

The corpus owes an obligation and a control in `headwater/standard` that bind `facet.value.blank`, in the shape `CT-FACET-1` and `CT-FACET-2` already carry. The statement of the obligation is the state the rule reports: a facet a document declares carries a value.

The order of the edit is fixed by [the package contract](../spec/07-distribution-and-federation.md): publish the source, move the pin, vendor the artifact, and resolve the lock. A run that resolves before it publishes leaves both sides agreeing on the old bytes, and the comparison that would report the gap passes.

## Discharge

The register section of a report over this corpus lists no rule as reaching no obligation. The exception is the three rules that are about the taxonomy rather than about a document. A finding of `facet.value.blank` carries an obligation identifier beside the rule name, in the way a finding of `facet.required.missing` carries `OB-FACET-1`.

**[#1492](https://github.com/headwater-ai/headwater/issues/1492) met that condition, and this record is discharged.** On the day of the change, twelve rules reached no obligation, and `facet.value.blank` was one of them. `headwater/standard` 4.14.0 adds ten obligations and twelve controls. Each control names one of the twelve rules and discharges one obligation. `CT-FACET-3` binds `facet.value.blank` to `OB-FACET-3`, whose statement is the one this record asked for: a facet that a document declares carries a value. The register section of a report over this corpus now reads "every rule this engine carries reaches one obligation". The change needed no exception for a rule about the taxonomy, because `control.observation.invalid` reaches `OB-REG-3`.

A finding of `facet.value.blank` now prints `(OB-FACET-3)` beside the rule name, and `engine/crates/cli/tests/blank_facet.rs` asserts it. The case `every_rule_reaches_one_obligation_of_the_standard_package` in `engine/crates/cli/tests/standard_controls.rs` fails on each rule that reaches no single obligation, and it names each one. So the next rule that ships without a control fails that case.
