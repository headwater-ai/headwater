---
id: HW-OBL-0240
status: current
status_since: 2026-10-13
summary: "A workflow sits outside the code path scope, two install-route scripts have no governing document, three judge mutants survive, the write hook names a link path, and the verify wait omits --repo. None has a reader outside this repository."
last_verified: 2026-10-13
title: "Five run-tooling gaps from run 20261012-2342, filed together"
waiting_on: build
provenance:
  warrant: asserted
---

# Five run-tooling gaps from run 20261012-2342, filed together

## Context

`hw-run-policy` sends a finding about the build order, its tooling or the CI of this repository to this shelf. Run `20261012-2342` wrote these as intake lines from the work on #1490, #1315, #1433 and #1561. The product owner ruled each one RECORD, four at the fifth-merge pass and one at the final pass, both on 2026-10-13. Each item was checked on `56febd34`. The product owner read the `anchors.code_path.scope` list in `.headwater/overlay.yml`, the `schedule` and the `readme-apt` job of `.github/workflows/install-routes.yml`, the `judge` function of `tools/repo/release-guide-fixtures.sh`, the `advisory` text of `.claude/hooks/write.sh`, and the mergeable wait in `.claude/agents/hw-verify.md`.

## Obligation

- `.github/workflows/install-routes.yml` (#1490) is a second workflow outside `anchors.code_path.scope`, which names `tools/**` and `site/**` and no path under `.github/`. [HW-OBL-0227](0227-seven-run-tooling-gaps-from-run-20260929-2201-filed-together.md) records the gap with `readme-apt.yml` as its example. This is one more instance, and the remedy there covers it.
- `tools/repo/readme-install-block.sh` and `tools/repo/install-route-issue.sh` sit inside `tools/**`, and no document under `docs/` or `.headwater/` names either. The maintainer of #1490 asks whether a process decision is owed for the rule that one scheduled workflow runs every README route and opens one bug issue.
- The `judge` function of `tools/repo/release-guide-fixtures.sh` lets mutants J1, J6 and J7 survive, and it accepts `sha256:zz` as a digest (the #1315 verifier, report 3). A `t9` arm with `sha256:zz` that expects "carries no verified_revision" holds J7.
- For an edit through a symbolic link inside the root, the `advisory` text of `.claude/hooks/write.sh` says a document "declares that it governs" the link path, though the edge names the file that the link reaches. `headwater route` names the real path. #1648 replaces the shell hooks, so it is the natural change to carry the fix.
- The mergeable wait in `.claude/agents/hw-verify.md` runs `gh pr view <N>` with no `--repo`. From an agent worktree the call fails with "Could not resolve to a PullRequest", and the `!= UNKNOWN` test then passes on the error text (round 2 of the verify of PR #1741). `hw-run-policy` lists `gh issue view` and `gh pr edit` as calls that fail this way and does not list `gh pr view`.

## Discharge

Each item discharges alone. The first does when HW-OBL-0227 discharges. The second does when a document governs the two scripts, or a process decision states the rule they implement. The third does when a case kills J1, J6 and J7. The fourth does when the hook, or what replaces it, names the real path. The fifth does when the wait names `--repo headwater-ai/headwater` and refuses a wait on an error. The record discharges when every item has.
