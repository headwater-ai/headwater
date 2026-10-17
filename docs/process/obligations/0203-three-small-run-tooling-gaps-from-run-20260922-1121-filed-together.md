---
id: HW-OBL-0203
status: discharged
status_since: 2026-10-13
summary: "A missing project-board item for two issues, an interactive-shell grep shadowed to basic-regex semantics, and a stale hook-exit-code comment have no reader outside this repository. This record files the three together under the intake cap."
last_verified: 2026-10-13
title: "Three small run-tooling gaps from run 20260922-1121, filed together"
waiting_on: build
---

# Three small run-tooling gaps from run 20260922-1121, filed together

## Context

`hw-run-policy`'s intake rule caps what a single pass sends to this register as its own full record, past which one record lists the rest. Run `20260922-1121` surfaced three small findings this way, none with a reader outside this repository. No two of them share a root cause, and none shares one with [HW-OBL-0202](0202-a-bare-none-footprint-is-claimed-as-a-literal-path-so-two-disjoint-issues-collide.md).

## Obligation

**Claim-through-the-board silently skipped two issues.** `gh project item-list` against board 1 (`PVT_kwDOEsVrw84BgGeU`) was checked directly on 2026-09-23. It has no item for #847 or for #855. #828, #817 and #971 — the run's other four merges — are all present. The mechanism `hw-build.md` names for moving a claimed issue onto the board did not run, or did not land, for these two. Nothing in the run's own log explains why. Worth checking whether this is systemic (issues filed a certain way, or moved out of a milestone by an earlier top-of-run pass) or two isolated gaps, before treating it as fixed by adding the two missing cards by hand.

**Interactive `grep -E` is not what it looks like in this sandbox.** `grep` is shell-function-shadowed to `ugrep -G`, which forces basic regular-expression semantics even when `-E` is passed. This silently broke a sanity check on run `20260922-1121` that assumed GNU `grep -E` semantics. Confirmed by `type grep` in the same session. A non-interactive `sh` script is unaffected. Only an agent reaching for `grep -E` directly in an interactive shell hits it.

**`.claude/hooks/intent.sh` documents a blocking behavior Claude Code does not have.** A comment in its shadow-log section, predating issue #971, claims a non-zero exit from this hook would block the prompt. Two separate agents independently fetched Claude Code's current hook documentation on 2026-09-22. They confirmed only exit code 2 blocks a `UserPromptSubmit` hook. #971 did not correct the comment because it was out of that change's scope.

## Discharge

The record is discharged on 2026-10-13, by the pull request for slice 1 of #1483. Each item is paid, or it does not reproduce.

**The board-item gap is paid by #1047.** Since `cd975fa3`, `tools/run/board-move.sh` finds the card with `gh project item-add`, which returns the card that exists or adds one. So a claim through that script cannot land without a card. On 2026-10-13, board 1 has a card for #847 at `Done`, and it has no card for #855. #855 closed on 2026-09-22, the day before #1047 merged, so its missing card is a record of the old gap and nothing reads it.

**The `grep` shadowing does not reproduce, and the skill line it asked for is not added.** On 2026-10-13, `type grep` in the agent's Bash tool shows a function that runs `ugrep -G` and then the caller's own options. A later `-E` overrides `-G`: `echo x | grep -E 'x+|y'` matches, and `echo ab | grep 'a|b'` does not. So `grep -E` keeps extended semantics. The same function also passes `--ignore-files` and `--exclude-dir=.git`, so a recursive `grep` skips the files that `.gitignore` names and the `.git` directory. That is a different trap, and this record does not carry it.

**The stale comment is paid by #1390.** Since `a0736315`, the shadow-log section of `.claude/hooks/intent.sh` says that exit 2 is the one status that blocks a `UserPromptSubmit` hook, and it cites this record.
