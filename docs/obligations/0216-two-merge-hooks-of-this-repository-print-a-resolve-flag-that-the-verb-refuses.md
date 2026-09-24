---
id: HW-OBL-0216
status: draft
status_since: 2026-09-24
summary: "The merge-regenerate and merged-fold-check hooks tell the merger to run headwater taxonomy resolve --write, and the verb rejects --write because it writes by default."
last_verified: 2026-09-24
title: "Two merge hooks of this repository print a resolve flag that the verb refuses"
waiting_on: build
---

# Two merge hooks of this repository print a resolve flag that the verb refuses

## Context

The integrators of #809 and #1051 in run `20260924-0411` each found this remedy text. The product owner ruled it Record, because both hooks belong to this repository and do not ship to an adopter. A search of `engine/crates` found no copy of the text.

## Obligation

`.githooks/merge-regenerate` at line 136 and `.githooks/merged-fold-check` at line 144 tell the merger to run `headwater taxonomy resolve --write`. The verb refuses the flag with "unexpected argument --write", because it writes by default. The working form is `headwater taxonomy resolve`.

## Discharge

This record discharges when both hooks print `headwater taxonomy resolve`, and a case runs the printed remedy and sees it succeed.
