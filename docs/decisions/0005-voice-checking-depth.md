---
id: HW-DR-0005
title: Q5 — Voice checking depth
status: current
status_since: 2026-08-11
last_verified: 2026-08-11
summary: Voice checking stays lexical, with a curated pattern set, a per-category posture and a reasoned escape hatch.
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-EVAL-what-a-check-can-know
---

# Q5 — Voice checking depth

## Context

One [evaluation](../evaluations/what-a-check-can-know.md) settles this with [Q21](0021-terminological-succession-and-validity-under-merge.md), because a retired-term rule is a lexical rule and the two records share every problem. The leaning survives. What decides it is not what the open-question entry expected, and the measurement that shows this was available in the repository all along.

**The measurement, because this project ran a lexical checker on its own specification.** `tools/ste-lint.py` reported 0 errors and 613 warnings over 14 files, 5,432 sentences and about 82,000 words. That script is not in the tree. The check layer carries its lexical rules and none of the three that the sample below measures. This paragraph is the reading that settled the question. A hand-adjudicated sample gives three false-positive rates, under the two labels that [spec 4](../spec/04-assurance-model.md#suppression) already declares. The passive rule reaches 8% (60 of 449 sampled), the auxiliary rule 56% (25 of 54), and the progressive rule 83% (a census of all 12). The history adds the landing cost of a blocking rule. Of 58 sentence-length errors on the first run, 32 were defects in the sentence splitter rather than long sentences.

**The errors do not come from where the entry assumes.** Three sources, in order of size. Segmentation and span, which decides what is a sentence and which text belongs to the author. Part of speech, which decides whether the matched word is the verb that the rule assumed. And the lexicon, which produced approximately no errors across four rules and one landing.

**So the classifier is refused on aim rather than on accuracy.** It attacks the smallest of the three sources. It does not touch segmentation, and it does not improve a lexicon that is already right. The trigger to reopen is named: a voice category with a mechanical remediation, whose measured errors come mostly from part of speech.

## Decision

Voice checking is lexical. The curated pattern set, the per-category posture and the reasoned escape hatch all stand.

## Consequences

**The largest correction is to the revisit condition.** "Measured false-positive data" names an instrument that cannot answer this question. The passive rule measures 92% precise on its sample, 449 findings stand, and not one of them should block. What decides posture is the [fixability bar](../spec/12-check-layer.md#fixability): a category may block only when its remediation is mechanical and total. A category whose remediation is a rewrite is permanently advisory. [Spec 4](../spec/04-assurance-model.md#where-promotion-cannot-finish) now states that as a general rule, beside the case where promotion is skipped.

**And the instrument is worse than imprecise. It is empty.** Against 613 advisory findings this corpus holds four escape hatches, so the suppression-derived rate has almost no denominator. That is spec 4's stated blind spot in its extreme form. The adjudicated sample is therefore the only instrument that reaches an advisory rule, and the sample above is one.

**Two obligations land on the parser rather than on the rules** ([spec 3](../spec/03-authoring-and-lifecycle.md#what-a-lexical-rule-gets-wrong-and-where-posture-comes-from)). A voice check reads author-owned text, so quotations, code, citations and generated blocks are outside every voice rule by construction. And sentence segmentation joins the [correctness roots](../spec/12-check-layer.md#the-correctness-roots), because that is where the errors were.

**The [principle 4](../spec/00-vision-and-scope.md#design-principles) exception does not reach here.** A voice finding that fires wrongly is visible and cheap. One that fails to fire costs a sentence that a later reader or a later run still catches. Both error classes recover, so the ordinary promotion path applies and only the fixability bar stops it.
