---
id: HW-DR-0034
status: draft
status_since: 2026-08-24
summary: "Acceptance is the merge onto main. A provenance block on an unmerged branch is a proposal, so an agent may write `accepted_by` there, and stop rule 5 binds main rather than the byte."
last_verified: 2026-08-24
title: "Q34 — Whether acceptance means merged to main, and what an agent may write before that"
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-ai-integration
    - HW-OBL-0108
---

# Q34 — Whether acceptance means merged to main, and what an agent may write before that

## Context

[HW-OBL-0108](../obligations/0108-an-agent-writes-the-acceptance-stamp-of-every-document-in-this-corpus.md) holds two readings of [stop rule 5](../spec/05-ai-integration.md#the-stop-rules) and could not choose between them. The first reading keeps the rule as written. An agent writes `warrant: asserted` and nothing else, and a human moves both fields by hand at review. The second reading treats a pull request as the review and a merge as the act `accepted_by` names. Under it, the stamp is a proposal before the merge and a fact after.

The obligation could not settle on the second reading. The corpus keeps no record that separates a document a human merged from a document an agent stamped and nobody read. Both are the same bytes, on `main` or off it.

A re-measurement on 2026-08-24, over every document under `docs/` outside `docs/taxonomies/`, updates the count the obligation took ten days earlier.

| reading | 2026-08-14 | 2026-08-24 |
|---|---|---|
| documents that carry `drafted_by` | 182 | 203 |
| documents that carry `accepted_by` | 179 | 181 |
| documents that carry `warrant: asserted` and no `accepted_by` | 6 | 24 |

The population that stamps itself grew by two documents in ten days. The population that withholds the stamp grew by eighteen. Neither growth answers the question the obligation raised, and no ruling governed either one while they grew.

Two further obligations bear on the second reading and neither settles it. [HW-OBL-0030](../obligations/0030-the-provenance-block-belongs-to-the-engine-and-nothing-states.md) records that no taxonomy declares the provenance block. So no rule of this engine reads a field of it, on either side of a merge. [HW-OBL-0105](../obligations/0105-nothing-plays-the-hook-role-that-two-relations-name.md) records that no actor in this engine writes a relation the base package assigns to `hook`. HW-OBL-0108 cites that gap as a reason a two-stage warrant has no way to move from `asserted` to `accepted` at merge time.

## Decision

**Acceptance is the merge onto `main`.** A document is governed from the moment its content lands on `main`, and not before. A provenance block on a branch, or in a pull request that has not merged, states a proposal. No rule of this engine, and no reading of stop rule 5, treats it as a claim that a human has read the document.

**An agent may write `warrant: accepted` and `accepted_by` on a document it drafts.** This holds where the document goes into a pull request. A named human must review that request before the merge. The rule that follows from this is symmetric. An agent may also write `warrant: asserted` and leave the stamp for the reviewer to add. Both are compliant. This ruling does not prefer one over the other.

**Self-acceptance is a document that reaches `main` with no human review between the draft and the merge.** It is not a value an agent writes on a branch nobody has to accept before the branch closes. A branch that closes without a merge carries whatever stamp it carried. The ruling reaches that case cleanly. The document it proposed never joined the corpus this repository governs. So the stamp on it answers to nothing, the same way the rest of the branch does not answer to `headwater check --strict`.

**This reading needs no actor that promotes a stamp at merge time.** The first reading needs one, because a two-stage warrant needs a write between the draft and the merge. HW-OBL-0105 records that no such actor exists in this engine. Under this ruling nothing moves the stamp. The stamp a draft carries is the stamp the merged document carries, and the merge decides whether that stamp was true.

**This ruling picks between the two readings HW-OBL-0108 posed, and takes the second.** A pull request is the review. A merge is the acceptance `accepted_by` names.

**This ruling does not reach the third practice.** [HW-OBL-0125](../obligations/0125-nine-documents-state-a-warrant-the-closed-set-does-not-hold-and-no-check-reads-one.md) records nine documents that state `warrant: proposed`, a value outside the closed set [spec 3](../spec/03-authoring-and-lifecycle.md#evidence-has-three-honest-states-not-two) declares, each with an acceptor named beside it. That is a defect in the value, not a question about when acceptance happens, and this ruling leaves it for HW-OBL-0125 to settle.

## Consequences

**What a writer meets.** An agent drafting a document under `docs/` may write `accepted_by` and `warrant: accepted` at draft time. This holds where the document is going into a pull request a named human will read before the merge. Where no human will read it before the merge, the agent writes `warrant: asserted` and nothing else. Nothing then stands between the draft and `main`.

**What stop rule 5 now means.** [Spec 5](../spec/05-ai-integration.md#the-stop-rules) states the rule against a byte an agent may write, and this ruling reads it against a document that reaches `main`. The wording changes to say so, so a later reader meets the ruling in the rule rather than in a citation to it.

**What this settles.** [HW-OBL-0108](../obligations/0108-an-agent-writes-the-acceptance-stamp-of-every-document-in-this-corpus.md) is discharged. The 181 documents that already carry `accepted_by` from the drafting agent needed no correction under either reading, and this ruling confirms it. The 24 that hold the stamp back needed none either, and this ruling does not ask that they be rewritten to match the older habit.

**What stays open.** [HW-OBL-0030](../obligations/0030-the-provenance-block-belongs-to-the-engine-and-nothing-states.md) is unaffected. The provenance block still answers to no taxonomy declaration. So no rule of this engine reads `accepted_by` or `warrant`, on either side of a merge, whatever this ruling says about what the field means. [HW-OBL-0105](../obligations/0105-nothing-plays-the-hook-role-that-two-relations-name.md) is unaffected in its own right, because it concerns `governs` and `traces_to` rather than the provenance block. This ruling removes the one case HW-OBL-0108 borrowed it for, which is the promotion actor a two-stage warrant would have needed.

**What reopens this.** A branch-protection posture that lets a document reach `main` with no review would erase the line this ruling draws. Nothing in this engine enforces that posture today. So the ruling stands on the process this repository already runs, rather than on a control the engine holds.
