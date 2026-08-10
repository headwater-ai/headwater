# STE editorial pass — source findings to address

Point-in-time record, 2026-08-10. During the ASD-STE100 house-profile pass over specs 0–12 (commits `786c5e2`, `d03d613`, `651a302`), the editors flagged the items below as possible defects or open decisions in the *source content*. Editorial policy was to report, not fix, anything that needs author judgment. Two objective typos found in the same pass were fixed directly and are not listed here.

## Possible wording defects

| Location | Finding | Suggested action |
|---|---|---|
| [spec 4](../spec/04-assurance-model.md), degraded-controls discussion | "An uncovered control cannot hide" — context (gaps attach to obligations; controls degrade) suggests the intended word is *obligation*, not *control* | Confirm and correct the noun |
| [spec 4](../spec/04-assurance-model.md), same section | "A reader who cannot tell the maintainer … is a control that does not exist" equates a person with a control; the intended referent is probably the missing feedback channel | Decide whether the metonymy is intended |
| [spec 9](../spec/09-open-questions.md), Q17 | "a determined reader can difference" uses *difference* as a verb; reads as intentional diff-terminology but is nonstandard | Keep deliberately or change to "diff"/"compare" |
| [spec 10](../spec/10-theoretical-foundations.md), B.2 | Source read "every enum facet's value document how to choose"; the pass rendered it "every enum facet value must document how to choose it" — the original may have been a typo for "facet's values document" | Confirm the rendered meaning is the intended one |
| [spec 3](../spec/03-authoring-and-lifecycle.md), absence-class sentence | "is the same absence class" was rendered "is in the same absence class"; flag in case the identity phrasing (a class, not membership) was intentional | Confirm |
| [spec 11](../spec/11-adjacent-work.md), §A intro | "the `validate` / `audit` split arrived at [spec 6]" is the source author's own construction and reads oddly | Optional rephrase |

## Terminology and spelling decisions (corpus-wide rulings needed)

| Question | Evidence | Options |
|---|---|---|
| American or British spelling | Corpus is British (organisation, artefact, customise) but "knowledge organization" appears in [spec 2](../spec/02-taxonomy-model.md) and [spec 7](../spec/07-distribution-and-federation.md); "judgment" and "judgement" both occur ([spec 2](../spec/02-taxonomy-model.md), [spec 5](../spec/05-ai-integration.md)). STE rule 1.14 prescribes American unless a directive says otherwise | Rule in CLAUDE.md either way, then sweep once. A directive keeping British satisfies rule 1.14 |
| "register", two senses | [Spec 1](../spec/01-conceptual-model.md) uses *register* for the voice regime and for the register projection — one term, two concepts, a rule-1.11 violation in spirit, sharpened by the recent retirement of the standalone register concept | Rename the voice-regime sense (e.g. "voice") or accept the collision knowingly |
| "fails open" | [Spec 5](../spec/05-ai-integration.md) says routing "fails open" meaning it stays silent below the confidence threshold; conventional security usage would call that failing closed. The usage is consistent with design principle 7 in [spec 0](../spec/00-vision-and-scope.md) | Keep as project vocabulary (documented in principle 7) or align with conventional usage |
