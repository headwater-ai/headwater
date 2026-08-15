---
id: HW-OBL-0118
status: current
status_since: 2026-08-14
summary: "The read set artifact holds one line per document, so an external anchor is on no line and a gate that compares listed hashes cannot see the target of one leave the tree."
last_verified: 2026-08-14
title: "The published read set names no anchor, so a gate decides nothing about one"
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

# The published read set names no anchor, so a gate decides nothing about one

## Context

[HW-OBL-0117](0117-a-cached-verdict-about-an-anchor-survives-the-change-that-falsifies-it.md) is the same gap in the cache key, and it is discharged. The key of an edge instance names the binding that the resolver returned, so two states of one anchor no longer share an entry.

The key is one of the two uses that [spec 12](../spec/12-check-layer.md#the-read-set-and-what-a-merge-does-to-a-verdict) gives the read set. The other one is the artifact that `headwater check --read-set` writes and that `headwater gate` reads back. This record is about the second use, and the fix for the first one does not reach it. A key is computed inside one run, where a resolver has already answered. An artifact is read by a later process over a later tree, which has no answer of any resolver in front of it.

## Obligation

**A gate compares one hash for each line of the artifact, and no line of the artifact names an anchor.** `ReadSet::render` writes one `input` line for each document that an instance read. The answer of a resolver is not a document, so an anchor target is on no line. A gate therefore hashes nothing that moves when that target leaves the tree.

The measurement is a hand run on 2026-08-14 over this repository, which declares thirteen anchor edges:

    $ headwater check --read-set /tmp/read-set
    $ grep -c 'hooks/lib.sh' /tmp/read-set
    0
    $ mv .claude/hooks/lib.sh /tmp
    $ headwater gate --read-set /tmp/read-set
    the verdicts this run reached do not carry to this tree
      identifier.claimed_twice is a barrier: its verdict is a predicate over the extent of the census, and a list of members states no extent

**The barrier is what hides it, and the barrier is not a fix.** A corpus-scoped instance exists on every run, because `identifier.claimed_twice` takes one target that no declaration selects. So every artifact carries a `barrier` line, and `Verdict::carries` is false on every gate over this corpus. The answer is void before the input comparison begins. The limit above is real and it is unreachable through this verb today.

**What makes it reachable is a per-rule answer.** Spec 12 says that "a per-instance answer wants a per-instance artifact, and this is not one". Take a gate that voids the barrier alone and carries the verdicts of every rule that is no barrier. It reports that a `governs` verdict survives a tree that deleted the file the edge names. [#82](https://github.com/headwater-ai/headwater/issues/82) is the drift report, and it reads this artifact.

## Discharge

Nothing discharges this yet, and two shapes are open.

**The narrow shape is a line of its own.** The artifact gains a keyword for a target that no path names, carrying the anchor kind, the resolver and the binding. A gate that meets that keyword reports that it cannot decide, on the terms it already uses for an input with no content hash. It costs one line for each anchor edge and it decides nothing new.

**The wider shape is a resolver that states a digest over what it resolved against.** A gate re-runs the resolvers over the later tree and compares that digest. It decides the question rather than refusing it, and it makes a gate read the tree rather than a list. That list is the economy [spec 12](../spec/12-check-layer.md#the-read-set-and-what-a-merge-does-to-a-verdict) fixed as the whole test.

**What no fixture here shows.** `engine/crates/check/src/gate.rs` holds every reason a verdict does not carry, and an anchor is not among them. No case can fail on this while a barrier voids each answer first. So the measurement above is a hand run rather than a case in a suite. A gate that reports per rule is what makes one possible.
