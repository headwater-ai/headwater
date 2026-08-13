---
id: OBL-repo-0017
title: "Publishing the read set skips a re-run on two of the eighteen merges where the question is live"
status: current
status_since: 2026-08-11
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
    - DR-repo-0021
---

# Publishing the read set skips a re-run on two of the eighteen merges where the question is live

## Context

[Q21](../spec/09-decisions.md#q21--terminological-succession-and-validity-under-merge) rules terminological succession and validity under merge. Publishing the read set should let a gate skip a full re-run on most merges.

## Obligation

The instrument is the fraction of merges whose read set the other side never touched.

## Discharge

It ran. A run now publishes the union of its inputs, so the fraction is computable over the history of this repository. Across the 58 merge commits of `main` the mainline had not moved past the merge base in 40. No verdict was at risk in any of those. Eighteen carry a window. In two of the eighteen the mainline moved nothing that the read set holds, so a gate skips the re-run. In sixteen it did.

The claim holds at 42 of 58 merges and fails at 2 of the 18 where the question is live. The reason is structural. Two Shape rules generate over every kind, so the union is every classified document and it can exclude almost nothing. What decides a merge is the reach of the rule set rather than the count of the corpus-scoped checks that [spec 12](../spec/12-check-layer.md#the-read-set-and-what-a-merge-does-to-a-verdict) calls the barriers.

**Two of eighteen is the reading of the comparison over listed inputs, and the barrier rule sits above it.** [Spec 12](../spec/12-check-layer.md#the-read-set-and-what-a-merge-does-to-a-verdict) voids a verdict that rests on the extent of the census, because a list of members states no extent. `identifier.claimed_twice` is corpus-scoped, and it puts one instance in every run of this corpus. So every read set this corpus publishes carries one `barrier` line. A gate therefore carries none of the eighteen, and the fraction for this corpus is zero. [OBL-repo-0102](0102-two-documents-state-what-voids-a-verdict-and-they-do-not.md) carries the ruling, and the count of barriers is the number that decides this record after all.
