---
id: HW-OBL-0017
title: "Publishing the read set skips a re-run on two of the eighteen merges where the question is live"
status: current
status_since: 2026-08-11
waiting_on: build
last_verified: 2026-08-14
summary: "Q21 claims that a published read set lets a gate skip a full re-run on most merges, and the measurement came out against it."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0021
---

# Publishing the read set skips a re-run on two of the eighteen merges where the question is live

## Context

[Q21](../spec/09-decisions.md#q21--terminological-succession-and-validity-under-merge) rules terminological succession and validity under merge. Publishing the read set should let a gate skip a full re-run on most merges.

## Obligation

The instrument is the fraction of merges whose read set the other side never touched.

## Discharge

It ran. A run now publishes the union of its inputs, so the fraction is computable over the history of this repository. Across the 58 merge commits of `main` the mainline had not moved past the merge base in 40. No verdict was at risk in any of those. Eighteen carry a window. In two of the eighteen the mainline moved nothing that the read set holds, so a gate skips the re-run. In sixteen it did.

The claim holds at 42 of 58 merges and fails at 2 of the 18 where the question is live. The reason is structural. Two Shape rules generate over every kind, so the union is every classified document and it can exclude almost nothing. What decides a merge is the reach of the rule set rather than the count of the corpus-scoped checks that [spec 12](../spec/12-check-layer.md#the-read-set-and-what-a-merge-does-to-a-verdict) calls the barriers.

**Two of eighteen is the reading of the comparison over listed inputs, and the barrier rule sits above it.** [Spec 12](../spec/12-check-layer.md#the-read-set-and-what-a-merge-does-to-a-verdict) voids a verdict that rests on the extent of the census, because a list of members states no extent. `identifier.claimed_twice` is corpus-scoped, and it puts one instance in every run of this corpus. `lifecycle.deletion.not_permitted` is the second barrier and it puts one more, so this record read two `barrier` lines on every read set this corpus published on 2026-08-14. One was always enough, and a corpus that carries more barriers than two reaches the same answer sooner. A gate therefore carries none of the eighteen, and the fraction for this corpus is zero. [HW-OBL-0102](0102-two-documents-state-what-voids-a-verdict-and-they-do-not.md) carries the ruling, and the count of barriers is the number that decides this record after all.

**What happens next is a statement rather than a further run, which is why this record reads `build`.** The instrument ran and the reading is stable, because a barrier line is structural for any corpus whose rule set generates over every kind, and the count of them decides nothing that one does not. [Spec 12](../spec/12-check-layer.md#the-read-set-and-what-a-merge-does-to-a-verdict) states the claim about skipping a re-run and does not state that reading. What discharges this record is the qualification in spec 12, and no measurement of this corpus adds to it.
