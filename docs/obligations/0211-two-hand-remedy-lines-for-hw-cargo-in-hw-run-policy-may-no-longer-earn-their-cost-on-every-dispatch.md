---
id: HW-OBL-0211
status: current
status_since: 2026-09-26
summary: "Since #1047 and the link suite of #849, the two hw-cargo workarounds in hw-run-policy may be dead text that every build-order stage pays to read."
last_verified: 2026-09-24
title: "Two hand-remedy lines for hw-cargo in hw-run-policy may not earn their cost on every dispatch"
waiting_on: build
---

# Two hand-remedy lines for hw-cargo in hw-run-policy may not earn their cost on every dispatch

## Context

The build of #849 (PR #1062) in run `20260924-0411` asked whether two lines of `.claude/skills/hw-run-policy/SKILL.md` still earn their cost. Every build-order stage loads that skill, so each line costs every dispatch. The product owner ruled the finding Record.

## Obligation

The skill carries two hand remedies for `tools/hw-cargo`: the slot probe with `flock -n` and the workaround for a wait on slot 1 alone. The line itself says to delete it when `tools/hw-cargo` is fixed. #1047 and the link suite of #849 may have fixed it.

The same build found a reusable test method. A suite finds its script through `$(dirname "$0")/..`, so a mutant script in a scratch `tools/` directory shows that a fixture goes red without a broken commit.

## Discharge

This record discharges when a measurement on main shows whether `tools/hw-cargo` still needs each remedy, and each line that `tools/hw-cargo` does not need leaves the skill.
