---
id: HW-DR-0062
status: current
status_since: 2026-09-11
summary: "A refused transcript fails the run where its state carries the `live` role. A role that says nobody relies on the document leaves the run green."
last_verified: 2026-09-11
title: "A refused recording is held by the reliance its state claims, and not by promotion"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
---

# A refused recording is held by the reliance its state claims, and not by promotion

## Context

A committed transcript pins the taxonomy lock of the tree it ran against. The intake refuses the recording where that digest has moved. `headwater generate` then writes a result that carries the refusal and no verdict, and the run reports it. Whether the run also **fails** was decided by one predicate: the transcript is not a draft.

[#814](https://github.com/headwater-ai/headwater/issues/814) reports what that predicate costs. The standard lifecycle declares `transitions: {draft: [current, deprecated], current: [superseded, deprecated]}`. No transition returns a document to `draft`, and both states a promoted document can reach are past `draft`. So no legal state let a recording go stale. Every change that moved the lock owed four fresh sessions before it could merge.

The toll was paid in full on 2026-09-11. [#413](https://github.com/headwater-ai/headwater/issues/413) made a meta-schema member optional and moved the lock. [#510](https://github.com/headwater-ai/headwater/issues/510) republished the package and moved the pin. Both branches were verified and green on everything else. Both then failed `generate`, `generate --check` and the workspace suite, and the `Protect main` ruleset carries no bypass actor, so GitHub refused each merge. Two of the ten branches of that build-order run moved the lock.

**The taxonomy already separates the two questions that the predicate folded together.** `packages/headwater-standard/taxonomy.yml` gives guidance on every value of `lifecycle_state`. `current` is "the document states what holds now, and a reader may rely on it". `deprecated` is "the document is no longer to be relied on, and nothing replaced it". Only one of the five values asserts reliance, and each value carries a role that says which. "Not a draft" is a different question from "claims to be relied upon", and the engine was asking the first one.

The gate itself is sound and the reason it exists is measured. `f615fb86` landed a transcript and moved the lock in one commit. The recording refused itself on the day it merged. Three governed documents then stated measurements taken off a result that held no verdict, and nothing printed a word about it.

## Decision

**A refused recording fails the run unless a role the engine can read says that nobody relies on it.** The decision reads the role on the state value and never the value itself. Two roles release the refusal.

- `initial` is a recording that somebody is still working on. The remedy for a refusal is a fresh recording rather than an edit. A gate that a contributor cannot clear is a gate that gets removed.
- A `terminal-` role is a recording that the corpus has retired. The document is kept as a record, and nothing new may rest on it, so a stale recording there contradicts nothing the corpus asserts.

Everything else holds the refusal. Four readings land there. A row parsed no document, or a document declares no state. A value sits outside the vocabulary, or a role sits outside what this engine folds. Each one says that nothing has stated where the recording stands. A recording that escaped the gate by declaring nothing is the shape `f615fb86` landed.

**The reading is the one that [HW-DR-0026](0026-q26-whether-terminality-belongs-to-a-state-or-to-a-state-and-a-regime.md) already placed in the check layer.** `StateFacet::standing` folds a state value into the terms that [spec 3](../spec/03-authoring-and-lifecycle.md#lifecycle) states its dependency rule in. The fold is now public, and `headwater-generate` calls it. One reader of the role serves every rule that asks, so a taxonomy cannot be sound under one reading and gated under another.

That fold carries four answers, and the fourth is here for this decision. Spec 3's own rule reads an initial state and an unreadable role alike, because neither of them is live. This decision needs the two apart. One of them releases the refusal and the other holds it.

## Consequences

**A refused recording that claims reliance still fails, and that is the whole of what still fails.** Moving a refused transcript to a `live` state fails `generate` and `generate --check`, with the remedy that names the transcript and asks for a fresh recording. A transcript at `draft` that somebody promotes without a fresh recording fails on the promotion. `engine/crates/generate/tests/probe_result.rs` holds both, and it holds the four answers a role gives as a table.

**A stale recording now has a state to stand in.** The retirement is `current` to `deprecated`, which the standard lifecycle already permits, with `status_since` moved to the day of the retirement. The document says in its own prose that the lock moved and that nobody recorded fresh sessions. `docs/probe-runs/regression-probe-transcript-for-2026-09-11.md` is the first such recording.

**A sixth state classifies itself.** No literal state value in `engine/crates/generate/` decides whether a refusal is held. A taxonomy that adds a state, or renames every state it has, is read here through the role each value declares. That is [spec 2](../spec/02-taxonomy-model.md#the-immutable-core)'s worked overlay, reaching one more component.

**What this does not do is the measurement.** The efficacy figure of this corpus stays absent until somebody records four fresh sessions. The owner ruled on 2026-09-11 to let the recording go stale until it is worth bringing up to date. This decision makes stale a state the system can hold. It changes no confirmation, no digest a transcript pins, and nothing about what a refusal means.
