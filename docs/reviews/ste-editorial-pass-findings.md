---
id: HW-REV-ste-editorial-pass
status: current
status_since: 2026-08-10
last_verified: 2026-08-10
summary: What the ASD-STE100 editorial pass found in the source content, and the terminology rulings that the glossary pass raised.
doc_type: review_record
title: "STE editorial pass — source findings to address"
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: [claude-fable-5, claude-opus-5]
  activity: edit+report
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  assesses:
    - HW-SPEC-vision-and-scope
    - HW-SPEC-conceptual-model
    - HW-SPEC-taxonomy-model
    - HW-SPEC-authoring-and-lifecycle
    - HW-SPEC-assurance-model
    - HW-SPEC-ai-integration
    - HW-SPEC-engine-architecture
    - HW-SPEC-distribution-and-federation
    - HW-REG-open-questions
    - HW-EVAL-theoretical-foundations
    - HW-EVAL-adjacent-work
    - HW-SPEC-glossary
---

# STE editorial pass — source findings to address

Point-in-time record, 2026-08-10. During the ASD-STE100 house-profile pass over specs 0–12 (commits `786c5e2`, `d03d613`, `651a302`), the editors flagged the items below as possible defects or open decisions in the *source content*. Editorial policy was to report, not fix, anything that needs author judgment. Two objective typos found in the same pass were fixed directly and are not listed here.

## Possible wording defects

| Location | Finding | Suggested action |
|---|---|---|
| [spec 4](../spec/04-assurance-model.md), degraded-controls discussion | "An uncovered control cannot hide" — context (gaps attach to obligations; controls degrade) suggests the intended word is *obligation*, not *control* | Confirm and correct the noun |
| [spec 4](../spec/04-assurance-model.md), same section | "A reader who cannot tell the maintainer … is a control that does not exist" equates a person with a control; the intended referent is probably the missing feedback channel | Decide whether the metonymy is intended |
| [spec 9](../spec/09-open-questions.md), Q17 | "a determined reader can difference" uses *difference* as a verb; reads as intentional diff-terminology but is nonstandard | Keep deliberately or change to "diff"/"compare" |
| [spec 10](../evaluations/theoretical-foundations.md), B.2 | Source read "every enum facet's value document how to choose"; the pass rendered it "every enum facet value must document how to choose it" — the original may have been a typo for "facet's values document" | Confirm the rendered meaning is the intended one |
| [spec 3](../spec/03-authoring-and-lifecycle.md), absence-class sentence | "is the same absence class" was rendered "is in the same absence class"; flag in case the identity phrasing (a class, not membership) was intentional | Confirm |
| [spec 11](../evaluations/adjacent-work.md), §A intro | "the `validate` / `audit` split arrived at [spec 6]" is the source author's own construction and reads oddly | Optional rephrase |

## Terminology and spelling decisions (corpus-wide rulings needed)

The last three rows arrived later the same day, from the pass that built the [glossary](../spec/glossary.md) rather than from the STE pass. An index of every term is what makes a name collision visible, so the glossary found them. They are recorded here because this table is where corpus-wide terminology rulings are tracked.

| Question | Evidence | Options |
|---|---|---|
| American or British spelling | Corpus was British (organisation, artefact, customise) with drift: "knowledge organization" in [spec 2](../spec/02-taxonomy-model.md) and [spec 7](../spec/07-distribution-and-federation.md); "judgment" and "judgement" both occurred. STE rule 1.14 prescribes American unless a directive says otherwise | **Resolved 2026-08-10: American ruled (CLAUDE.md), corpus swept, reviews left as written** |
| "register", two senses | [Spec 1](../spec/01-conceptual-model.md) uses *register* for the voice regime and for the register projection — one term, two concepts, a rule-1.11 violation in spirit, sharpened by the recent retirement of the standalone register concept | **Resolved 2026-08-10: the voice sense renamed to "voice". The register projection and the registers it generates keep the word** |
| "fails open" | [Spec 5](../spec/05-ai-integration.md) says routing "fails open" meaning it stays silent below the confidence threshold; conventional security usage would call that failing closed. The usage is consistent with design principle 7 in [spec 0](../spec/00-vision-and-scope.md) | Keep as project vocabulary (documented in principle 7) or align with conventional usage |
| "gap", two senses | A [disposition](../spec/04-assurance-model.md) on an obligation (no control discharges it yet) and a value of `evidence_basis` on a document ([spec 3](../spec/03-authoring-and-lifecycle.md)). Both senses are then said to "appear in the gap register", so the collision may not be only lexical: the specs do not say whether that is one register or two | **Resolved 2026-08-10: the `evidence_basis` value renamed to `unevidenced`, parallel to `evidenced` and `reconstructed`. `gap` is now only the disposition. Still open: whether spec 3 and spec 4 mean the same gap register, or two that share a name — recorded in [spec 3](../spec/03-authoring-and-lifecycle.md#evidence-has-three-honest-states-not-two)** |
| "audit", two senses | `taxonomy audit` ([spec 6](../spec/06-engine-architecture.md)) measures a schema against a corpus and is a CLI verb. A conformance audit ([spec 4](../spec/04-assurance-model.md)) is a periodic human sample that asks whether a specification is still true of the system | **Resolved 2026-08-10: the spec 4 activity renamed to "accuracy audit", which states what it checks. `taxonomy audit` keeps the CLI verb** |
| "conformance", two senses | [Spec 4](../spec/04-assurance-model.md) uses it for whether the corpus still describes the system. [Spec 7](../spec/07-distribution-and-federation.md) uses it for whether a consumer wired the published method, with `headwater conformance` as the command. The two ask unrelated questions | **Resolved 2026-08-10 with the row above: "conformance" is reserved for the spec 7 adoption sense, which owns the command name** |
