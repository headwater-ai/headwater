---
id: HW-OBL-0225
status: current
status_since: 2026-09-29
summary: "Run 20260929-1205 surfaced eight findings about its own agents, hooks, run scripts and CI, none with a reader outside this repository. This record files them together under the intake cap."
last_verified: 2026-09-30
title: "Eight run-tooling gaps from run 20260929-1205, filed together"
waiting_on: build
---

# Eight run-tooling gaps from run 20260929-1205, filed together

## Context

`hw-run-policy` caps what one pass sends to the register as full records. Past the cap, one record lists the rest. Run `20260929-1205` wrote eight intake lines about the build order, its tooling and this repository's CI. The product owner ruled each one RECORD, on the fifth-merge pass, the tenth-merge pass and the end-of-run pass of that run. None names a reader outside this repository. Each item below gives the stage that found it and the issue in hand at the time. Three of the eight items are discharged, and the Discharge section states them. Five items stay open.

## Obligation

**The agents of the build order.**

- The adjudication note for #1339 said HW-DR-0097 supersedes HW-DR-0047. The front matter of HW-DR-0097 declares `constrains`. The build copied the error, and the maintainer caught it. Nothing tells an adjudicator to read a relation it names from the front matter. (build, #1339)
- The Done-when of #1386 named `headwater neighbors` as a graph check. `neighbors` ranks summaries by meaning and reads no declared edge (`docs/interfaces/headwater-neighbors.md`). An issue that asks whether two documents are linked should name `headwater explain`, and no instruction to an agent that writes a Done-when says so. (build, #1386)

**The run scripts and hooks.**

- `hw_engine()` in `.claude/hooks/lib.sh` looks only for `engine/target/release/headwater` and `engine/target/dev-release/headwater`. In a tree whose only engine is `engine/target/debug/headwater`, which `tools/hw-cargo test` leaves, the write-time hook fails open and prints nothing. (build, the red-main fix under #1366)

**Parallel branches and CI.**

- An edit to `docs/spec/12-check-layer.md`, `docs/interfaces/headwater-mcp.md` or the governs-edges evaluation moves a read-set digest in nine files under `docs/probe-results/`. They are the `regression-probe-transcript-*` files and the campaign pilot of 2026-09-28. So a footprint of documents alone that names no probe result still regenerates them. The #1340 adjudicator's footprint missed them. HW-OBL-0224 records the same transcripts as a merge-conflict source. (build, #1340)

## Discharge

Each item discharges alone, when the file it names says or does what the item asks, or when a change records why it stays. The record discharges when every item has. An item that grows a reader outside this repository leaves this record for an issue, and the record says where it went.

Three items are discharged by #1419, which merged as `16eecca0`:

- The integrator had to load the `hw-run-policy` skill, but its `tools` list had no Skill tool, so the #1389 integrator ran without the policy. The `tools` line of `.claude/agents/hw-integrate.md` now names `Skill`. (integrate, #1389)
- A worktree-isolated parent passes its isolation to `hw-integrate`, which then cannot move the shared checkout. A detached tree under `.claude/worktrees/` worked for the post-merge rebuild, `generate`, `check --strict` and bless, and no instruction named that route. `.claude/agents/hw-integrate.md` and `hw-run-policy` now name it. HW-OBL-0224 (its second item) and HW-OBL-0221 carry the same gap, and this record does not discharge them. (integrate, #1368)
- A pull request body that said "Neither closes #1384" made GitHub list #1384 in `closingIssuesReferences`, because the keyword matches inside a negation. No build agent's instructions warned against it. `.claude/agents/hw-build.md` now says that a `Refs` pull request carries no closing keyword, not even in a negated sentence. This repository has no pull request template. (parent, #1384)

One item is discharged by #1518, the pull request for #1484:

- `tools/run/queue-done.sh` printed "not queued: never removed from one" for PR #1412. The GraphQL timeline holds a `RemovedFromMergeQueueEvent` with reason `failed_checks` at 2026-09-29T19:09:58Z. HW-OBL-0224 records two earlier misreports of this script. The script read removals alone, so it could not tell a pull request never queued from one whose removal was not visible yet. It now reads the added and removed events together, and an add with no entry and no later removal holds the wait. (integrate, #1412)
