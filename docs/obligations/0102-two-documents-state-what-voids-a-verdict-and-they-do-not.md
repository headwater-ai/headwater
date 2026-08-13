---
id: OBL-repo-0102
title: "Two documents state what voids a verdict, and they do not agree"
status: current
status_since: 2026-08-13
last_verified: 2026-08-13
summary: "Spec 12 voids a verdict by a tree diff, and the engine voids it by a hash comparison that no added document reaches."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - SPEC-HW-check-layer
    - SPEC-HW-assurance-model
---

# Two documents state what voids a verdict, and they do not agree

## Context

[Spec 12](../spec/12-check-layer.md#the-read-set-and-what-a-merge-does-to-a-verdict) rules that a merge is an ordinary change. It takes the merge result and the tree that a run evaluated, then it derives the invalidated instances as it derives them from any diff. [Spec 4](../spec/04-assurance-model.md#a-verdict-is-about-one-state-of-the-corpus) states the same rule in the same words.

`engine/crates/check/src/readset.rs` states a different rule. A gate compares each input of the published read set against the tree that it now holds. An input whose hash moved invalidates the instances that read it.

The two are not one test. A diff reads two trees, so it sees a file that the merge added. A read set lists the inputs that the checks opened. A file that no instance read reaches no list, so it moves no hash. The database literature that [spec 10](../spec/10-theoretical-foundations.md#f6-write-skew-names-the-anomaly-and-read-sets-detect-it) cites names this class of miss a phantom. It is the part of the transfer that the specification did not carry.

## Obligation

The corpus owes one statement of what voids a verdict, and that statement must rule on a document that a merge adds.

Three consequences follow from the disagreement, and no document records any of them.

The first is a sentence that the published test falsifies. Spec 12 rules that a corpus-scoped instance reads everything, so any concurrent change voids it. Under a comparison over listed inputs, a merge that only adds documents moves no listed hash, and the instance survives.

The second is that the reading of [OBL-repo-0017](0017-publishing-the-read-set-skips-a-re-run-on-two-of-the-eighteen.md) points the gate at the merges it cannot inspect. Two Shape rules generate over every kind, so an edit to any classified document voids almost every instance. The merges that a gate could carry are therefore the ones that add rather than edit. That is the class that a comparison over listed inputs cannot see.

The third is the clock. The void condition of both specifications reads a tree alone. A windowed participation expectation goes stale with no tree change at all, and `readset.rs` says so where no specification does.

## Discharge

One statement in spec 12 of what a gate reads, and the statement decides three things. It decides whether the test is a diff of two trees or a comparison over listed inputs. It decides whether the clock voids a verdict. It decides whether membership of the census counts as an input.

[OBL-repo-0028](0028-a-run-cannot-report-the-corpus-tree-because-nothing-computes.md) holds the other half of the answer. A diff of two trees needs a tree, and no run computes one. So the specification today asks for a test that the engine cannot run.

Nothing ships a wrong verdict while this stands open. No verb of this engine takes a read set and a later tree, and the commit hook re-runs the corpus. Every verdict today is therefore a verdict about the tree that produced it.

A model in Alloy is the cheaper instrument for the first decision, and it is optional. It holds documents as atoms, and a branch as a pair of a read set and a write set. One invariant over the whole corpus then produces the trace that a comparison over listed inputs misses.
