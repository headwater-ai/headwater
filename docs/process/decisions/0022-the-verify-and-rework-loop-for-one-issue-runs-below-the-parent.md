---
id: HW-PD-0022
status: current
status_since: 2026-09-27
summary: "One agent per issue, hw-iterate, dispatches the builder and each verifier and sends every FAIL back to the same builder. The parent wakes once per issue, rules the final PASS, and never reads the branch with git show or git diff."
last_verified: 2026-09-27
title: "The verify and rework loop for one issue runs below the parent"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  governs:
    - .claude/agents/hw-iterate.md
    - .claude/commands/next-run.md
    - .claude/commands/next.md
---

# The verify and rework loop for one issue runs below the parent

## Context

Before this decision, the parent of a build-order run dispatched `hw-build`, then `hw-verify`, and it sent each `FAIL` back to the builder itself. Every one of those steps woke the parent at its full context. [HW-PD-0003](0003-a-dispatch-pays-when-it-retires-more-parent-turns-than-it-costs.md) makes one parent turn at full context the unit of cost.

Run `20260927-0443` measured the cost of that loop. The verifiers returned 56 verdicts: 38 `FAIL` and 18 `PASS`. There were 91 verify dispatches and 105 build reports for 48 builds. Each `FAIL` cost about 6 parent calls at about 180k tokens each, so about 1M parent tokens for each `FAIL`. That is about 15% of the tokens that the parent read in the run. The parent woke about 8 times for each merged issue.

The parent also read branches itself. It made about 44 `git show` or `git diff` calls against build branches, which put about 97k characters into its context. The parent then read those characters again on every later call of the run.

A subagent can dispatch a child with `isolation: "worktree"`, and it can resume that child by `SendMessage` to the child's id. This was measured on 2026-09-28, in the build of #1276. A subagent dispatched a child in its own worktree, and the child kept a word. The subagent then sent the child a question with `SendMessage`, and the resumed child answered with that word. `SendMessage` was a deferred tool in that subagent, and the subagent loaded it with `ToolSearch`.

## Decision

The owner ruled on the issue on 2026-09-28:

> the per-issue manager rules on intermediate verdicts, and the parent keeps the final veto.
>
> - The manager owns the build → verify → rework loop for one issue. A `FAIL` goes back to the builder by the manager's own `SendMessage`, with the verifier's finding and nothing added, as the veto does today. The parent is not woken for it.
> - The manager reports to the parent once: a final `PASS` with what the last verify proved and every earlier round's finding, or a stop.
> - The parent rules on that final report. It may still veto a `PASS`, and a veto goes back to the manager, not to the builder.

The manager is `.claude/agents/hw-iterate.md`. The parent dispatches it after `hw-adjudicate` returns `BUILD` and the footprint is claimed. The parent keeps adjudication, because it rules the kinds of refusal and it makes the claim.

`hw-iterate` dispatches `hw-build` and each `hw-verify` with `isolation: "worktree"`. It chooses the attacks for each verify from `hw-verification-bar`. It sends each `FAIL` to the same builder by `SendMessage`, with the finding verbatim. It never dispatches a second builder for the issue, because a fresh builder throws away a settled design. It dispatches a fresh verifier for each round, so that no verifier checks a repair that it asked for.

`hw-iterate` stops at the third `FAIL` of one issue and reports `STOP`. The parent rules a `STOP` as it rules a refusal. A parent veto of a `PASS` goes to `hw-iterate`, and it counts toward the three.

`hw-iterate` never merges, never enqueues, never moves a board card and never files an issue. It reads verdicts and notes, and it never reads the branch with `git show` or `git diff`.

The parent never runs `git show` or `git diff` against a build branch. It rules from the report of `hw-iterate`, from `build.md` and from `verify-report.md`.

Two shapes were considered. In the first shape, the builder dispatches its own verifier. That shape was rejected, because the verifier is then no longer independent of the agent whose work it checks. In the second shape, one manager per issue dispatches both. This decision takes the second shape.

## Consequences

The parent wakes for the adjudicate report, for the `hw-iterate` report, and for its share of an integrator report. The target is 3 parent wakes or fewer for each merged issue. Only a run after this change can measure that number, and #1276 stays open until one does. If the number is still above 3, this decision is reopened.

The veto of [HW-PD-0004](0004-coordination-is-a-create-only-claim-and-authority-stays-on-the-tree.md) stays on the tree. `hw-iterate` is the parent of its builder and its verifiers, so its `SendMessage` flows down the tree and is not a message from a peer.

`.claude/commands/next-run.md`, `.claude/commands/next.md`, `.claude/run/doctrine.md`, `hw-run-policy`, `hw-build` and `hw-verify` state the new loop. The check `df -h /` before each verify moves from the parent to `hw-iterate`. Case 10 of `.claude/agents/fixtures.sh` holds the shape: the parent names `hw-iterate` and neither of the two stages that it dispatches.
