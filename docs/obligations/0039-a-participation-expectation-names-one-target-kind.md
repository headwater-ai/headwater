---
id: HW-OBL-0039
title: "A participation expectation names one target kind"
status: discharged
status_since: 2026-09-11
waiting_on: build
last_verified: 2026-09-11
summary: "The declaration carried one required `to_kind`, and the member is optional now. An expectation that names none admits every target kind of its relation."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-taxonomy-model
---

# A participation expectation names one target kind

## Context

The [declaration](../spec/02-taxonomy-model.md#participation-expectations) carries one `to_kind`. A corpus that expects a citation from any one of several register kinds cannot state that.

## Obligation

An abstract kind over those kinds looks like the remedy, and it is a dead end. This corpus measured it on `requirement-verified` over `verified_by`, which admits an `acceptance_criterion` and a `probe`. Expectation well-formedness refused `to_kind: governed_document` twice: once because no shelf reaches that kind, and once because `verified_by` does not admit it at its target end. So the abstract kind buys a kind that exists only as a target and still does not validate.

The route that does run is the one the runtime already took. `Participation::satisfied` reads an absent `to_kind` as any target document the relation admits, and `evaluate` already carries the message and the remediation for that case. Only `engine/crates/meta/meta-schema.yml` held the member `required`, so no declaration could reach either branch.

## Discharge

The meta-schema now declares `to_kind` optional. [Spec 2](../spec/02-taxonomy-model.md#participation-expectations) states what the absence means, and `requirement-verified` declares no `to_kind`.

Two fixtures hold the change. `engine/crates/resolve/fixtures/validate/valid/` is the passing side, where an expectation with no `to_kind` resolves and validates clean. `engine/crates/check/tests/participation_target.rs` holds the behavior. Its broad arm reports only the document that reaches nothing, and its narrow arm still reports the probe-verified one.

The [fixture corpus](../taxonomies/decision-record/fixtures/README.md) of the second library entry measured the cost from the outside. It holds an evaluation and no `decision_register`, so the `evidence-cited` expectation reported a finding that no document in that corpus could ever satisfy. That expectation still names a `to_kind`, and it may now drop it.
