---
id: HW-DR-0054
status: current
status_since: 2026-09-06
summary: "A tree holds no concurrency, so two branches mint one number in silence. A claim store of one file for each identifier makes the two branches meet, and the rule that already reports a duplicate then fires on the branch."
last_verified: 2026-09-06
title: "The upper bound of a reconcile-first allocator is the corpus and a claim store"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0021
---

# The upper bound of a reconcile-first allocator is the corpus and a claim store

## Context

[Spec 3](../spec/03-authoring-and-lifecycle.md#identifiers) states that allocation reads a tree and that a tree holds no history. So the highest value a run finds is a lower bound on every value ever allocated. That sentence is about deletion, and it is correct about deletion. A tree holds no concurrency either, and nobody wrote that consequence down.

Every branch cut from one state reads the same corpus. So two branches mint the same number. Each branch is internally consistent and passes a strict run, the two file names differ, and the merge is silent. The pair first exists after the second merge, when no run reads either change. This corpus met the failure three times in one working day on 2026-09-06. `0048` landed twice. Two claims on `0049` were caught by a person who read other branches by hand, which is not a mechanism.

**Three forms of a counter were measured on real merges, in scratch repositories, rather than reasoned about.**

A bare watermark holding the highest number **fails silently**. Both branches write the same value, the version control system reads agreement, and the merge is clean with the duplicate invisible. Anyone who implements this option that way ships something that looks like a fix.

An append-only ledger, one line for each mint, **conflicts when nothing collided**. Two branches that mint two different identifiers in one namespace append at the same end of one file. The two lines land in one hunk, and the merge refuses. That is a false positive on every pair of concurrent mints, and the decision shelf saw three mints in one working day. The measurement that the ledger also refuses the innocent case is new. [#573](https://github.com/headwater-ai/headwater/issues/573) reads the ledger as a form that conflicts in both cases, and treats that as a virtue.

A single claimant-bearing line **cannot express what a resolver needs**. One line holds one claim. So a branch that holds a number the mainline skipped reads as a branch below the high-water mark. The obvious resolution discards a legitimate claim and forces a renumber.

## Decision

**The upper bound of a reconcile-first allocator is the corpus and a claim store together.** The store holds one file for each identifier, at `.headwater/ids/<scheme>/<identifier>`, and the file holds the path of the document that minted it. A claim is written once, at the mint, and nothing modifies one after that. The directory grows and no file in it does.

**The store does not report the collision. It makes the two branches meet.** Two branches that claim one value add one path with two different contents, which is an add-add conflict and nothing subtler. To resolve that conflict the second branch takes the mainline into its own tree. `identifier.claimed_twice` then reports the pair on the branch, before the merge, where the remedy is a rename rather than a renumber. So the file is what makes the read set of an existing rule reachable in time. Neither half answers alone.

**The path inside the file is the mechanism and not a label.** Two branches that add one claim path holding zero bytes merge at exit 0 with no conflict. The version control system compares content before it selects a merge strategy, and two identical empty blobs are one change. A claim file that holds nothing is the watermark failure at the grain of one file. So a claim is never empty, both writers refuse to make one that is, and `tools/repo/id-store-fixtures.sh` fails on an empty claim anywhere in the store.

**Both writers create and never overwrite.** `headwater new` and `headwater check --fix` open a claim with create-new semantics. The test for an occupied path and the creation are one system call, so an occupied path is a refusal. A claim file is the only record of which document minted an identifier. The tree cannot hold that record, and nothing repairs a claim that was written over. That is the one place here where a wrong write destroys a fact rather than reporting a wrong one.

**`.headwater/ids/**` takes no merge attribute, ever.** `merge=union` on that path concatenates two claimants into one file and exits 0, which is the same loss by another route.

**The claim comes first and the document second.** Nothing is written before the claim, so a claim that cannot be made refuses with an untouched tree. A document write that then fails leaves a spent number with no document, and spec 3 already declares that state legitimate.

**Two rules hold the store, and one direction of the pair is deliberately absent.** `identifier.claim.missing` is an error and it carries a patch, because the correction is one file whose content is the path of the document. `identifier.claim.stale` is advisory, because a decision about which of the two moved is a rewrite. A claim that names a document the corpus no longer holds is **correct** under the rule that an identifier is never reused. No rule reports one.

## Consequences

**Four schemes take a claim, not six.** `decision_id`, `acceptance_criterion_id`, `obligation_record_id` and `requirement_id` declare `allocation: reconcile-first`. [#573](https://github.com/headwater-ai/headwater/issues/573) names `register_id` and `interface_contract_id` as well, and both allocate `minted-once`. Every reconcile-first scheme patterns a fixed-width sequence and every minted-once scheme patterns a slug. That correlation is exact today, and it says what the store is for: sequence allocation. Two branches that pick one slug write one file name, so the document itself is the conflict and the version control system reports it.

**One residual is named rather than claimed away.** `docs/taxonomies/design-spec/bundle.yml` gives the specification shelf a layout that opens with a sequence while `spec_id` patterns a slug. Two branches that add a part with one slug at two sequence prefixes produce two paths and one identifier, silently. The store keys on the identifier rather than on the sequence, so it extends to every scheme at no design cost. That extension is not this ruling.

**A scaffolder may read what a checker may not, and this store is not that case.** [HW-REQ-0001](../requirements/0001-the-engine-reaches-no-network-at-check-time.md) forbids a network read at check time, and the option of reading remote branches would have needed that boundary stated. The store needs no network and no input the check layer does not already take, so the question the issue raised does not arise here.

**The store is bootstrapped by the fixer rather than by a script.** `identifier.claim.missing` is fixable, so `headwater check --fix` writes what the corpus already spent. The same mechanism repairs a claim lost to a hand edit, and it reaches an adopter who runs `headwater init` over an existing corpus. A regenerator would rewrite claims, which the never-modify rule forbids. It would also be correct exactly once, because the store diverges from the corpus the first time a document is deleted.

**The bootstrap of this corpus is 216 files and it is not contiguous.** `docs/decisions/0049` is absent from the mainline and an open change claims it. So the decision store bootstraps to 52 files, covering 0001 to 0048 and 0050 to 0053.

**The store is the extreme decomposition of a rule that another change is landing.** An open change rules that a fold over a corpus is derived and never stored. It rules that a recorded artifact holds one record for each entity, in a fixed order, and holds no count of those records. This store is that rule taken as far as it goes: one record for each entity, one file for each record, and no count anywhere. This record states the relationship in prose rather than as an edge. The record that carries the general rule is not on the mainline today.
