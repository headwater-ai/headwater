---
id: OBL-repo-0123
status: current
status_since: 2026-08-14
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
    - SPEC-HW-taxonomy-model
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

That is the measurement this record holds. The rule that would catch it at commit time cannot be declared, and the count is one facet of five in one repository. A second corpus with a conditional facet is what turns one instance into evidence that the shape is general.
