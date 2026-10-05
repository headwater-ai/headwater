---
id: HW-DR-0105
status: draft
status_since: 2026-10-03
summary: "Proposed, not ruled: the embedding path is not routing's silence-only fallback, because it is never silent and nothing grades its recall. The collection goes on."
last_verified: 2026-10-03
title: "105 — Whether the embedding path becomes the silence-only fallback of routing, and what happens to the shadow collection"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0064
---

# 105 — Whether the embedding path becomes the silence-only fallback of routing, and what happens to the shadow collection

**This record is a proposal. The owner has not ruled on it.** It stays at `draft` until the owner rules, and nothing in it binds a build before then.

## Context

[HW-DR-0064](0064-q64-whether-intent-time-routing-gains-an-offline-embedding-path-in-shadow-mode.md) built an instrument and integrated nothing. Its last consequence says that whether the fallback tier ever fires is a second ruling, and that the ruling waits on the data. [#404](https://github.com/headwater-ai/headwater/issues/404) proposed the tier: when the deterministic route is silent for `NoDocumentReached` or `NoPurposeMatched`, offer what the embedding path finds. [#927](https://github.com/headwater-ai/headwater/issues/927) asks for this ruling, and for a ruling on what happens to the collection.

The collection period closed on 2026-10-02 at 500 joined person prompts. [The evaluation of the log at its bound](../evaluations/what-the-shadow-mode-routing-log-showed-at-its-bound.md) holds every figure below, and the command that printed it. Four figures bear on the ruling:

- **The embedding path is never silent.** Over 515 prompts under the one model digest, it offered documents on all 515. On the 102 prompts where the deterministic route was silent, it offered documents on all 102.
- **No floor on the score separates the two cases.** The top neighbor score on a silent line has a median of 0.3252. On a line where the route offered pointers, the median is 0.4127. The two ranges overlap from 0.1585 to 0.5498.
- **The log cannot grade recall.** The main thread of a session read a corpus document after only 11 of 587 joined prompts. On the 7 such prompts under the model digest, neither path offered a document that the session read.
- **The two paths agree on some document in 183 of 413 prompts** where both offered one.
- **A blind grade found the embedding offer on a silent prompt mostly noise.** [The grade of #1670](../evaluations/what-a-blind-grade-of-the-two-routing-paths-offers-showed.md) scored 71 silent prompts with two agent raters and no person. On 17 of 71 the path offered a document that a session would need, and 686 of its 710 documents were not such a document.

## Decision

**Proposed: the embedding path does not become the silence-only fallback tier.** The tier would fire on every silent prompt, because the path is never silent. That includes a prompt with no term at all, such as the one-letter prompt `b` in the log. The log holds no evidence that the documents it would offer are the ones a session goes on to read. Spec 5 permits embeddings as a fallback for a fuzzy lookup, and never as the authority. A fallback with no evidence of value fails the first half of that sentence.

**Proposed: the collection keeps writing, and nobody mines it again until a reading can grade recall.** The hook writes one line a prompt, and that line is the only record of when and why the route is silent. The embedding column stays, because a second reading needs it. A second reading is worth running only when it can attribute the reads of an agent thread to the person prompt that dispatched the agent. The main thread reads too little to grade either path.

## Consequences

**Spec 5's fallback sentence stands as written, and `route.rs` gains no tier.** This is the state that HW-DR-0064 left, so the proposal changes no code.

**The question opens again on one condition.** That condition is a reading of at least 58 silent person prompts, each followed by a read of a corpus document. The read can be on any thread of the session. Until then, a ruling for the tier rests on no measurement. The blind grade of #1670 measures usefulness and not reads, so it does not meet this condition, and it leaves the proposal standing.

**The owner may rule the other way.** A ruling for the tier needs a floor on the score first, so that the tier can stay silent. The figures above say that no single floor does that on this log, so the ruling would also need a second signal beside the score.
