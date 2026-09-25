---
id: HW-OBL-0216
status: discharged
status_since: 2026-09-25
summary: "The merge-regenerate and merged-fold-check hooks tell the merger to run headwater taxonomy resolve --write, and the verb rejects --write because it writes by default."
last_verified: 2026-09-25
title: "Two merge hooks of this repository print a resolve flag that the verb refuses"
waiting_on: build
---

<!-- headwater allow=lifecycle.transition.not_permitted scope=file until=2026-12-31 reason=accepted_deviation note=the record merged at draft against HW-DR-0052, and a check against the merge base cannot see it pass through current inside one pull request -->

# Two merge hooks of this repository print a resolve flag that the verb refuses

## Context

The integrators of #809 and #1051 in run `20260924-0411` each found this remedy text. The product owner ruled it Record, because both hooks belong to this repository and do not ship to an adopter. A search of `engine/crates` found no copy of the text.

## Obligation

`.githooks/merge-regenerate` at line 136 and `.githooks/merged-fold-check` at line 144 tell the merger to run `headwater taxonomy resolve --write`. The verb refuses the flag with "unexpected argument --write", because it writes by default. The working form is `headwater taxonomy resolve`.

## Discharge

This record discharges when both hooks print `headwater taxonomy resolve`, and a case runs the printed remedy and sees it succeed.

[#1058](https://github.com/headwater-ai/headwater/issues/1058) discharged this record on 2026-09-25. Both hooks now print `headwater taxonomy resolve` with no flag. The case `every_resolve_remedy_the_merge_hooks_print_is_one_the_verb_accepts` in `engine/crates/cli/tests/merge_driver.rs` reads each `headwater taxonomy resolve` command that the two hooks print. It runs each command on an adopted tree and requires exit 0. If `--write` goes back into either hook, the verb refuses it and the case fails.
