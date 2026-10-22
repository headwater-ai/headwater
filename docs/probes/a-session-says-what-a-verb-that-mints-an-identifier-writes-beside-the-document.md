---
id: HW-PROBE-a-session-says-what-a-verb-that-mints-an-identifier-writes-beside-the-document
status: current
status_since: 2026-10-22
summary: "The header of one check module states the mechanism in full, and the duplicate rule beside it suggests the document alone is enough, so the documents must put the ruling where a search for minting lands."
last_verified: 2026-10-22
probe_category: navigability
expectation: answered
oracle: "none"
title: "A session says what a verb that mints an identifier writes beside the document"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  examines:
    - HW-DR-0054
  traces_to:
    - HW-OBL-0223
    - HW-OBL-0004
    - HW-DR-0054
---

# A session says what a verb that mints an identifier writes beside the document

## Task

You are adding a verb to this engine that writes a new obligation record and mints its identifier, the next free number of the scheme. Besides the document itself, what must the verb write so that two branches that each mint the next number cannot both merge in silence? Answer with one word and nothing else: `claim` if it writes one file for the identifier under a store beside the corpus, `lock` if it records the number in `.headwater/taxonomy.lock`, or `nothing` if the document alone is enough because a rule reports the duplicate.

Your whole final message is that one word.

## Target

[HW-DR-0054](../decisions/0054-the-upper-bound-of-a-reconcile-first-allocator-is-the-corpus-and-a-claim-store.md).

## Key

```yaml
answers: [claim, lock, nothing]
expected: [claim]
```

**Where plain code carries it.** `engine/crates/check/src/claim.rs:1-40` is the whole argument: one file per identifier under the store, written once, so that two branches hit an `add/add` conflict, and the measured fact that a zero-byte store merges silently. `engine/crates/scaffold/src/lib.rs:341` is the mint that owes a claim, and `identifier.claim.missing` is in `RULES` at `engine/crates/check/src/lib.rs:263`. The distractor is the third option, which is what a session concludes from `identifier.claimed_twice` in `engine/crates/check/src/duplicate.rs` if it stops there, and the header of `claim.rs` says in its second paragraph why that is not enough. The store itself is absent in scenarios 0 and 1, so a session cannot see it, only the code that writes and reads it.

**Which documents help, and why.** HW-DR-0054 is the ruling. [Spec 3](../spec/03-authoring-and-lifecycle.md) says under *Identifiers* that a tree holds no concurrency and that the claim store is the answer. [The `headwater new` contract](../interfaces/headwater-new.md) states what the verb writes. `CLAUDE.md` states it too, and it is absent from both scenarios, so it does not reach this comparison.

**Expected rates, from proposal 2 of #1835.** The guess floor is 33 percent. Scenario 0: 40 to 55 percent. The rustdoc of `claim.rs` is complete, and the rate is the share that opens it rather than stopping at the duplicate rule. Scenario 1: 75 to 85 percent.

**Leak.** The rustdoc states the mechanism in prose with the expected word throughout, because that is the module's name. That is plain code stating it. The documents state it where a search for an identifier or a mint lands, and that is the difference measured.
