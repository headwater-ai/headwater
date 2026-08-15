---
id: HW-OBL-0102
title: "Two documents state what voids a verdict, and they do not agree"
status: current
status_since: 2026-08-13
last_verified: 2026-08-14
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
    - HW-SPEC-check-layer
    - HW-SPEC-assurance-model
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

The second is that the reading of [HW-OBL-0017](0017-publishing-the-read-set-skips-a-re-run-on-two-of-the-eighteen.md) points the gate at the merges it cannot inspect. Two Shape rules generate over every kind, so an edit to any classified document voids almost every instance. The merges that a gate could carry are therefore the ones that add rather than edit. That is the class that a comparison over listed inputs cannot see.

The third is the clock. The void condition of both specifications reads a tree alone. A windowed participation expectation goes stale with no tree change at all, and `readset.rs` says so where no specification does.

The first consequence has a construction, and `identifier.claimed_twice` is the rule that supplies it. One branch adds a document that mints `HW-SPEC-new`. A second branch adds a different document that mints the same identifier. Each branch is valid, because each holds one claimant of it. The merge holds two claimants, so the merge result is invalid. The read set of the corpus-scoped instance lists every document of the branch that produced it. Neither branch lists the document that the other one added. Every listed hash therefore stands, and a comparison over listed inputs reports that both verdicts survive.

The cache and the gate read one artifact by two procedures, and that is why one of the two is sound. A run rebuilds the read set of every instance from the tree in front of it. A document that the tree gained joins that set and moves the key. A gate holds the set that an earlier run published and compares it against a later tree. A document that the tree gained is on no list. That a cache cannot serve a stale verdict is therefore no evidence about what a gate can decide.

## Discharge

**This record is discharged.** [Spec 12](../spec/12-check-layer.md#the-read-set-and-what-a-merge-does-to-a-verdict) carries one statement of what a gate reads. A gate reads the published read set and the tree in front of it, and nothing else. `headwater gate --read-set <path>` is that statement as a verb, and the statement decides the three things above.

**The test is a comparison over listed inputs.** A gate hashes the file at each listed path and holds it against the hash the run recorded. [HW-OBL-0028](0028-a-run-cannot-report-the-corpus-tree-because-nothing-computes.md) holds the other reading, and this ruling settles the dependency rather than waits on it. A comparison over listed inputs needs no tree, so a run that computes none blocks nothing here. That record stands on [spec 6](../spec/06-engine-architecture.md#ci-adapters), which asks a run for a tree for reasons of its own.

**The clock voids a verdict.** The artifact names each rule that read the injected clock on a `windowed` line. A gate asked about another day voids those rules, and no tree change is needed for that.

**Membership of the census is an input, and a list of members carries no membership.** A read set records what a run read. It never records that those were all the documents there were. So nothing this design holds carries a corpus-scoped verdict across a merge. The artifact names each such rule on a `barrier` line, and a gate voids every barrier whatever the listed hashes did. [Spec 10 §F.6](../spec/10-theoretical-foundations.md#f6-write-skew-names-the-anomaly-and-read-sets-detect-it) names the piece the transfer dropped: a read set is not a predicate lock.

**The construction in the section above runs against the engine.** Two trees each hold one claimant of one identifier, and each tree passes `headwater check --strict`. The merge holds both claimants and fails it. A gate that holds the first tree's read set against the merged tree reports that the verdict does not carry, and it names the barrier. The same gate over the same artifact with the `barrier` line struck out reports that the verdict carries. That second run is the phantom itself, measured rather than argued.

**A model in Alloy is unnecessary, because the trace runs.** The model was the cheap instrument for the first decision. It holds documents as atoms, and a branch as a pair of a read set and a write set. A trace against the engine answers the same question at a higher standard, and this record carries one.

**Two documents keep the reading this record found, and each keeps it for a reason.** [Q21](../decisions/0021-terminological-succession-and-validity-under-merge.md) is a decision, and succession amends a decision rather than an edit. [What a check can know](../evaluations/what-a-check-can-know.md) is an evaluation, and an evaluation records what one exercise concluded. A reader who meets either sentence reads a record rather than a rule.
