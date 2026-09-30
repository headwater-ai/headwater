---
id: HW-OBL-0212
status: discharged
status_since: 2026-10-01
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

The pull request PRNUM for #1484 discharges this record by the second route. `tools/run/ci-done.sh` leaves out a cancelled run when another run of the same workflow on the same commit completed and ran. It also leaves out every check run of that run's check suite. It prints the run as `(superseded, <branch>)`, and a lone cancelled run stays red. No caller changes, because the script needs no branch name. The `d5aa24bd` case in `tools/run/ci-done-fixtures.sh` holds the shape, and `sh tools/run/ci-done.sh d5aa24bd` now reads green against GitHub. The merge commit is not known when this is written, and the pull request names it once it lands.
