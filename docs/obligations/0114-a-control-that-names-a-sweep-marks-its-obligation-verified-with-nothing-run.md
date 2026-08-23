---
id: HW-OBL-0114
status: current
status_since: 2026-08-14
waiting_on: ruling
summary: The register holds no coherence obligation at all, and the one mechanism that could bind the sweep to one reports it verified from the declaration alone.
last_verified: 2026-08-14
title: "A control that names a sweep marks its obligation verified with nothing run"
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-assurance-model
    - HW-SPEC-check-layer
---

# A control that names a sweep marks its obligation verified with nothing run

## Context

[Spec 4](../spec/04-assurance-model.md#discharging-coherence-obligations-the-assisted-sweep) says that the assisted sweep is how coherence obligations get discharged at all. This repository declares 25 obligations and every one of them carries `class: cohesion`. The class the sweep serves has no member, so the sweep discharges nothing here and the claim has never been tested against a declaration.

The binding a sweep would need is a control. A control names a mechanism and the obligations that mechanism discharges. The engine reads two prefixes, `check:` and `phase:`, and it names any other prefix external. Spec 4 admits an external control on purpose, because a corpus may declare a control that no check layer runs.

## Obligation

An obligation reads `verified` only when something that ran discharged it. The register derives the disposition from the controls that name the obligation, minus the controls whose mechanism the engine does not implement. An external control is in neither set, so it counts toward discharge with no run behind it.

That reading is defensible for a control that a person performs on a schedule and records. It is wrong for a sweep. A sweep is opt-in and network-bound, and no run of the checks can observe that one happened. A count over sweeps that ran has a denominator that anybody empties by not running one.

So the corpus owes one of three answers. The engine reads a `sweep:` prefix and reports an obligation bound to it as unverified until a report lands. The register separates an external control that leaves evidence from one that does not. Or the specification states that a coherence obligation is never `verified` and says what disposition it carries instead.

## Discharge

Measured on 2026-08-14, from the code and from a fixture that runs. `Register::mechanism` in `engine/crates/check/src/register.rs` returns `Mechanism::External` for any prefix outside `check:` and `phase:`. `Disposed::discharged` is `controls.len() > unimplemented.len()`, and `Projection::of` puts only an `Unimplemented` mechanism in the second list. So one external control is enough.

`engine/crates/sweep/fixtures/sweep.taxonomy.yml` declares `OB-SWP-1` with `class: coherence` and one control whose mechanism is `sweep:undeclared_conflict`. The test `a_control_that_names_a_sweep_verifies_its_obligation_with_nothing_run` asserts that the obligation reads discharged and that no control under it is unimplemented. No sweep ran in that test, and no run of the checks can run one.

The count of 25 obligations, all of class `cohesion`, is over `packages/headwater-standard/taxonomy.yml` and `.headwater/overlay.yml` on the same day. The engine's own report gives the same total under `obligations`.
