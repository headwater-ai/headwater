---
id: HW-OBL-0114
status: discharged
status_since: 2026-09-19
waiting_on: ruling
summary: A committed snapshot separates an external control that ran from one that only claims to, and a sweep control discharges nothing without one.
last_verified: 2026-09-19
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

[#934](https://github.com/headwater-ai/headwater/issues/934) took the second of the three answers this record named. The register now separates an external control that leaves evidence from one that does not. `crate::observation::Observations` reads a committed snapshot at `.headwater/observations.yml`. Each entry names one control and the commit it ran against, on the same offline terms the identifier claim store already reads by. `Disposed::unobserved` (`engine/crates/check/src/register.rs`) is the subset of an obligation's controls that name an external mechanism and that the snapshot does not name. `Disposed::discharged` excludes such a control. [Spec 4](../spec/04-assurance-model.md#every-obligation-has-exactly-one-disposition) states the rule. A control that names a mechanism outside the engine discharges nothing unless a committed snapshot names it and the commit it ran against.

`engine/crates/sweep/fixtures/sweep.taxonomy.yml` still declares `OB-SWP-1` with `class: coherence` and one control, `CT-SWP-1`, whose mechanism is `sweep:undeclared_conflict`. No snapshot in that fixture tree names `CT-SWP-1`. So `a_control_that_names_a_sweep_discharges_nothing_with_no_observation` now asserts the corrected reading: the obligation is not discharged, and `CT-SWP-1` is in `unobserved` rather than in `unimplemented`. A `sweep:` name is external, not an unrecognized `check:`/`phase:` name, so the two lists stay disjoint.

The obligation itself is unchanged: the class the sweep serves still has no member run over this corpus. What this record owed was the register's grading of that one control, and that grading is what changed. A later corpus might declare a coherence obligation whose sweep a person runs and records by hand. That corpus could still ask whether such a sweep should verify the obligation directly, rather than through a committed snapshot. No corpus here declares one, so that question stays open.
