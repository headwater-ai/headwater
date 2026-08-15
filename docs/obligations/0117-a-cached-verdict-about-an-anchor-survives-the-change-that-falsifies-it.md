---
id: HW-OBL-0117
status: discharged
status_since: 2026-08-15
summary: "The read set of an edge instance holds the declaring document, so a target outside the corpus is in no key and a cached run reports what an uncached run does not."
last_verified: 2026-08-14
title: "A cached verdict about an anchor survives the change that falsifies it"
provenance:
  warrant: proposed
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-check-layer
---

# A cached verdict about an anchor survives the change that falsifies it

## Context

[Spec 12](../spec/12-check-layer.md#the-read-set-and-what-a-merge-does-to-a-verdict) states the rule this record is about. "The key is a hash of exactly those inputs, and a key that omits one is a correctness bug." The same section names the shape of the failure. "A cache that serves a verdict across that edit is the correctness bug a complete key exists to prevent."

`relation.target.unresolved` is edge-scoped, and the read set of one instance is the document that declared the edge. The target document joins it where the target is a document. A target that is not a document contributes no input at all. A read set is a list of corpus paths, and an anchor names something outside the corpus. So the key over an instance about an anchor names no byte of the thing the anchor points at.

## Obligation

**A run with the cache on reports a verdict that the tree in front of it falsifies, and nothing says so.** The cache is on unless a caller passes `--no-cache`, so this is what a default run does.

The measurement is a hand run on 2026-08-14 over a repository built for it. It carries one edge onto a committed snapshot and one `governs` edge onto a path in the source tree. Both targets are then removed. The first run writes the cache and the second reads it:

    $ headwater check          # the snapshot lost the item, and the file is deleted
      0 findings of relation.target.unresolved
      17 served from cache, 0 evaluated

    $ headwater check --no-cache
      2 findings of relation.target.unresolved
        `SPEC-HAND-one` declares `audited_by: 12345`, and the snapshot is not the pinned artifact
        `SPEC-HAND-three` declares `governs: src/service.rs`, and no such path is in the source tree

Both resolvers are reached, so this is a property of the read set rather than of either one. The `governs` case has been reachable since the source-tree resolver shipped, and no record held it. The snapshot case arrives with `headwater_import::anchors`.

**What the engine already does for the same problem, one input over.** `Cache::key` returns nothing for an instance whose input carries no digest. The comment on that branch states the reason. "The result cannot be keyed on a hash that does not exist, and inventing one is the correctness bug spec 12 names." An anchor target is the same case, and it never reaches that branch, because the read set of the instance never held the target.

**What this costs a reader.** A green run over a corpus whose edges point at nothing is the exact outcome [`relation.target.unresolved`](../spec/12-check-layer.md#two-phases-and-why-the-order-matters) exists to prevent. The commit gate runs `headwater check --strict` with the cache on, so a deletion that breaks an anchor passes the gate on the commit that makes it.

## Discharge

**This record is discharged, and the key of an edge instance names what a resolver said about its target.** `Target::resolution` is the binding in full, and `Cache::key` writes it on a line of its own beside the identity of the edge.

**The two answers were weighed and the wider one won at a narrower grain.** The record proposed a digest over what a resolver resolved *against*. It rejected that for the source tree, because a digest over every path an anchor could name is a walk of the repository. What an instance read is smaller. It read the answer a resolver gave about one string, and the resolvers run in phase A on every run. So that answer is in hand when the key is computed, and the measured cost of naming it is nothing.

**The refusal was the other candidate, and the difference between them is a number no gate reports.** `Cache::key` returns nothing for an input with no digest, and an anchor edge could join that branch. It reaches the same verdicts. It also makes every anchor instance permanently unkeyed. Neither the byte-identity differential nor the instance count can see that. The first compares verdicts, and the second counts instances that ran either way, so the cache counters are what separate the two. Over the fixture tree the refusal moves the unkeyed count from 64 to 67. It evaluates the moved anchor by dropping its key, where the component that ships evaluates it by changing one.

**The measurement, repeated on the branch on 2026-08-14.** The same hand run over this repository, which declares thirteen anchor edges, reports the finding on the cached run:

    $ headwater check          # the file a `governs` edge names is removed
      relation.target.unresolved (OB-REL-4): `HW-SPEC-ai-integration` declares
        `governs: .claude/hooks/lib.sh`, and that target `code_path`: no
        `.claude/hooks/lib.sh` in the source tree

**No number of this corpus moves, and the cache accounting moves by one instance.** A warm run serves 2520 of 2703 instances before the change and 2520 after it, and 27 findings are reported either way. The run over the tree that lost the file serves 2519 and evaluates 1. So the invalidation is exact rather than broad, which is what a refusal could not have been.

**What the fixture set gained.** `a_moved_anchor_target_is_not_served_from_the_entry_before_it` writes one edge onto a path outside the corpus, runs, removes the path, and runs again. It was written before the fix and it failed on the assertion named for it, with every other case in that file passing. It also fails on the refusal, on the assertion that no instance lost its key. The fixture taxonomy declares the anchor kind and the relation. The tree under `fixtures/check/` declares no edge onto either, so the recorded reports hold what they held.
