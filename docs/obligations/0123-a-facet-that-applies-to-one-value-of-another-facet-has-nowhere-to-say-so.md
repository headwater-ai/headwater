---
id: HW-OBL-0123
status: current
status_since: 2026-08-14
waiting_on: ruling
summary: "Two rules of the language meet on a facet whose value is the author's judgment, and a sentinel is what the corpus wrote instead."
last_verified: 2026-08-14
title: "A facet that applies to one value of another facet has nowhere to say so"
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-taxonomy-model
---

# A facet that applies to one value of another facet has nowhere to say so

## Context

The measurement layer declared two kinds and five facets, and getting them declared met two rules of this language at once. The [facet canons](../spec/02-taxonomy-model.md) refuse a facet that no kind requires, no shelf discriminates on, no expectation reads and no projection filters on. `headwater new` refuses a kind that requires a facet with a closed value set that no engine role determines. So a facet is required or it is unread, and a value that an author chooses is neither.

Both refusals were measured on this repository rather than argued:

    the resolved taxonomy `facets.probe_category`: facet canons: relevance: nothing reads it

    headwater: `probe` requires the facet `probe_category`, and it declares a closed value set and it carries no role this engine derives a value for

The first is what the resolver says when `probe_category` is optional. The second is what the scaffolder says when it is required. `headwater new --facet <facet>=<value>` answers the second, so the four closed sets of the measurement layer are required and a person states each value.

## Obligation

**The first gap is closed and the second is not.** `oracle` names the check that a `patched` expectation grades a produced patch against, and it applies to one value of `expectation` and to no other. The language cannot say that. So `oracle` is required of every probe, and four of the five expectation forms write `none` into it.

The corpus owes a way to state that a facet is required when another facet holds a named value. Two shapes are candidates. A `required_when` member on the facet, read by `facet.required.missing`, would put the condition where the facet is. The other shape is a kind for each expectation form. It costs five kinds that differ in one value. [Spec 2](../spec/02-taxonomy-model.md) refuses that shape: two documents that differ only in a value are one kind with a facet.

## Discharge

Nothing discharges this today, and the harness pays the cost meanwhile. `headwater probe plan` refuses a `patched` probe that declares the sentinel. It also refuses a probe of any other form that names a rule. Both directions are enforced, and no check enforces either: the harness never gates. A probe with the wrong pairing reaches a commit and stops a run rather than a merge.

**That measurement holds while `docs/probe-runs/` holds nothing, and the day a transcript lands it stops holding.** A `probe_result` projection is graded against the selection the plan composes. A plan the wrong pairing refuses composes none, so the projection writes no result. `generate --check` then reports the committed result as one this corpus no longer derives. The merge stops and the reason names the probe. The gate is then the projection layer and never a check, and it reaches the pairing only through a corpus that holds a recorded run.

That is the measurement this record holds. The rule that would catch it at commit time cannot be declared, and the count is one facet of five in one repository. A second corpus with a conditional facet is what turns one instance into evidence that the shape is general.

## What the grader added, which is a second instance and a third shape

**The shape recurred inside this repository before any second corpus arrived.** The `answered` expectation is satisfied by one value of a closed set that the probe declares, and nothing held that set. It applies to one value of `expectation` and to no other. That is this record's shape exactly, so the count is two conditional declarations of five forms rather than one.

**The second one did not become a facet, and the reason names a shape this record does not carry.** A closed set of answers is a list, and no facet of this language holds a list. It goes in a fenced block under the `Expectation` section, which the `probe` kind already requires. `headwater probe plan` enforces the pairing in both directions, as it does for the oracle. **A required section carries a per-document declaration where a facet cannot.** That is a third candidate beside `required_when` and the kind-per-form, and it is cheaper than either. It costs the four other forms nothing, and no sentinel is written anywhere.

It is not free, and the price is what keeps this record open. A facet is what a check reads, and a fenced block under a heading is read by the harness alone. So the pairing for `answers` is enforced in one place and by one component, exactly as the pairing for `oracle` is. Whether `oracle` should move to the same section is a question for whoever discharges this. The move would trade a sentinel on every probe for a declaration that no check can see.
