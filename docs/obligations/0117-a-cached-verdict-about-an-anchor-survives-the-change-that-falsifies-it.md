---
id: OBL-repo-0117
status: current
status_since: 2026-08-14
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
    - SPEC-HW-check-layer
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

Nothing discharges this yet. Two answers are open and they are different sizes.

**The narrow answer is to refuse the key.** An instance whose target is an anchor is not keyable, on the branch that already exists for an input with no digest. It costs the evaluation of one instance per anchor edge on every run, and it is exact.

**The wider answer is to put the anchor state into the key.** Each resolver states a digest over what it resolved against. That is the bytes of a file for the source tree, and the pin for a snapshot. A snapshot supplies one already, because a pin is a digest over the artifact. The source tree supplies none. A digest over every path an anchor could name is a walk of the repository rather than of the corpus.

**What no fixture here shows.** The cache suite holds the invariant that a cached run and an uncached run agree. It holds it over the fixture corpus, which declares no anchor edge whose target moves between two runs. So the suite passes and the property does not hold. The measurement above is therefore a hand run rather than a case in a suite. A fixture that moves an anchor target between two runs is what would have caught it.
