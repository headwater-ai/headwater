---
id: HW-EVAL-what-a-blind-grade-of-the-two-routing-paths-offers-showed
status: current
status_since: 2026-10-03
summary: "Two agent raters scored 4,929 offered documents blind. On 17 of 71 silent prompts an embedding neighbor scored 2, and the route's first pick scored 2 on 5 of 290 prompts."
last_verified: 2026-10-03
title: "What a blind grade of the two routing paths' offers showed"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0105
    - HW-DR-0064
    - tools/run/relevance-grade.sh
---

# What a blind grade of the two routing paths' offers showed

[HW-DR-0064](../decisions/0064-q64-whether-intent-time-routing-gains-an-offline-embedding-path-in-shadow-mode.md) logs two offers for each person prompt: the documents that the deterministic route offered, and the documents that the embedding path would have offered. [The evaluation of the log at its bound](what-the-shadow-mode-routing-log-showed-at-its-bound.md) counted how often each path speaks. It could not say whether what a path offered was of use. [#1670](https://github.com/headwater-ai/headwater/issues/1670) asked for a blind grade of that, and this document reports it.

## What was graded, and when

**No person checked any figure here.** The issue asked the owner to rate about 30 prompts blind, as a spot check on the agent raters. The owner waived that check on 2026-10-03. So no figure below was compared with the judgment of a person. The 30 prompts of `tools/run/relevance-grade/owner-sample.tsv` were drawn and never rated.

**The unblinded figures were seen before a threshold was fixed.** The stage that checked the issue before the build read the per-path figures from the committed record. No threshold for "useful" had been fixed at that time. So this document reports both thresholds, score 2 and score 1 or more, and it chooses neither.

The scores are on `main` at commit `eba83912`, which merged as pull request #1678. The grade ran on 2026-10-03 at 12:40:51 UTC, over the `sample.tsv` and `consensus.tsv` of that commit. Their sha256 digests are `fa01fc85…5690` and `b08388eb…0262`. One command prints every figure of the grade from a clone, and a rerun writes the same bytes:

    sh tools/run/relevance-grade.sh grade

It writes `tools/run/relevance-grade/grade.txt`. It reads the sample and the consensus, and nothing else. It refuses, with exit 4, before it writes a figure, when an offered document or a `missing` flag has no consensus value. The record has a value for each of its 5,290 items, and none is unresolved.

## The sample

`tools/run/relevance-grade/draw.txt` states the draw. It used seed 1670 and cut the log at 2026-10-03T04:42:21Z. 515 joined prompts were under the model digest `sha256:f342437e…8e44`. Each prompt went in one stratum:

| stratum | in the log | excluded | kept | drawn |
|---|---|---|---|---|
| silent: the route offered nothing | 102 | 31 | 71 | 71 |
| disjoint: both offered, no document in both | 230 | 0 | 230 | 230 |
| overlap: both offered, one document or more in both | 183 | 7 | 176 | 60 |

A stated rule excluded 38 prompts before the draw: 11 that were one bare slash command, and 27 that held no word to rate. `tools/run/relevance-grade/excluded.tsv` lists each one with its reason. The issue named "the 102 silent prompts", and 71 of them stayed after the rule. The sample is 361 prompts. It holds a digest of each prompt's text and never the text.

## The raters, and how far they agree

Two agent raters, `a` and `b`, scored the whole sample. Both were `claude-opus-5-5`. Each batch of 20 prompts went to a fresh context, with the batch file as its only input, in 19 batches for each rater. `tools/run/relevance-grade/raters.tsv` lists each batch. A rater saw one list per prompt, with each document once, and with no path, rank or score. A rater scored each document 0, 1 or 2 for whether a session that answers the prompt would need it. A rater also flagged `missing` when a needed document was not in the list.

`tools/run/relevance-grade/agreement.txt` holds the agreement:

| stratum | documents | exact agreement | quadratic-weighted kappa | kappa on `missing` |
|---|---|---|---|---|
| silent | 710 | 689 (0.9704) | 0.9236 | 0.7897 |
| disjoint | 3,408 | 3,278 (0.9619) | 0.8815 | 0.7123 |
| overlap | 811 | 781 (0.9630) | 0.9179 | 0.7802 |
| all | 4,929 | 4,748 (0.9633) | 0.8968 | 0.7442 |

The two raters differed on 227 items. A third agent pass, also `claude-opus-5-5`, scored each disputed item in 13 batches, and the consensus took the median of the three scores. 5,063 items were agreed and 227 were settled by the third pass.

**Both raters are one model.** So their agreement shows that the model gives the same answer twice. It does not show that two independent judges agree. Two independent judges could agree less than these two did.

## The figures, per path and per stratum

A document's consensus score is credited to each path that offered it. So a document in both offers counts once for each path. "Prompts with one at the threshold" counts the prompts on which the path offered at least one document at that score. The interval is Wilson 95%. No row pools the strata, because the overlap stratum is a draw of 60 from 176 and the other two are whole.

At score 2, "a session would need it":

| stratum | path | prompts offered on | documents | documents at 2 | prompts with one at 2 | Wilson 95% | first offered at 2 |
|---|---|---|---|---|---|---|---|
| silent | route | 0 of 71 | 0 | n/a | n/a | n/a | n/a |
| silent | embedding | 71 of 71 | 710 | 24 (0.0338) | 17 of 71 (0.2394) | 0.1552 to 0.3504 | 4 of 71 (0.0563) |
| disjoint | route | 230 of 230 | 1,108 | 15 (0.0135) | 15 of 230 (0.0652) | 0.0399 to 0.1048 | 1 of 230 (0.0043) |
| disjoint | embedding | 230 of 230 | 2,300 | 51 (0.0222) | 40 of 230 (0.1739) | 0.1304 to 0.2281 | 12 of 230 (0.0522) |
| overlap | route | 60 of 60 | 297 | 15 (0.0505) | 13 of 60 (0.2167) | 0.1312 to 0.3362 | 4 of 60 (0.0667) |
| overlap | embedding | 60 of 60 | 600 | 24 (0.0400) | 21 of 60 (0.3500) | 0.2417 to 0.4764 | 7 of 60 (0.1167) |

At score 1 or more, "a session would perhaps need it":

| stratum | path | documents at 1 or more | prompts with one at 1 or more | Wilson 95% | first offered at 1 or more |
|---|---|---|---|---|---|
| silent | embedding | 76 of 710 (0.1070) | 35 of 71 (0.4930) | 0.3800 to 0.6066 | 13 of 71 (0.1831) |
| disjoint | route | 83 of 1,108 (0.0749) | 67 of 230 (0.2913) | 0.2364 to 0.3531 | 12 of 230 (0.0522) |
| disjoint | embedding | 314 of 2,300 (0.1365) | 134 of 230 (0.5826) | 0.5180 to 0.6445 | 49 of 230 (0.2130) |
| overlap | route | 58 of 297 (0.1953) | 35 of 60 (0.5833) | 0.4573 to 0.6994 | 12 of 60 (0.2000) |
| overlap | embedding | 122 of 600 (0.2033) | 42 of 60 (0.7000) | 0.5749 to 0.8010 | 24 of 60 (0.4000) |

The consensus flagged a needed document as `missing` on 23 of 71 silent prompts (0.3239). It flagged one on 117 of 230 disjoint prompts (0.5087) and on 21 of 60 overlap prompts (0.3500).

**Most offered documents of both paths were not needed.** At score 2, the share of offered documents is between 0.0135 and 0.0505 in every row. The embedding path offers about ten documents a prompt, and the route offers about five. So the embedding path has more prompts with one useful document, and also more documents that a session would not need.

## The silent prompts: useful or noise

On a silent prompt the route offered nothing, and the embedding path offered ten documents on each of the 71. That is the case the fallback tier of [#404](https://github.com/headwater-ai/headwater/issues/404) would serve.

**The embedding offer on a silent prompt is mostly noise.** 686 of its 710 documents (0.9662) did not score 2, and 634 of 710 (0.8930) scored 0. On 17 of 71 prompts (0.2394) it offered one document or more that a session would need. On 35 of 71 (0.4930) it offered one that a session would perhaps need. Its first neighbor, which a tier would show first, scored 2 on 4 of 71 prompts (0.0563). On 23 of the 71 prompts, the raters said that a needed document was not in the list at all.

## What the grade decides

**For [HW-DR-0105](../decisions/0105-whether-the-embedding-path-becomes-the-silence-only-fallback-of-routing-and-what-happens-to-the-shadow-collection.md), the proposal stands, and the grade is evidence for it.** The issue set the test. An offer that is useful on most silent prompts reopens the record, and an offer that is mostly noise leaves it standing. Neither threshold reaches most of the 71 silent prompts. Score 2 reaches 17 of 71, and its Wilson interval ends at 0.3504, well below a half. Score 1 or more reaches 35 of 71, which is less than a half by one prompt. Its interval, 0.3800 to 0.6066, contains a half, and one changed score on one more prompt would make it 36 of 71. So at score 1 or more the grade cannot tell the share from a half. That reading does not show a useful offer on most silent prompts, and the score 2 reading shows that there is none. 634 of the 710 offered documents scored 0. HW-DR-0105 stays a draft. Only the owner rules on it, and this grade does not rule for the owner.

The grade does not meet or miss the reopening condition of HW-DR-0105. That condition is a reading of at least 58 silent prompts, each followed by a read of a corpus document. This grade measures usefulness by the judgment of a rater, and not reads. It graded 71 silent prompts, but no read follows them in the record.

**For [#1659](https://github.com/headwater-ai/headwater/issues/1659), one line of the plan grades against the route's first pick, and this grade makes that key weak.** The plan is `tools/probe/layer-campaign.spec` over `.headwater/probe.yml`. Each of its six discovery lines pools the rate of three probes. One of the three is the probe that [the evaluation of 2026-09-17](the-pointer-probe-grades-a-session-against-the-router-s-own-first-pick-so-a-correct-session-that-declines-a-wrong-pointer-fails.md) reads. Its expectation is the document that the router ranks first for its task, and no person judged that document to answer the task.

The route offered on 290 of the logged prompts. Its first pick scored 2 on 5 of the 290 (0.0172), and 1 or more on 24 of 290 (0.0828). So a first pick of the route is seldom a document that a session would need. A session that opens it passes that probe, and a session that declines it fails. The grade is over person prompts and not over the task of that probe. So it does not show that the probe's own target is wrong. It shows that a target from the router's rank alone carries little evidence of relevance. A reader of the campaign's discovery rate must keep that in mind. This is most important for an arm that changes whether a session sees the pointer.

This document changes no line of #1659's plan. The plan's tooling belongs to [#1472](https://github.com/headwater-ai/headwater/issues/1472), and a change to a probe's target is a ruling for the owner.

## What the grade cannot say

- No person checked a score, and both raters are one model. A systematic error of that model is in every figure.
- A score is a judgment of whether a session would need the document. It is not a record that a session read it.
- The overlap stratum is a draw of 60 from 176. Its figures are an estimate for the 176, and the other two strata are whole.
- The figures are over the person prompts of one owner's sessions in one repository, under one model digest.
