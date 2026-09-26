---
id: HW-OBL-0208
status: current
status_since: 2026-09-26
summary: "A collation opener class, a cache key's CLI wiring, a cache doc word and a recurring board-card gap, each with no reader outside this repository."
last_verified: 2026-09-23
title: "Four small test and wording gaps from run 20260923-0733 filed together"
waiting_on: build
---

# Four small test and wording gaps from run 20260923-0733 filed together

## Context

The intake of run `20260923-0733` held four findings that name no reader outside this repository. The product owner ruled each one Record, and `hw-run-policy` caps what one pass sends to this register, so the four sit in one record. They share no root cause with each other.

## Obligation

**The collation opener class holds a character that opens no command.** `tools/repo/collation-fixtures.sh` puts `}` in the class of characters that may precede a `sort` or `comm` invocation. POSIX puts only a redirection or an operator after `}`, so the member catches no real command, and it causes the false positive on `echo "${count} sort keys"`. The member predates PR #1042, whose adjudication kept it out of scope. The same PR added `&` to the class, which makes `echo "a & comm b"` a new false positive. The merge accepted that one. Both errors are in the loud direction.

**No test holds the CLI wiring of a cache-key component.** PR #1045 (#1029) put the compiled rule set into the check-rule cache key. If `rules_digest()` is frozen at both call sites in `engine/crates/cli/src/main.rs` (near lines 5140 and 5418), all 23 `headwater-cli` test binaries stay green. The verifier suspects that the wiring of the lock digest has the same gap, and did not check it. The missing test is a class, not a line: a CLI-level test that a warm cache misses after a key input moves.

**The cache module states the wrong component's job.** The module comment of `engine/crates/check/src/cache.rs` says that the rules component catches a rule that "changed". The per-rule `VERSION` does that job. The rules component catches a rule that was added or removed.

**The board-card gap of HW-OBL-0203 recurred.** In this run, `board-move.sh` found no project-board card for #1030, #974, #975, #1038 and #976, and remainder issue #1051 was filed without one. [HW-OBL-0203](0203-three-small-run-tooling-gaps-from-run-20260922-1121-filed-together.md) asks whether the gap is systemic. Six issues in one run is evidence that it is.

## Discharge

**The opener class** discharges when `}` leaves the class and a fixture holds the `echo "${count} sort keys"` line silent. The `&` member stays, by the ruling at the merge of PR #1042.

**The cache-key wiring** discharges when a CLI-level test sees a warm cache miss after each key input moves, the rule set and the lock digest among them, and the test fails when either call site is frozen.

**The cache comment** discharges when the module comment names addition and removal for the rules component and names `VERSION` for a changed rule.

**The board-card recurrence** discharges with HW-OBL-0203's board-item gap, and adds nothing to that record's Discharge clause.
