---
id: HW-OBL-0233
status: current
status_since: 2026-10-03
summary: "Run 20261002-1233 left thirteen findings about the self-hosted CI runner, stale counts in CI records, shadow-log scripts and the build-order agents. None has a reader outside this repository."
last_verified: 2026-10-03
title: "Thirteen run-tooling gaps from run 20261002-1233, filed together"
waiting_on: build
provenance:
  warrant: asserted
---

# Thirteen run-tooling gaps from run 20261002-1233, filed together

## Context

`hw-run-policy` sends a finding about the build order, its tooling or the CI of this repository to this shelf. Run `20261002-1233` wrote these intake lines, and the product owner ruled each one RECORD. Intake lines 11, 12, 13, 15, 19, 22, 48, 59, 73, 89, 90, 99 and 124 were ruled during the run. Lines 79, 80, 84, 85, 96 and 105 were ruled at the top of run `20261003-1026`. Each item was checked on `a46bd95e`, and each names the file and the symbol that was read. Two lines of the run are not here. HW-PD-0020 no longer says "No job is skipped on the new event", so lines 17 and 18 are withdrawn. HW-PD-0021 names a newer digest for `.claude/skills/hw-run-policy/SKILL.md`, so line 12 holds for HW-PD-0007 alone.

## Obligation

**The self-hosted CI runner.**

- The merge-group job `headwater check (advisory)` in `.github/workflows/ci.yml` failed twice on `sccache rustc -vV` with "Timed out waiting for server startup", ten seconds into the step that builds the engine. One failure ejected a verified pull request. The job names no `SCCACHE_STARTUP_TIMEOUT` and no `RUSTC_WRAPPER`, and `tools/ci/toolchain.sh` names neither. The wrapper comes from the environment of the runner, and nothing in this repository starts it or bounds its startup. The fix is a startup timeout in the runner environment, a retry of the build step, or a build with no wrapper when the server does not start.

**Counts and sentences in the CI records that are no longer true.**

- HW-PD-0007 declares `governs` onto `.claude/skills/hw-run-policy/SKILL.md` with `verified_revision` `sha256:05bbe166…`, which the run reported as suspect. A re-read of HW-PD-0007 against the skill restamps the edge or amends the record.
- The Context of HW-PD-0017 says "A run is two jobs". That holds for the self-hosted pool only. The evaluation that HW-PD-0017 rests on says that each job condition opens with `!cancelled()`, and the `deploy` job does not. A contributor who restores the old prose behind a valid stamp turns nothing red.
- The Context of HW-OBL-0145 says "The job carries 55 steps". The job carried 75 steps when line 19 was measured, and #927 added the step that runs `tools/run/shadow-mine-fixtures.sh`. The record declares no `governs` edge, so no check sees the count drift.
- The Context of HW-OBL-0173 says that `.github/workflows/ci.yml` "holds 40 `run:` steps over 1022 lines, and five of its lines carry `exit 1`". The file has 2270 lines on `a46bd95e`.

**Stamps and governed scope.**

- A stamp taken on a branch older than `main` reverts a newer stamp at the squash. #1614 wrote `verified_revision` for two edges of spec 17 against the copies on its own branch, so `main` carried digests that no tree of `main` held. Both edges went suspect when it merged. Nothing in `check --change` or the merge queue compares a stamp with the content on the tip of the merge group. The cost is a suspect finding and a re-read, not a silent pass.
- `tools/run/shadow-mine.sh`, `tools/run/shadow-mine-fixtures.sh`, `tools/ci/governed-scope.jq` and `tools/ci/governed-scope-fixtures.sh` are in the governed scope of `tools/**`, and no document declares `governs` onto any of them. The mining how-to names `shadow-mine.sh` in prose only. HW-OBL-0231 records the same gap for `tools/ci/toolchain.sh`.
- The edit hook in `.claude/hooks/lib.sh` says that `tools/probe/probe-record-fixtures.sh` "is in the governed scope, and nothing governs it". `headwater explain` says that the path is outside every corpus root. The hook and the engine disagree about the path, so a contributor meets a warning that no edge can clear.
- No rule says whether the `last_verified` of a process obligation moves when a change edits its Discharge. HW-OBL-0224 and HW-OBL-0225 keep `last_verified: 2026-09-30` after an edit on 2026-10-03. This item waits on a ruling and not on a build.

**Tests of the run tooling.**

- Four boundary mutants of `tools/run/shadow-mine.sh` survive `tools/run/shadow-mine-fixtures.sh`: p3, p10, p13 and p15, which the verify report of #927 lists. No case tests whether the nine restamped `governs` edges onto `.github/workflows/ci.yml` go suspect again on a later edit of the workflow.
- The intent hook `.claude/hooks/intent.sh` skips a prompt that opens with `<task-notification>` or `<cross-session-message`, and not one that opens with `<agent-message from=`. 314 of 937 distinct prompt identifiers in the shadow log from 2026-09-20 to 2026-10-03 are such envelopes. So a raw count of 1,020 lines reads as past the bound of #927, and the joined count of person prompts is 587.

**The agents of the build order.**

- A builder of #1631 pushed its rebased branch with `git push --force-with-lease`. `.claude/agents/hw-build.md` says "You never merge, and you never force-push." The branch was its own and had no open pull request, and a later round merged `main` instead.
- The first live session through the plain-HTTP route of #1467 cost 6.17 cents, against the one cent that the adjudication note stated. `.claude/agents/hw-build.md` does not ask a build note to state a spend estimate before a live session runs.

## Discharge

Each item discharges alone, when the file it names says or does what the item asks, or when a change records why it stays. The record discharges when every item has. An item that gains a reader outside this repository leaves this record for an issue, and the record says where it went.
