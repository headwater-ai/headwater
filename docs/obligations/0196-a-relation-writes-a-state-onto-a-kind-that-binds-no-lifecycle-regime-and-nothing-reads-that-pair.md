---
id: HW-OBL-0196
status: current
status_since: 2026-09-12
summary: "A rule reads on_target.set_state, and it skips a kind that binds no lifecycle regime and can stand in no state."
last_verified: 2026-09-12
title: "A relation writes a state onto a kind that binds no lifecycle regime and nothing reads that pair"
waiting_on: ruling
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
---

# A relation writes a state onto a kind that binds no lifecycle regime and nothing reads that pair

## Context

A relation may declare `on_target.set_state`, which names the state that the edge writes onto the document at its target end. [Spec 2](../spec/02-taxonomy-model.md#the-meta-schema) now asks, under `relation coherence`, that the state is one every kind at the `to` end can stand in. The clause reads the lifecycle regime of each concrete kind the `to` end stands for, and it asks whether that regime names the state.

The clause skips a kind whose chain binds no lifecycle regime at all. The engine reads the binding the way `headwater_check::Shape::lifecycle_of` reads it, nearest ancestor first, and a kind that binds none answers `None` there too. A kind that binds no regime stands at no state. A relation that writes one onto it is a worse defect than the one this clause refuses. No rule of `taxonomy validate` reports it.

The common case is an anchor-adjacent kind. A relation endpoint may name an anchor kind rather than a kind, and `referential_integrity` already admits both. A concrete kind that binds no regime is the other half of the same shape, and it passes every one of the twenty-three rules.

## Obligation

The corpus owes a ruling on what the engine reports when a relation writes a state onto a kind that can carry no state.

Three answers are open, and the record states the gap rather than picks one. The first is a refusal under `relation coherence`, beside the clause that ships here. The second is a refusal under `lifecycle soundness`, which already owns the question of which kind reaches which state. The third is that the pair is legal. A kind with no regime is outside the lifecycle language, and a relation that writes a state onto it writes nothing.

Nothing here is a defect of the clause that ships. The clause answers the question that two declarations decide. The engine cannot pick the remedy for the third case. Each of the three answers moves a different rule, and two of them move spec 2.

This repository holds no instance. All seventeen concrete kinds of `headwater/standard` bind a lifecycle regime, and every bundle under `packages/headwater-standard/bundles/` binds one of the same two. So the finding is about what a third-party package may declare, and not about what this corpus states today.

## Discharge

This record discharges when the owner rules, in a decision record that names which of the three answers holds and what reopens the ruling.

A ruling that the pair is a refusal discharges it with a fixture beside the one this change adds. It also names in spec 2 the rule that carries the refusal. A ruling that the pair is legal discharges it with one sentence in the `relation coherence` bullet of spec 2. That sentence says the clause reads a kind that binds a regime, and says nothing about a kind that binds none. The rustdoc above `relation_coherence` in `engine/crates/resolve/src/rules.rs` states the scope today. A reader who meets the scope there has no ruling to read.
