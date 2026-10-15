---
id: HW-OBL-0208
status: current
status_since: 2026-09-26
summary: "Of four small gaps from one run, three are closed, and the recurring board-card gap waits on HW-OBL-0203."
last_verified: 2026-09-23
title: "Four small test and wording gaps from run 20260923-0733 filed together"
waiting_on: build
---

# Four small test and wording gaps from run 20260923-0733 filed together

## Context

The intake of run `20260923-0733` held four findings that name no reader outside this repository. The product owner ruled each one Record, and `hw-run-policy` caps what one pass sends to this register, so the four sit in one record. They share no root cause with each other.

## Obligation

**The board-card gap of HW-OBL-0203 recurred.** In this run, `board-move.sh` found no project-board card for #1030, #974, #975, #1038 and #976, and remainder issue #1051 was filed without one. [HW-OBL-0203](../process/obligations/0203-three-small-run-tooling-gaps-from-run-20260922-1121-filed-together.md) asks whether the gap is systemic. Six issues in one run is evidence that it is. This is the one gap of the four that is still owed.

The first slice of [#1494](https://github.com/headwater-ai/headwater/issues/1494) closed the other three on 2026-10-15.

- **The collation opener class.** `}` left the class in `tools/repo/collation-fixtures.sh`, and a new arm holds `echo "${count} sort keys"` silent and a brace group piped to `sort` loud. With `}` put back, that arm fails. The `&` member stays, by the ruling at the merge of PR #1042.
- **The cache module comment.** The module comment of `engine/crates/check/src/cache.rs` says that the rules component catches a rule that was added or removed. It says that the per-rule `VERSION` catches a rule that changed.
- **The cache-key wiring.** The lock half is held through the binary. The case `a_warm_check_gives_the_cold_verdict_after_the_lock_moves` is in `engine/crates/cli/tests/cache_key.rs`. It retires a term in the taxonomy and changes no document. Then it holds that a warm `check` gives the report of a cold one. It fails when the lock digest is frozen where the CLI builds its cache. The rule-set half cannot be held this way, because a test cannot change the rule set of the binary that it runs. So it is held by structure. `Cache::compiled` reads `rules_digest` itself, and the CLI builds its cache in `Loaded::cache` alone. A case in `engine/crates/cli/tests/wiring.rs` holds both. It fails on a call of `Cache::at`, on the name `rules_digest`, or on a second constructor call. `check --fix` builds its cache through the same `Loaded::cache`. A fix run with a frozen key alone moves no verdict, because the report run after it writes the cache file again. So no test of the binary could show that call site frozen. The one constructor call is what holds it.

## Discharge

This record discharges with the board-item gap of HW-OBL-0203, and adds nothing to the Discharge clause of that record.
