---
id: HW-OBL-0118
status: current
status_since: 2026-08-14
waiting_on: ruling
summary: "The read set artifact lists documents and files, so an anchor target is on no line and a gate cannot see it leave the tree."
last_verified: 2026-10-15
title: "The published read set names no anchor, so a gate decides nothing about one"
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft+revise
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-check-layer
---

# The published read set names no anchor, so a gate decides nothing about one

## Context

[Spec 12](../spec/12-check-layer.md#the-read-set-and-what-a-merge-does-to-a-verdict) gives the read set two uses. The first use is the cache key. [HW-OBL-0117](0117-a-cached-verdict-about-an-anchor-survives-the-change-that-falsifies-it.md) was the anchor gap in the key, and it is discharged: the key of an edge instance names the binding that the resolver returned. [#1029](https://github.com/headwater-ai/headwater/issues/1029) also put the identity of the compiled rule set into the key, through `headwater_check::rules_digest`.

The second use is the artifact that `headwater check --read-set` writes and that `headwater gate` reads back. This record is about the second use. A key is computed inside one run, where a resolver has already answered. A later process reads the artifact over a later tree, and no resolver has answered there.

## Obligation

**A gate compares one hash for each line of the artifact, and no line of the artifact names an anchor.** `ReadSet::render` writes `lock`, `clock`, `barrier`, `windowed`, `change-scoped`, `version` and `input` lines. An `input` line names a document or a file that a run read. The answer of a resolver is not such a file, so the target of an anchor is on no line. A gate hashes nothing that moves when that target leaves the tree. The `Reason` enum in `engine/crates/check/src/gate.rs` holds every reason a verdict does not carry, and no reason names an anchor. [#1345](https://github.com/headwater-ai/headwater/issues/1345) added imports snapshots as file lines and the reason `escaped`, and neither one names an anchor.

**A barrier voids every gate over this corpus before the input comparison starts.** A corpus-grained rule, such as `identifier.claimed_twice`, runs on every run of this repository and writes a `barrier` line. `Verdict::carries` is then false. A hand run on 2026-10-15 shows it:

    $ headwater check --read-set /tmp/read-set
    $ grep -c 'hooks/lib.sh' /tmp/read-set
    0
    $ headwater gate --read-set /tmp/read-set
    gate the verdicts this run reached do not carry to this tree
      identifier.claimed_twice is a barrier: its verdict is a predicate over the extent of the census, and a list of members states no extent

`.claude/hooks/lib.sh` is the target of an anchor edge in this repository, and the artifact does not list it. The gap is real, and no gate can reach it while a barrier voids each answer.

**A per-rule answer is what makes the gap reachable.** Spec 12 says that "a per-instance answer wants a per-instance artifact, and this is not one". Take a gate that voids the barrier rules alone and carries the verdicts of every other rule. That gate reports that a `governs` verdict survives a tree that deleted the file the edge names. `relation.target.suspect` reports the same drift, but it reads a run and not this artifact.

**Two shapes can close the gap, and choosing one is a design decision.**

- **The narrow shape is a line of its own.** The artifact gains a keyword for a target that no path names, with the anchor kind, the resolver and the binding. A gate that meets the keyword reports that it cannot decide, in the terms it already uses for an input with no hash. It costs one line for each anchor edge, and it decides nothing new.
- **The wide shape is a digest from the resolver.** Each resolver states a digest over what it resolved against. A gate runs the resolvers again over the later tree and compares the digest. It decides the question rather than refusing it. But a gate then reads the tree and not a list, and spec 12 fixed the list as the whole test.

## Discharge

The question for a person: **does the read set gain an anchor line that a gate refuses, or a resolver digest that a gate compares?** The first is the narrow shape, and the second is the wide shape.

A precondition comes before either shape can show a failing fixture. `headwater gate` must answer per rule, so that a barrier voids its own rules and not every verdict. Until then, no case in a suite can fail on an anchor, and the hand run above is the only evidence.

This record is discharged when spec 12 states the ruling and the chosen shape ships. A gate that answers per rule then fails a fixture whose anchor target left the tree.
