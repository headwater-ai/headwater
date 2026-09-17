---
id: HW-DR-0071
status: current
status_since: 2026-09-17
summary: "A route does not read headings and offers no anchor inside a document. A measured heading surface never reached spec 02 or spec 03 for the task that raised the question. At every threshold, function words reached wrong anchors."
last_verified: 2026-09-17
title: "A route offers a document and never a heading inside it"
relations:
  governs:
    - engine/crates/query/src/route.rs
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
---

# A route offers a document and never a heading inside it

## Context

[Issue #915](https://github.com/headwater-ai/headwater/issues/915) asks whether a route may offer a heading anchor inside a document, and which documents qualify. [HW-DR-0070](0070-the-matched-purposes-take-turns-at-a-route-budget-and-each-pointer-states-what-reached-it.md) rules on the other half of the issue.

The case is the task "Decide which shelf a new document belongs on, and where its identifier comes from." Two sections answer it. The "Kind resolution" section of `docs/spec/02-taxonomy-model.md` answers the shelf half. The "Identifiers" section of `docs/spec/03-authoring-and-lifecycle.md` answers the identifier half. Each summary covers a whole spec part, and neither names a shelf or an identifier. [Spec 5](../spec/05-ai-integration.md#intent-time-routing) fixes the read set as summaries, facets, relations and code-path anchors. The issue says that only a heading surface reaches the answer. It also says that a heading is a weaker signal than a summary.

This record tests that claim before it rules. A branch built the route with a heading surface. It ran over this repository on 2026-09-17, with the turns of HW-DR-0070 in place.

## What the measurement found

**The surface.** A document qualified where its kind declares no section contract. A contract heading such as "Context" is the same on every document of its kind, so it separates nothing. The route read the level-two and level-three headings of each qualifying document. The best heading added to the score of its document, and the pointer carried the anchor of that heading.

**First, with the weights that summaries use.** A term separates documents where at most half of the corpus carries it. Headings in this corpus are long, and most of their words are function words. The 2,726 level-two and level-three headings under `docs/` hold "the" 543 times, "and" 288 times and "what" 263 times. So function words reached headings. The first task offered `docs/spec/12-check-layer.md#four-things-stop-a-sweep-from-gating-and-none-of-them-is-a-rule-that-somebody-keeps`, which "where", "comes" and "from" reached. It also offered an evaluation by the heading "What this fixes for Q17, which it does not decide".

**Second, with a term rare among headings.** A term could reach a heading only where few headings carry it. That rule needed a threshold, and it also needed a plural fold, because the heading "Identifiers" does not hold the task term "identifier". Four thresholds ran: one heading in 200, 100, 50 and 25. The result was the same at all four:

- Neither spec 02 nor spec 03 reached the default five for the first task. Spec 12 took the `behavior` slot on its summary, which "where it comes from" reaches.
- Wrong anchors reached the five. "decide", "which" and "on" reached the heading "What decides it" in `docs/evaluations/language-choice.md`. At one in 25, "where" reached "Where the LLM coherence sweep fits" in spec 12.
- One anchor was right. For the second task of the issue, `docs/spec/12-check-layer.md#severity-is-the-checks-posture-is-the-controls` answers "what its severity is".

The spec 02 heading "Kind resolution" shares no term with the task at any threshold. The spec 03 heading "Identifiers" shares one term only through an English plural fold.

## Decision

**A route offers a document and never a heading inside it.** The read set in spec 5 stays as it is. No kind qualifies.

The reason is the rule in spec 5 that a wrong pointer costs more than a missing one, because an agent follows it. A heading anchor is a stronger claim than a document pointer. It tells an agent where to read, and not only what to open. On this corpus the heading surface made that claim wrongly more often than rightly. It never reached the case that raised the question.

Two parts of the build would each need a ruling of their own, and neither has one:

- **A threshold on heading rarity.** No threshold rests on a declaration. Each one tested here was chosen after the corpus was read, and four of them gave the same miss.
- **A plural fold.** The engine holds no word list for one language. `route.rs` gives that as the reason that a term most of the corpus carries weighs nothing. A fold for English plurals would be the first rule of that kind.

## Consequences

The answer to the first task is still out of reach of the route. That gap is in how the corpus names its topics, and not in how many surfaces the route reads. Spec 02 answers "which shelf" under a heading that says "Kind resolution". A lexical router cannot join those words, and no weighting of headings changes that.

Two ways remain open, and this record does not choose between them:

- **An authored cue.** Spec 5 already makes the summary the scent facet. A spec part that answers a common task under an unexpected name can say so in its summary, at the cost of a longer summary.
- **A meaning-based lookup.** [HW-DR-0064](0064-q64-whether-intent-time-routing-gains-an-offline-embedding-path-in-shadow-mode.md) collects a shadow-mode log for an embedding path, and [#819](https://github.com/headwater-ai/headwater/issues/819) builds it. That path reaches no agent until a second ruling. The first task is a case that its comparison can report on.

A later proposal to read headings has to answer the two findings above. It also has to show a measurement over a corpus that did not set its threshold.
