---
id: HW-DR-0069
status: draft
status_since: 2026-09-16
summary: "No word-length rule joins the check layer. The six-sentence limit of the standard the house regime declares lands as a fifth defect of the language rule, advisory permanently. The paragraph that raised the question is a hand-kept index, and a derived list repairs it where no lexical rule reads it."
last_verified: 2026-09-16
title: "A paragraph limit counts sentences under the language rule and never words"
relations:
  governs:
    - engine/crates/check/src/language.rs
  traces_to:
    - HW-DR-0005
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-fable-5-1
  activity: measure+draft
  evidence_basis: evidenced
---

# A paragraph limit counts sentences under the language rule and never words

## Context

The rendered page of [spec 13](../spec/13-open-obligations.md) carries paragraphs that run past a screen. The question was whether the 25-word sentence rule should gain a paragraph rule beside it.

`regimes.language.ste_house` in `.headwater/overlay.yml` declares `controlled: ASD-STE100` with `profile: house`, and `language.controlled.not_met` is what the engine holds that pair to. The rule reports four defects today: a sentence past 25 words, a semicolon in running prose, a contraction, and a British spelling. The standard the regime names states a paragraph rule of its own. Writing rule 6.5 asks for one topic a paragraph, and rule 6.6 caps a paragraph at six sentences. Neither one is a word count.

**This repository ran a paragraph detector once, and retired it unmeasured.** `tools/ste-lint.py` carried `paragraph-sentences` beside three voice detectors. [HW-DR-0005](0005-voice-checking-depth.md) adjudicated a sample of the three and never the fourth, and the script left the tree in `a39e3fb` with all four. The `ste-editor` skill records that nothing checks sentences per paragraph, and it asks an author to check that by hand. So the question is older than the page that raised it, and the one measurement it needed was never taken.

**The measurement, over the nine governed shelves on 2026-09-17, is a count over the tree at `8c730e57`.** It reads 5,890 paragraphs, with list items, tables, headings and fenced code left out. No check holds the figures below, and the build this record asks for is what would. 515 paragraphs run past 100 words, 60 past 150, 14 past 200, two past 300 and one past 500. 388 paragraphs run past six sentences, which is 6.6 percent, and the two measures pick the same tail. Of the 60 paragraphs past 150 words, two stay within six sentences, and 329 of the 388 past six sentences stay under 150 words.

