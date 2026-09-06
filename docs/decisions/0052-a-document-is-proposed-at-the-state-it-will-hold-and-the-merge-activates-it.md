---
id: HW-DR-0052
status: current
status_since: 2026-09-06
summary: "An author writes the state a document will hold once it lands, and the merge activates it. The state facet answers to the merge, the way HW-DR-0034 made the warrant facet answer to it."
last_verified: 2026-09-06
title: "A document is proposed at the state it will hold, and the merge activates it"
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0034
    - HW-OBL-0030
---

# A document is proposed at the state it will hold, and the merge activates it

## Context

[HW-DR-0034](0034-q34-whether-acceptance-means-merged-to-main-and-what-an-agent-may-write-before-that.md) rules that acceptance is the merge onto `main`. A provenance block on a branch states a proposal, and the merge decides whether the stamp was true. That ruling reaches the provenance block and no other part of a document.

**The state facet had no such ruling, and the two facets moved apart.** `headwater new` writes the opening state of the regime the kind binds, which is `draft`. No document, no skill, no hook and no rule then says what moves a document off that state, or when. So one event carries a declared meaning for `warrant` and no meaning at all for `status`.

**The corpus measured the gap on 2026-09-06.** Of 277 typed documents, 66 stand at `draft`. [HW-DR-0047](0047-how-the-two-halves-of-the-site-share-one-host.md) and [HW-DR-0048](0048-the-served-sitemap-is-derived-from-the-served-directory.md) are merged, deployed and serving requests. Both stand at `draft`, whose declared guidance reads "the document is being written or argued over, and nothing may rely on it". The obligation shelf splits at `HW-OBL-0126`: records 0001 to 0125 stand at `current` or `discharged`, and records 0126 to 0161 stand at `draft` bar two. One kind, one shelf, one meaning, and no record of a change in practice.

Four whole shelves stand at `draft` entire. They are the nineteen interface contracts, the four probes, the one tutorial whose fixture suite blocks CI, and the requirement beside its acceptance criterion. A second session met the same absence while writing two new records. It looked for what `draft` means on the decision shelf, and found nothing to copy but the deviation.

## Decision

**An author writes the state the document will hold once the branch lands.** For a document proposed as finished, that state is `current`. The merge activates it, the way the merge activates the acceptance stamp beside it.

**`draft` on a branch states that the document is not proposed as finished.** It is the honest value for work a reader is asked to comment on rather than rely on. It is not a placeholder that an author leaves because the scaffolder wrote it.

**The ruling is symmetric with HW-DR-0034 and needs no actor that promotes a state at merge time.** Nothing moves the value between the branch and `main`. The state a proposal carries is the state the merged document carries, and the merge decides whether that value was true.

**A document already on `main` at the wrong state is corrected rather than left.** The correction moves `status` and stamps `status_since` with the date of the correction, because that is the date the document entered the state.

## Consequences

**What a writer meets.** `headwater new` writes `draft`, because a regime opens at its initial state and a scaffolder reports the value it derived. The author moves the value before proposing the document. The authoring skill states this beside the paragraph that states the acceptance stamp, so a writer meets both readings of the merge in one place.

**What this corpus owes.** The 66 documents standing at `draft` are merged, and this ruling says what they should read. The change that carries this record corrects them and stamps each one with the date of the correction. The dwell each document spent at `draft` stays in the history rather than in the facet, which is the ordinary cost of a correction.

**What no rule reads.** [HW-OBL-0030](../obligations/0030-the-provenance-block-belongs-to-the-engine-and-nothing-states.md) records that no taxonomy declares the provenance block, so nothing reads the acceptance stamp. The state facet is declared, and `lifecycle.state.not_admitted` and `lifecycle.transition.not_permitted` both read it. Neither asks whether a merged document is finished, because neither has a merge to read. So this ruling rests on the process this repository runs rather than on a control the engine holds.

**What this makes reportable.** [#569](https://github.com/headwater-ai/headwater/issues/569) holds a rule that would report a live document resting on one at a state whose role is `initial`. That rule is worth having once a `draft` document is rare, and it is noise while two thirds of a shelf stands there. This ruling is what makes the population small enough for the rule to say something.

**What reopens this.** A kind whose documents are argued over on `main` for weeks would need `draft` to mean what it says on a merged document. No kind of this corpus works that way today.
