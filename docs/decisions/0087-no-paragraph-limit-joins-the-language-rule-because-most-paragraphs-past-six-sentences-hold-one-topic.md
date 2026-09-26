---
id: HW-DR-0087
status: current
status_since: 2026-09-26
summary: "The six-sentence paragraph defect is retired before it lands. 50 of 60 sampled paragraphs past six sentences held one topic, so the count misses what rule 6.5 asks, and no word count replaces it."
last_verified: 2026-09-26
title: "No paragraph limit joins the language rule, because most paragraphs past six sentences hold one topic"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  supersedes:
    - HW-DR-0069
  governs:
    - to: engine/crates/check/src/language.rs
      verified_revision: sha256:4ed3b112c0e9c08ba9264da27070a07ce958e666a85a085cb591131e52c01bac
  traces_to:
    - HW-DR-0005
---

# No paragraph limit joins the language rule, because most paragraphs past six sentences hold one topic

## Context

**[HW-DR-0069](0069-a-paragraph-limit-counts-sentences-under-the-language-rule-and-never-words.md) named the measurement that would reopen it.** That record landed the six-sentence limit of ASD-STE100 rule 6.6 as a fifth defect of `language.controlled.not_met`. It also named the one question that could retire the defect. The question is rule 6.5's: of the paragraphs past six sentences, how many hold one topic? Where most hold one, that record says that the count measures the wrong thing, and that the defect is retired rather than tuned.

**A build of the defect gave the population.** [PR #1175](https://github.com/headwater-ai/headwater/pull/1175) built it for [#903](https://github.com/headwater-ai/headwater/issues/903), with list items and quotations outside it, as HW-DR-0069 stated. Over this repository at commit `bfc0b24e`, it reported 830 new advisory findings. The denominator is 7,093 paragraphs outside lists and quotations, in the 394 documents that the `ste_house` regime binds. So 11.7 percent of those paragraphs run past six sentences. On the ten governed shelves alone, the figure is 768 of 6,667.

**The count is higher than the census in HW-DR-0069, for two reasons.** That census read 388 of 5,890 paragraphs at `8c730e57`, with a cruder sentence splitter than the one the engine uses. Since then, the prose cleanup has split long sentences into short ones, and each split adds a sentence to its paragraph. A verifier derived the 7,093 again with an independent Markdown parser, and no document gave a different count. The verifier also counted three flagged paragraphs by hand, and each count matched the engine.

**The sample met the reopening clause.** The method is the one in [HW-DR-0005](0005-voice-checking-depth.md): a seeded random sample, adjudicated by hand. `random.Random(903)` drew 60 of the 830 findings. The builder judged that 50 of the 60 hold one topic, which is 83 percent. A second reader judged 51 of 60, and the two readers agreed on 59 of 60. The second reading was not blind, because that reader saw the first labels before the paragraphs.

**The result does not rest on the borderline cases.** The second reader named five paragraphs that were near the line. Count every one of the five as more than one topic, and 47 of 60 still hold one topic under either reading. That is 78 percent, and it is still most of the sample.

**The paragraphs that fire are the house pattern that HW-DR-0069 described.** A bold lead sentence states one claim, and the sentences after it carry the claim. The 25-word sentence rule keeps each sentence short, so one claim takes more sentences. Some paragraphs are a list in sentence form, with one short sentence for each item. The verifier also found a false positive in the segmentation: an inline quotation of three short sentences counts as three.

## Decision

**The six-sentence defect does not join `language.controlled.not_met`.** Rule 6.6 exists to serve rule 6.5, and on this corpus the count does not find a second topic. About five findings in six ask an author to split a paragraph that holds one topic. The remediation that HW-DR-0069 wrote, to split the paragraph where its second topic starts, has no answer for those paragraphs. So the defect is retired, as HW-DR-0069 ruled for this result.

**The limit is not tuned.** A limit of seven or ten sentences fires less often, but it is still a count. The sample shows that the count does not read the topic. HW-DR-0069 refused that path before the measurement, and this record keeps the refusal.

**No word-count rule joins the check layer either.** HW-DR-0069 refused a word count for a paragraph, and its reasons stand. ASD-STE100 states no word count for a paragraph. The measured tails of the two counts coincide, so a word count reports the same paragraphs, and the sample reaches it too.

**The rest of HW-DR-0069 stands.** The paragraph in [spec 13](../spec/13-open-obligations.md) that raised the question is a hand-kept index, and [#835](https://github.com/headwater-ai/headwater/issues/835) owes the mechanism that replaces it. Identifier density stays refused as a proxy for a paragraph in the shape of a list. HW-DR-0069 carries the argument for both, and this record supersedes it without a change to either.

**One topic to a paragraph stays a check that a person makes.** The `ste-editor` skill tells an author to hold a paragraph to one topic and to six sentences, by hand. That instruction stays, and no finding stands behind it.

## Consequences

**No engine code moves.** `engine/crates/check/src/language.rs` keeps four defects, and its cache version stays where it is. PR #1175 closes unmerged, by the owner's ruling on #903. An adopter who declares ASD-STE100 with `profile: house` gets no paragraph finding. On this corpus, that is 830 advisory findings that nobody receives, and the sample says that about five in six of them are wrong.

**The evidence stays with the pull request.** PR #1175 carries the build, the counts and the sample result at commit `bfc0b24e`. A person can draw the same sample again from the findings of that commit with seed 903.

**The two lines of prose that HW-DR-0069 expected to go stale stay true.** The `ste-editor` skill says that nothing checks sentences per paragraph. `CLAUDE.md` lists no paragraph rule among the advisory rules. Neither line moves. The skill gains one sentence that names this measurement, beside the Q5 sample that it already reports.

**What reopens this: a detector that reads the topic rather than a count.** A detector that finds a second topic in a paragraph answers rule 6.5 directly. It reopens this record when an adjudicated sample, on the method of HW-DR-0005, shows that most of its findings hold more than one topic. A count of sentences, of words or of identifiers does not reopen it.
