---
id: HW-OBL-0211
status: discharged
status_since: 2026-10-13
summary: "The hw-cargo line in hw-run-policy was not dead text, because the tool still waited on slot 1 alone when every slot was busy. The tool now takes whichever slot frees first, and the line is gone."
last_verified: 2026-10-13
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

The record is discharged on 2026-10-13, by the pull request for slice 1 of #1483. Both remedies were one line of the skill. A measurement on `main` at `c5469354` showed that `tools/hw-cargo` still needed it. A caller that found no free slot waited on slot 1 alone. That pull request makes the tool probe every slot once a second, so it takes whichever slot frees first. A case in `tools/hw-cargo-fixtures.sh` holds three slots, frees slot 2 first and requires the tool to take slot 2. It failed on the tool as it stood and passes on the fix. The line leaves `.claude/skills/hw-run-policy/SKILL.md` in the same change.
