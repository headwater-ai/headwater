---
id: HW-OBL-0118
status: discharged
status_since: 2026-09-23
waiting_on: build
summary: "The read set artifact holds one line per document, so an external anchor is on no line and a gate that compares listed hashes cannot see the target of one leave the tree."
last_verified: 2026-09-23
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

[HW-OBL-0117](0117-a-cached-verdict-about-an-anchor-survives-the-change-that-falsifies-it.md) is the same gap in the cache key, and it is discharged. The key of an edge instance names the binding that the resolver returned, so two states of one anchor take separate entries.

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

**What makes it reachable is a per-rule answer.** Spec 12 says that "a per-instance answer wants a per-instance artifact, and this is not one". Take a gate that voids the barrier alone and carries the verdicts of every rule that is no barrier. It reports that a `governs` verdict survives a tree that deleted the file the edge names. [#82](https://github.com/headwater-ai/headwater/issues/82) built the drift report as `relation.target.suspect`, and it reads a run rather than this artifact. So the finding is sound and a gate that carried it is the half this record still holds open.

## Discharge

**This record is discharged for the gap its own text last named as unowned.** The narrow and wide shapes above are about a different use of the read set, and this change does not reach them. The Context and Obligation sections are about the artifact `headwater check --read-set` writes and `headwater gate` reads back. That limit stands exactly as measured. A barrier voids every answer before the input comparison begins. Neither the narrow shape (a keyword for an anchor line) nor the wide shape (a resolver-stated digest read back by the gate) has been built. What discharges here is the postscript below: the debt [#855](https://github.com/headwater-ai/headwater/issues/855) filed into this record as unowned.

**[#855](https://github.com/headwater-ai/headwater/issues/855) widened the neighbourhood-scoped cache key to carry a bound anchor's resolution.** `over_edges` already carried one into the edge-scoped key. #855 named the gap it did not close: neither key carried the engine's own binary version. A verdict about a bound `check_rule` anchor could survive an engine upgrade that dropped, renamed or changed the rule it names. Only the lock digest and the rule's own `VERSION` moved the key, and the compiled `RULES` list entered neither. That gap is [#411](https://github.com/headwater-ai/headwater/issues/411)'s debt, filed here as intake for the product owner to triage: "no issue owns closing it yet."

**[#1029](https://github.com/headwater-ai/headwater/issues/1029) is that issue.** `Cache::key` now carries a seventh component: the identity of the compiled rule set. `headwater_check::rules_digest` hashes the sorted, joined text of `crate::RULES`. The value changes on an add or a remove and stands still on a reorder of the same set. `Cache::at` takes it as an argument on the terms `lock` already sets. A run's cache is keyed on which rules the binary that produced it compiled, and `Cache::key` never reads the constant itself.

**The measurement is `an_engine_upgrade_that_drops_a_rule_serves_nothing` in `engine/crates/check/tests/cache.rs`.** It warms a cache under one rules identity. It "upgrades" to a second identity standing in for a build that dropped a rule, and it re-runs the same corpus against the same lock. Before this change, the test failed with `Report { hits: 280, misses: 0, unkeyed: 57 }`: every keyed instance served a verdict from before the simulated upgrade. After this change, `hits` is 0 and `misses` is 280. The warm cache serves nothing across the boundary this component exists to name.
