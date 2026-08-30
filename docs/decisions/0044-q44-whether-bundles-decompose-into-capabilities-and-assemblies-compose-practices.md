---
id: HW-DR-0044
status: draft
status_since: 2026-08-30
summary: "Bundles decompose into capabilities after adopter evidence, and assemblies name the practices that compose those capabilities."
last_verified: 2026-08-30
title: "Q44 — Whether bundles decompose into capabilities and assemblies compose practices"
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: codex
  activity: inspect+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0003
    - HW-DR-0040
    - HW-SPEC-distribution-and-federation
    - HW-OBL-0040
---

# Q44 — Whether bundles decompose into capabilities and assemblies compose practices

## Context

**The library still packages whole traditions as bundles.** `design-spec` carries specifications, registers, evaluations, and reviews. `decision-record` changes the base decision kind and adds obligation records. `standards-spec` and `brd-prd` each also carry several kinds, shelves, relations, and identifier schemes.

**The `decision-record` dependency exposes a poor boundary.** It requires `design-spec` only because that bundle declares the `obligation` purpose. A team that uses decision records therefore selects a numbered-specification tradition that it may not use. The library doctrine records this cost and the `requires` label does not expand the consumer selection.

**Assemblies now separate a named practice from its sources.** The assembly implementation resolves one base package, an explicit bundle selection, and optional cross-bundle glue. It emits a flattened package for a batteries-included consumer. It also preserves the composer path for a consumer that selects capabilities directly.

**The implementation proves the mechanism and does not establish a new module map.** Commits `eb74b21` through `c0e0263` test the recipe reader, resolver, flattening, publication, and consumer path. They use temporary fixture assemblies. No adopter evidence yet identifies the capability boundaries that a published library should carry.

## Decision

**The library will separate capability bundles from practice assemblies.** A capability bundle owns one coherent reader need and the declarations that make that need resolve. A practice assembly selects capabilities and adds only its cross-capability glue, doctrine, and interaction corpus.

**The current bundles remain compatible inputs until the migration proposal has evidence.** This decision does not rename, remove, or split a published bundle. A later package release carries each compatibility and versioning choice.

**The first investigation is the `design-spec` and `decision-record` boundary.** It must test whether specification series, evidence and review, decision refinement, and obligation tracking form separate capability bundles. It must show that each candidate resolves independently or names its necessary capability dependency.

**The investigation records an address inventory and adopter evidence before it changes the package.** The inventory identifies every declaration owner and every direct dependency. The evidence identifies which capabilities adopters select together and which practice assemblies they need. A temporary fixture is evidence that the mechanism works, not evidence of an adopter population.

**An assembly never becomes a second authored taxonomy.** A flattened package remains derived from the capability selection. Publication compares its declarations with a fresh assembly resolution.

## Consequences

**The merged assembly branch carries no bundle split.** It adds the capability for a later package release to publish and consume flattened assemblies. It does not claim that `headwater/starter` or another flattened package exists.

**The next work has three bounded stages.** First, write the declaration inventory and collect adopter evidence. Second, prototype the smallest proposed capability split in fixtures and prove that legacy selections and proposed assemblies resolve. Third, publish a versioned package migration with doctrine, compatibility guidance, and a release record.

**HW-OBL-0040 stays open.** Lower-level bundles do not make arbitrary cross-bundle operations add-only. The assembly overlay solves only the named combination that owns its glue.

**The decision needs human acceptance before the package structure changes.** The proposal records the direction and the evidence bar. It does not authorize a package migration by itself.