**The paragraph that raised the question is one paragraph, and it is unlike every other one.** Under [*Design work that nothing blocks*](../spec/13-open-obligations.md#design-work-that-nothing-blocks), spec 13 holds a paragraph of 1,479 words and 96 sentences that names 21 obligation identifiers. It states where 93 items arrived from, one library entry at a time, and a list of 65 items follows it. The next longest paragraph in the corpus is 320 words, on the same page. At 15.4 words a sentence, not one sentence of the 96 breaks the sentence rule, and nine sentence findings stand on the whole page. [#835](https://github.com/headwater-ai/headwater/issues/835) already reports that page as drifted from the shelf it indexes, and it asks for a mechanism that cannot drift the same way again.

**The other thirteen paragraphs past 200 words are argument, and the sentence rule is why they are long.** Each carries between zero and three identifiers, between 11 and 18 sentences, and 14 to 19 words a sentence. [Spec 7](../spec/07-distribution-and-federation.md#the-migration-payload) holds one at *A facet that a kind starts to require*, and [spec 6](../spec/06-engine-architecture.md#cli) one at *This grammar is a statement of fact*. The house pattern is a bold lead sentence that states one claim, and then the sentences that carry it. 217 of the 388 paragraphs past six sentences open that way. A 25-word limit shortens each sentence and moves nothing out of the block, so the block grows one short sentence at a time.

**An advisory finding under this rule changes nothing today, and the figure is exact.** 221 sentence-length findings stand on the tree, `headwater check --strict` exits 0 over them, and no document carries an `allow` directive for the rule. That is the instrument [HW-DR-0005](0005-voice-checking-depth.md) called empty: findings with no denominator of adjudicated exceptions. The one advisory set anyone acted on is the 60 change-narration findings that [#605](https://github.com/headwater-ai/headwater/issues/605) filed. It moved because the issue named the adopter who reads the specification as the reader.

## Decision

**No word-count rule joins the check layer.** The standard the regime declares states no word count for a paragraph. The number would be this repository's own, chosen where the standard chose sentences. The measured tails coincide, so a word count reports no paragraph the sentence count misses, and it reports two it should not. And cutting words is the only remedy a word count can name. It does not repair a paragraph of 13 sentences at 17 words each.

**The six-sentence limit of ASD-STE100 rule 6.6 is the one paragraph measure the house profile admits, and it lands as a fifth defect of `language.controlled.not_met`.** It reads a block of `BlockKind::Paragraph` in author-owned text and counts the sentences the parser already segments. A count past six is the finding. A list item is outside it, because a list is the structure a long paragraph is split into. A quotation, a code span and a generated block are outside it by construction, as they are outside every language rule.

**Its severity is `warn`, permanently, because the remedy is a rewrite ([spec 3](../spec/03-authoring-and-lifecycle.md#what-a-lexical-rule-gets-wrong-and-where-posture-comes-from)).** `CT-LANG-1` in the base package states that posture for every defect of this rule. [Spec 12](../spec/12-check-layer.md#fixability) puts the bar per finding rather than per rule, so no second rule identifier and no taxonomy declaration moves.

**The remediation names the topic and never the count.** Rule 6.6 exists to serve rule 6.5, and a count is what an engine can read of a topic. So the finding says to split the paragraph where its second topic starts. Or it says to move the sentences that do not carry the first one into their own paragraph. A finding that said to cut the paragraph to six sentences would be met by joining sentences, and the sentence rule then reports the join.

**The paragraph that raised the question is not a case for this defect or for any rule.** It is a hand-kept narrative inside a hand-kept index, and the index is what #835 owes a mechanism. Each obligation record already carries its context and an edge to what produced it. So the narrative is a second copy of what the shelf states. When the register derives its lists from the shelf, that paragraph becomes a list or becomes nothing, and no lexical rule takes part. This record asks #835 to carry one more done-when: the class narrative does not survive as a paragraph.

**Identifier density is refused as a proxy for a list-shaped paragraph.** It separates exactly one paragraph of 5,890, and that paragraph leaves the tree with #835. A rule that reports zero from the day it lands is a saturated rule, and nothing distinguishes it from a rule that works.

**What reopens this: an adjudicated sample of the findings, on the method of HW-DR-0005.** The question the sample answers is rule 6.5's: of the paragraphs past six sentences, how many hold one topic. Where most hold one, the count measures the wrong thing, and the defect is retired rather than tuned. A limit tuned in numbers until it fires less often is the word count this record refused, in a different unit.

## Consequences

**A build lands the defect, and this record does not.** It moves `engine/crates/check/src/language.rs` and the module comment that lists the defects. It adds a fixture that reports a seven-sentence paragraph and stays silent on six, on a seven-sentence list item and on a seven-sentence quotation. It moves the cache version, because a warm cache serves the old verdict of a widened rule. It widens what `profile: house` reports for every adopter who declares that pair. That adopter is the reader outside this repository, and one who declared the standard gets the paragraph rule the standard states.

**About 388 findings arrive, nothing blocks, and the hook prints none of them.** The gate runs `--strict`, which fails on an error, and this defect is never one. A person who runs `headwater check` reads them, and the count is the population the sample above reads. What the 221 sentence findings show is that a finding nobody names a reader for stays on the tree. So the sample is the next step and not a later one. The build that lands the defect names the sample as its verification, and the share of one-topic paragraphs is the figure it reports.

**Two lines of prose go stale on landing, and no check reads either.** The `ste-editor` skill says that nothing checks sentences per paragraph, and `CLAUDE.md` lists what is advisory without a paragraph in the list. The skills fixtures assert engine-verb claims only, so both lines are hand-owed by the build.

**Spec 13 owes this record nothing beyond what #835 carries.** That page holds 34 of the 388 paragraphs past six sentences. 13 stand under *Design work that nothing blocks* and 16 under *What else each decision left open*. The derived register removes the class narratives, and the per-decision narratives that remain are the ordinary population of the defect. The two pages beside it with the largest counts are spec 7, with 34, and spec 6, with 19. Those are argument rather than index, they are the paragraphs the defect is for, and no restructuring of a register touches them.

**This record holds its own paragraphs to the rule it states.** Six sentences each, and a reader checks it here before checking it anywhere else.
