---
id: OBL-repo-0030
title: "The provenance block belongs to the engine, and nothing states its shape"
status: current
status_since: 2026-08-13
last_verified: 2026-08-14
summary: "Spec 3 gives the provenance block to the engine and states no shape for it, so a read depends on a key that nothing validates."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - SPEC-HW-authoring-and-lifecycle
    - SPEC-HW-ai-integration
---

# The provenance block belongs to the engine, and nothing states its shape

## Context

[Spec 3](../spec/03-authoring-and-lifecycle.md#provenance-is-recorded-not-assumed) puts the shape of the provenance block with the engine rather than with a taxonomy. It also says that four engine rules turn on the warrant. The [meta-schema](../../engine/crates/meta/meta-schema.yml) declares no provenance block, and no check reads one.

## Obligation

The query surface is the first code to read `provenance.warrant`, because [spec 5](../spec/05-ai-integration.md#intent-time-routing) makes a pointer state the `asserted` warrant beside the summary. So a read depends on a key that nothing validates. A document that misspells the key gets a pointer with no warning on it, and every rule that spec 3 promises is still absent.

## Discharge

**Onboarding was the obvious place to settle this and it settled the other half instead.** `headwater init` and `headwater infer` write no provenance block into any document, and the reason is not the missing shape. `accepted_by` records a human act, so a verb that wrote one across a corpus would assert an acceptance that nobody performed. The [first typing of this corpus](../spec/13-open-obligations.md#what-the-first-typing-of-this-corpus-found) already did that once by hand, and [the retrofit record](0057-a-retrofit-cannot-recover-the-two-dates-or-the-acceptance.md) states the result. A scaffolder that mints one document is where a warrant can be true.

**The scaffolder mints one document, and the missing shape is what stops it.** `headwater new` writes the facets that a kind requires and nothing else, because a facet is what a taxonomy declares and what a check reads. No taxonomy declares a provenance block, so the scaffolder has no declaration to derive one from. A block it wrote from an internal shape would put an unvalidated field into every document that a corpus mints. That is the drift that this record is about. So every scaffolded document of this repository carries no provenance block, and every hand-typed one carries one. That is the cost of the gap, stated as a fact about the tree rather than as a forecast.

**The acceptance half stays where it was, and the drafting half is now answerable.** `accepted_by` records a human act, and a run of a verb is not one. `warrant`, `agency`, `drafted_by` and `activity` describe who produced the text, and a scaffolder knows all four about the document it just wrote. What closes this record is a shape that a check reads, and the scaffolder is the caller that would fill it.
