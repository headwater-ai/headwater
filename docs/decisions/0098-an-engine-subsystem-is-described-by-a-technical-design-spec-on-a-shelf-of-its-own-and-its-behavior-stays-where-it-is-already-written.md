---
id: HW-DR-0098
status: current
status_since: 2026-09-28
summary: "Each engine subsystem gets one technical design spec on a new shelf, and it governs its crates by one pattern each. No crate gets a functional spec, because interface contracts, spec parts and requirements already state behavior."
last_verified: 2026-09-28
title: "An engine subsystem is described by a technical design spec on a shelf of its own, and its behavior stays where it is already written"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-engine-architecture
    - HW-DR-0074
---

# An engine subsystem is described by a technical design spec on a shelf of its own, and its behavior stays where it is already written

## Context

[#1283](https://github.com/headwater-ai/headwater/issues/1283) measured the engine source on 2026-09-28. `headwater taxonomy audit` reports that no `governs` edge reaches 137 of the 184 files under `engine/crates/**/src/**/*.rs`. The `check` crate has 8 of 54 files reached, and `resolve` has 3 of 17. An edit to one of those files names no document.

The gap is not only missing edges. No document describes the inside of a crate. [Spec 6](../spec/06-engine-architecture.md) describes the whole engine in eight sections: the pipeline, the checks, the projections and the performance targets. It is the only place that names the crate structure. Five spec parts are longer than 11,000 words, and spec 7 is about 17,600. Implementation detail and system promises share one document in each of them.

The question was whether each crate needs a functional spec and a technical spec, or a technical spec alone. A reader meets the behavior of the engine through its verbs, and the verbs do not map one to one onto crates. The `governs` edges of the 25 interface contracts show this. The `query` crate serves five verbs: `explain`, `mcp`, `query`, `route` and `show`. The `census` crate serves three. The `taxonomy` verb reaches three crates: `audit`, `compat` and `resolve`.

Behavior already has three homes. A spec part states what the system promises. An interface contract states what one verb does, in the shape of a manual page. A requirement states one testable statement, and an acceptance criterion settles it. Only the third home is thin, with two requirements and two criteria.

The owner ruled on the three questions below in a session on 2026-09-28.

## Decision

**An engine subsystem gets one technical spec, and no crate gets a functional spec.** A functional spec per crate would state the behavior of a verb a second time, cut along crates instead of verbs. That gives one sentence two owners, which [HW-PD-0001](../process/decisions/0001-orchestration-prose-has-one-owner-per-sentence.md) forbids for orchestration prose. The same reason applies here, because two copies of one behavior drift apart.

**A technical spec states how a subsystem works, and why.** It states the data structures, the invariants, the algorithms and the reasons for them. It links to the interface contract, spec part or requirement that states the behavior, with `traces_to`, and it does not repeat that behavior.

**The unit is a subsystem, not a crate.** A subsystem is one stage, or a small set of stages, of the pipeline that spec 6 draws. The stages are resolve, parse, graph build, cache, checks, queries, projections, export and explain. The authoring verbs and the measurement layer are two more subsystems that the diagram does not draw. Every crate under `engine/crates/` belongs to exactly one subsystem. Six crates hold one source file each, so a spec for each crate would often be one paragraph long.

**A technical spec is a `design_spec` on a new shelf, `docs/subsystems/`.** The numbered parts under `docs/spec/` stay at the level of the system. The `design_spec` kind comes from the design-spec bundle and already carries the `spec_id` scheme. So the only change to the taxonomy is the shelf.

**A technical spec governs each crate it owns by one pattern.** The anchor is `engine/crates/<crate>/src/**`, which [HW-DR-0074](0074-a-code-path-anchor-is-a-pattern-over-the-tree-and-it-binds-when-the-pattern-matches-at-least-one-entry.md) permits. A new file in the crate is then reached with no new edge. Exactly one technical spec governs the source of each crate. An interface contract can keep its edges onto the files that implement its verb.

**Spec 6 keeps the pipeline and the map from each stage to its subsystem spec.** The implementation detail in spec 6 moves to the subsystem spec that owns it. Specs 2, 7 and 12 move their implementation detail the same way, when the subsystem spec that owns it exists. A spec part keeps what the system promises.

## Consequences

A taxonomy change is owed before the first subsystem spec: a shelf declaration for `docs/subsystems/**` that admits `design_spec`. The `headwater-taxonomy` skill owns that change. It also answers one open point. `design_spec` requires the `sequence` facet on `spec_series`, and the new shelf declares whether it requires a sequence too.

The crate criterion of #1283 is met by the subsystem specs, not by spec 6. A `**` anchor on spec 6 would make one document the rule for every crate. The #956 recount of 2026-09-21 rejected that shape for a subtree that many documents share.

Each subsystem spec is a document to write, and it must read the code it describes. The work is about ten documents, one for each subsystem, and it can land one subsystem at a time. The audit reading for `engine/crates/**/src/**/*.rs` is the measure of progress.

This decision does not close the gap on the functional side. Two requirements and two acceptance criteria are too few to state the behavior of the engine as testable statements. That gap is separate work.

Where an adopter links a crate as a library, the public API of that crate is a contract with an outside reader. That contract is an interface contract or rustdoc beside the code, not a subsystem spec. This decision is open again if such an adopter appears and a subsystem spec starts to state an API contract.
