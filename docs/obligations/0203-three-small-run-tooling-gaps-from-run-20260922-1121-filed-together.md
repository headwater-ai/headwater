---
id: HW-OBL-0203
status: draft
status_since: 2026-09-23
summary: "A missing project-board item for two issues, an interactive-shell grep shadowed to basic-regex semantics, and a stale hook-exit-code comment, none with a reader outside this repository, recorded together under the intake cap."
last_verified: 2026-09-23
title: "Three small run-tooling gaps from run 20260922-1121, filed together"
waiting_on: build
---

# Three small run-tooling gaps from run 20260922-1121, filed together

## Context

`hw-run-policy`'s intake rule caps what a single pass sends to this register as its own full record, past which one record lists the rest. Run `20260922-1121` surfaced three small findings this way, none with a reader outside this repository, none sharing a root cause with each other or with [HW-OBL-0202](0202-a-bare-none-footprint-is-claimed-as-a-literal-path-so-two-disjoint-issues-collide.md).

## Obligation

**Claim-through-the-board silently skipped two issues.** `gh project item-list` against board 1 (`PVT_kwDOEsVrw84BgGeU`), checked directly on 2026-09-23, has no item for #847 or for #855, while #828, #817 and #971 — the run's other four merges — are all present. The mechanism `hw-build.md` names for moving a claimed issue onto the board did not run, or did not land, for these two, and nothing in the run's own log explains why. Worth checking whether this is systemic (issues filed a certain way, or moved out of a milestone by an earlier top-of-run pass) or two isolated gaps, before treating it as fixed by adding the two missing cards by hand.

**Interactive `grep -E` is not what it looks like in this sandbox.** `grep` is shell-function-shadowed to `ugrep -G`, which forces basic regular-expression semantics even when `-E` is passed, and silently broke a sanity check on run `20260922-1121` that assumed GNU `grep -E` semantics. Confirmed by `type grep` in the same session. A non-interactive `sh` script is unaffected; only an agent reaching for `grep -E` directly in an interactive shell hits it.

**`.claude/hooks/intent.sh` documents a blocking behavior Claude Code does not have.** A comment in its shadow-log section, predating issue #971, claims a non-zero exit from this hook would block the prompt. Two separate agents independently fetched Claude Code's current hook documentation on 2026-09-22 and confirmed only exit code 2 blocks a `UserPromptSubmit` hook; #971 did not correct the comment because it was out of that change's scope.

## Discharge

**The board-item gap** discharges when `hw-build.md`'s claim-through-the-board step is confirmed to run for every claimed issue (or the two missing cards are added and the mechanism is watched over a further run to see whether the gap recurs).

**The `grep` shadowing** discharges when `hw-run-policy`'s environment section carries a line naming it, the same way its other host-specific traps are recorded, so the next interactive session reaching for `grep -E` is warned before it is bitten.

**The stale comment** discharges when `.claude/hooks/intent.sh`'s shadow-log section is next touched and its exit-code claim is corrected to name exit 2, not any non-zero exit, as the one code `UserPromptSubmit` treats as blocking.
