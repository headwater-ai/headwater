---
id: HW-OBL-0212
status: current
status_since: 2026-09-26
summary: "The CI wait script counts every run that shares a commit, so a cancelled push run of a feature branch turns a green main merge red."
last_verified: 2026-09-24
title: "ci-done.sh reads a cancelled run of another branch as a red result on a main merge commit"
waiting_on: build
---

# ci-done.sh reads a cancelled run of another branch as a red result on a main merge commit

## Context

The integrator of #848 in run `20260924-0411` found this gap. The product owner ruled it Record, because only this repository's build order reads the script.

## Obligation

`tools/run/ci-done.sh` reported red on main merge commit `d5aa24bd`. The push run of main, 35956153807, was green. A push run of another branch, `fix/809-derived-check-attr` (run 35956584313), shared that commit and was cancelled. The script counts every run on the commit, whatever its branch.

## Discharge

This record discharges when the script reads only runs whose `head_branch` is the branch it waits on, or ignores a cancelled run that another branch superseded, and a case holds the `d5aa24bd` shape.
