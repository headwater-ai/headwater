---
id: HW-OBL-0227
status: current
status_since: 2026-09-30
summary: "Run 20260929-2201 surfaced seven findings about its claims, hooks, kills, pull request bodies and CI workflows. None has a reader outside this repository, so this record files them together."
last_verified: 2026-09-30
title: "Seven run-tooling gaps from run 20260929-2201, filed together"
waiting_on: build
---

# Seven run-tooling gaps from run 20260929-2201, filed together

## Context

`hw-run-policy` sends a finding about the build order, its tooling or this repository's CI to this shelf, and spec 13 lists none of them. Run `20260929-2201` wrote seven such intake lines. The product owner ruled each one RECORD, on passes 2, 3 and 4 of that run, and no pass had the time to write them. None names a reader outside this repository. Each item below gives the stage that found it and the issue in hand at the time. Where this record measured an item again on `origin/main` at `8d9feed4`, the item says so.

## Obligation

**The run scripts and hooks.**

- `claim` in `tools/run/run-dir.sh` stores each artifact as one file named by its slug, and it matches nothing else. So a glob claim and a literal path that the glob admits never meet. #1337 claimed `engine/crates/adapter/tests/fixture.*`, and the claim of `engine/crates/adapter/tests/fixture.json` by #1417 printed `CLAIMED`, not `HELD`. [HW-OBL-0202](0202-a-bare-none-footprint-is-claimed-as-a-literal-path-so-two-disjoint-issues-collide.md) records the same store from the other side, where two unrelated claims collide. (parent, #1417)
- `hw_governed_by_document` in `.claude/hooks/lib.sh` passes `--paths-at-most` to `headwater explain`, and it returns at once when the call fails. An engine built before #1346 (`1e791176`) refuses the option, so the edit advisory is silent until the clone rebuilds its engine. The hook could retry without the option on that refusal, or name the rebuild. (parent, #1346)

**The agents of the build order.**

- The builder of #1438 ran `pkill -f` on a `rustfmt` command line. It stopped the format check of the #1347 verifier (`HW_CARGO_SLOT=verify-1347`, pid 1608635), so that check had to run again. A kill by a command-line pattern cannot tell the job of one stage from the job of another. Before a stage kills a process, `hw-run-policy` tells it to check that the process is not the live build of another stage. It says nothing against a kill by pattern. (build, #1438)
- The body of PR #1419 quoted the negated keyword "Neither closes #1384", and GitHub closed #1384 when the owner merged it by hand from `run-lessons-20260929`. That pull request added the warning to `.claude/agents/hw-build.md`. The warning reaches a builder alone, and the read of `closingIssuesReferences` in `.claude/agents/hw-integrate.md` reaches only a merge that the integrator makes. Nothing reads the body of a pull request that a person opens or merges. (parent, #1384)

**The fixtures.**

- The case "every cited path is on this tree" in `.claude/skills/fixtures.sh` reads only the skill files of the tree it runs on. No case gives it a path that is missing. The #1418 verifier inverted its first `git check-ignore` test, and the suite stayed green. So a cited path that is missing and not ignored would pass. (verify, #1418)

**CI and its workflows.**

- `.github/workflows/` is outside every corpus root and outside `anchors.code_path.scope` in `.headwater/overlay.yml`. On `8d9feed4`, `headwater explain` refuses both `.github/workflows/readme-apt.yml` and `.github/workflows/release.yml` as outside every corpus root. So nothing reports that no document governs `readme-apt.yml`, which #1408 added. Seven records govern `release.yml` by an edge all the same. The scope could name `.github/workflows/**`, or name the workflows one by one. (build, #1408)
- An edge that governs a workflow, an agent definition or a skill goes suspect when that file changes. No build of this run refreshes the stamp, because its footprint does not name the record. On `8d9feed4`, `headwater check` reports 32 `relation.target.suspect` findings on 24 documents. Sixteen are edges onto `release.yml` and `ci.yml`, and twelve are edges onto files under `.claude/`. The intake line gave 652 instances, and this record could not reproduce that figure. `headwater change --verified` (#1392) is the route that stamps an edge again, and a person has to read each document first. [HW-OBL-0224](0224-thirteen-run-tooling-gaps-from-run-20260928-1109-filed-together.md) records the documents that govern `ci.yml`, and this item widens that item to every file outside the engine. (build, #1408)

## Discharge

Each item discharges alone, when the file it names says or does what the item asks, or when a change records why it stays. The record discharges when every item has. An item that grows a reader outside this repository leaves this record for an issue, and the record says where it went.
