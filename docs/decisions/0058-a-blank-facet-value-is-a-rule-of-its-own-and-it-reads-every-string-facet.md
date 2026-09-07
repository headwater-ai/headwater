---
id: HW-DR-0058
status: current
status_since: 2026-09-07
summary: "A declared facet that carries no content is a state that neither the required-facet rule nor the enum rule reaches. A new rule reports it, over every facet a taxonomy types as a string, at error severity."
last_verified: 2026-09-07
title: "A blank facet value is a rule of its own and it reads every string facet"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
---

# A blank facet value is a rule of its own and it reads every string facet

## Context

[#541](https://github.com/headwater-ai/headwater/issues/541) reports that a document whose `title` carries no value passes `check --strict` and reaches a generated shelf index as the identifier. The issue leaves two questions open. The first asks whether the answer widens `facet.required.missing` or adds a rule. The second asks whether the answer reads the facet in the `name` role or every facet.

**The population of degenerate shapes comes from the parser and not from a definition of blank.** `core_schema::as_null` resolves a plain scalar to null when its text is empty, `~`, `null`, `Null` or `NULL`. Twelve inputs were written onto one decision record of this corpus on 2026-09-07 and each one was run alone. Eleven of the twelve passed `check --strict` at exit 0, and the absent key was the one shape that a rule reported.

**Three of those eleven are green through every gate this engine has.** `title:` with nothing after the colon reaches the check layer as a plain scalar whose text is the literal `~`. A filter written as `trim().is_empty()` reads that text as content, so `headwater generate` writes the shelf index at exit 0 and the page renders `[~]`. `title: null` renders `[null]` on the same terms. The other eight shapes stop `headwater generate` and pass the check layer, which is a defect that a reader meets one verb later.

**A widening of the required-facet rule answers the second question with the wrong set.** That rule instantiates over the facets a kind requires. Five kinds of this corpus require `title` through the adopter overlay. The bundles require it on nine more, so a widening would reach most of this corpus. It would reach no facet that a kind declares as optional, and `oracle`, `verification_method` and `waiting_on` are all optional and all typed as strings. The rule would also fold two states into one message, because a key that is absent and a key that carries nothing are two repairs.

## Decision

**A new document-scoped rule, `facet.value.blank`, reports a declared facet that carries no content.** It sits beside `facet.required.missing`, which reports a key that is absent, and `facet.value.not_permitted`, which reports a scalar outside a closed set. The three rules partition the states a facet value can hold.

**The population is every facet whose declaration says `type: string` and that the kind does not forbid.** The role of the facet decides nothing here. The issue states that the gap is general rather than a property of `title`. A rule scoped to the `name` role would reopen it for the next emitter that reads a facet a person wrote.

**The rule carries three sub-verdicts and one message for each.** A plain scalar that `core_schema::as_null` resolves reports that the facet is declared with no value. A scalar whose text trims to nothing reports empty text. A value that is not a scalar at all reports the sequence or the mapping against the type the declaration states.

**The severity is error and the rule offers no patch.** [Spec 12](../spec/12-check-layer.md#fixability) makes a fix mechanical only where one correct outcome follows without judgment. The value of a blank `title` is a name a person writes. `facet.required.missing` holds the same pair of properties. The two states have to carry one severity, or the asymmetry the issue names survives the answer. The remediation names both repairs where the kind does not require the facet: write a value, or remove the key.

## Consequences

**The rule reports nothing over this corpus.** One instance is created for each of the 291 classified documents, so the instance count of a full run moves from 5324 to 5615. The finding count holds at 68, all of them advisory, and `check --strict` exits 0.

**A facet that declares a value set is read twice.** `verification_method` and `waiting_on` declare `type: string` and a closed set, so a blank value on one of them raises this finding and `facet.value.not_permitted` as well. Both statements are true and both name a different repair. A population that depended on a second declaration would be a narrowing that no reader of the taxonomy could predict from it.

**The rule reaches no obligation and the report says so.** `Bound::Unnamed` is legal and `control.mechanism.unimplemented` reports only the opposite direction, so an uncited rule is not a finding. The control that would bind it belongs in `headwater/standard` beside `CT-FACET-1` and `CT-FACET-2`, and that edit moves the package, the pin digest and the lock. [HW-OBL-0170](../obligations/0170-the-blank-facet-value-rule-reaches-no-obligation-so-the-report-names-none.md) holds the debt.

**A kind that has to generate no instance has one more facet family to forbid.** The coverage rule needs a classified document that no instance reads. The fixture corpus of the check crate builds one from a kind that forbids every facet the Shape rules read. A string facet added to that taxonomy has to join the `forbid` list of that kind, or the failing fixture for `coverage.document_unchecked` stops being reachable.

**Whether `type:` binds a document facet value at all stays open.** This rule reads the declared type of a facet to decide its own population. It reports a sequence or a mapping only for the facets in that population. A date facet carrying a mapping, and an enumerated facet carrying a sequence, both reach `check --strict` at exit 0. [HW-OBL-0171](../obligations/0171-no-rule-reads-a-document-facet-value-against-the-type-its-declaration-states.md) holds that question.
