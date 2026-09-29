---
id: HW-OBL-0224
status: current
status_since: 2026-09-29
summary: "Run 20260928-1109 surfaced thirteen findings about its own agents, hooks, run scripts and CI, none with a reader outside this repository. This record files them together under the intake cap."
last_verified: 2026-09-29
title: "Thirteen run-tooling gaps from run 20260928-1109, filed together"
waiting_on: build
---

# Thirteen run-tooling gaps from run 20260928-1109, filed together

## Context

`hw-run-policy` caps what one pass sends to the register as full records. Past the cap, one record lists the rest. Run `20260928-1109` wrote thirteen intake lines about the build order, its tooling and this repository's CI. The product owner ruled each one RECORD, on passes 2 to 9 of that run. None names a reader outside this repository. Each item below gives the stage that found it and the issue in hand at the time.

## Obligation

**The agents of the build order.**

- `hw-queue` cannot write `queue.md` into the run directory from a worktree-isolated session, because the harness refuses the path under the common directory. `tools/run/run-dir.sh` has no `queue <dir> <file>` verb, so the parent copied the file with a script. (parent, no issue)
- `CLAUDE.md` tells every session to call `EnterWorktree`. A worktree-isolated parent passes that isolation to `hw-integrate`, and the harness then refuses its commands in the shared checkout. So the move, rebuild, regenerate and bless step of `.claude/agents/hw-integrate.md` cannot run in a `/next-run` session. The fix is either an exemption in `CLAUDE.md`, which is the owner's file, or a post-merge step on the integrator's own worktree at `origin/main`. (parent, #764, seen on #1299)
- `.claude/agents/hw-build.md` ("Before you open the pull request") tells a builder to rebase onto `origin/main`. After a push, the rebased branch cannot land without the force-push that the same definition forbids. #1311 merged `origin/main` instead. (build, #1311)
- `hw-adjudicate` switched the parent worktree to `origin/main` and back, and left `HEAD` detached. An adjudicator must not move a checkout that it did not make. (adjudicate, #978)
- The mergeable wait in `.claude/agents/hw-verify.md`, `[ "$(gh pr view <N> --json mergeable -q .mergeable)" != UNKNOWN ]`, is met on an error. `gh pr view 1364` failed on this repository with "Could not resolve to a PullRequest", and the empty string is not `UNKNOWN`. `gh pr list --head <branch> --json mergeable,mergeStateStatus` works. (hw-verify, #1357)

**The run scripts and hooks.**

- `tools/run/queue-done.sh` printed `UNMERGEABLE` for #1328 for about nine minutes before the merge queue ejected it with `merge_conflict`. At the same time `gh pr view` showed CLEAN and green. Nothing records `UNMERGEABLE` as a sign of a coming ejection. (hw-iterate, #1348)
- `tools/run/queue-done.sh` printed "ejected: merged" for PR #1353, which merged as `284e056b`. An integrator that trusts it reports a false ejection. (integrate, #1334)
- `.claude/hooks/write.sh` (about lines 227-232) refuses a raw `Write` only on the "no document written there yet" answer of `headwater explain`. After #1367, a path through a dangling link out of the root gets the outside answer, so the hook advises there and does not refuse. No document says whether that is intended. (build, #1367)
- `tools/site/render-tutorial.py` and `tools/site/render-tutorial-fixtures.sh` are in the governed scope, `headwater route` says that nothing governs them, and no document names them. (hw-build, #1348)

**Parallel branches and CI.**

- The engine's check rule registry holds a hand-kept `RULES` count that each new rule bumps. Two branches that each add a rule compile alone and fail together. This cost two extra rounds and one queue ejection (#1311 against #1213, then against #1232). (integrate, #1311)
- Seven transcripts under `docs/probe-results/` record digests that nearly every merge moves. They are the campaign-pilot and second-campaign-pilot discovery arms of 2026-09-28, and the regression-probe transcripts of 2026-09-16 and 2026-09-17. Two branches that regenerate them conflict. In this run they caused four ejections or refusals (#1328 twice, #1349, #1356, #1362). They behave as a derived fold, and they carry no merge driver. (integrate, #1328)
- `headwater route` ranks HW-DR-0052 fifth of 445, the last slot of the default budget, by a margin of 12%. One new decision on identifiers or pull requests with a heavy summary moves it out and turns the route fixture red. The recorder case matches an exact phrase of the probe's task, and nothing holds a probe's expected value against the prose (PR #1362). (verify, #1294)
- The ten documents that govern `.github/workflows/ci.yml` report `relation.target.suspect` on `origin/main` before #978 changed anything: 32 warnings on both trees. They are HW-OBL-0173, HW-PD-0013 to HW-PD-0020 and the evaluation of the two triggers. Each owes a new reading against the workflow. (build, #978)

## Discharge

Each item discharges alone, when the file it names says or does what the item asks, or when a change records why it stays. The record discharges when every item has. An item that grows a reader outside this repository leaves this record for an issue, and the record says where it went.
