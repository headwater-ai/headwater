---
id: HW-OBL-0199
status: current
status_since: 2026-09-19
summary: The commit an observation snapshot names is stored and never compared. A snapshot naming a commit that never ran discharges a control exactly as well as one naming the commit that did.
last_verified: 2026-09-19
title: "An observation snapshot records a commit and nothing reads it back"
waiting_on: build
---

# An observation snapshot records a commit and nothing reads it back

## Context

[#934](https://github.com/headwater-ai/headwater/issues/934) gave the register an observation dimension. `crate::observation::Observations` reads `.headwater/observations.yml`. A control naming a mechanism outside the engine discharges its obligation only where an entry there names that control. Each entry also names a commit. [Spec 4](../spec/04-assurance-model.md#every-obligation-has-exactly-one-disposition) and [HW-DR-0073](../decisions/0073-a-verification-is-a-kind-and-its-identity-is-minted-rather-than-found-in-the-code-that-cites-it.md) ruling 3 both write the shape as "a control naming the commit it ran against."

`headwater check` runs no version control command. [`headwater-vcs`](../../engine/crates/vcs/src/lib.rs) states that boundary for the crate that owns it, and the check-evaluation path never links it. A comparison against this repository's own history, or against the commit that last changed a control's declaration, needs exactly that command.

## Obligation

Nothing reads the commit field back. `Observations::observed` asks only whether an entry names the control, and the commit is carried and never inspected. Two consequences follow from the same gap.

An entry naming a commit this repository never held discharges a control the same way an entry naming the real commit does. `CT-EXT-1: {commit: "never ran"}` and `CT-EXT-1: {commit: 8f2c1a0}` read identically to `Observations::observed`.

A snapshot recorded once discharges forever. If the control's mechanism, its taxonomy declaration, or the pipeline it names all change after the snapshot commit, nothing re-reads the snapshot against that change. HW-DR-0073 ruling 3 gives a verification a `suspect` state for exactly this shape of staleness. This module has no sibling state.

## Discharge

Waiting on a build. One remedy compares the recorded commit against something the engine can reach without a version control command. The other states in the taxonomy why an adopter's own process is trusted to keep the field honest. `headwater-vcs`'s existing boundary is the constraint either remedy has to work inside. A comparison against this repository's live history is not available to the check-evaluation path today, and widening that boundary is its own decision.
