---
id: HW-PD-0021
status: current
status_since: 2026-09-28
summary: "A stage runs its bounded wait in the foreground and keeps its turn. A background wait ends the stage's turn, and each attempt then wakes the parent at its full context."
last_verified: 2026-09-28
title: "A subagent waits in the foreground, because a background wait wakes its parent"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude
  activity: measure+draft
  evidence_basis: evidenced
relations:
  governs:
    - .claude/hooks/wait.sh
    - tools/run/wait-for.sh
    - .claude/skills/hw-run-policy/SKILL.md
---

# A subagent waits in the foreground, because a background wait wakes its parent

## Context

[HW-PD-0007](0007-a-background-wait-caps-below-the-cache-lifetime-and-re-issues-itself.md) caps a wait at four minutes, so that the wait ends before a subagent's five-minute prompt cache expires. It also starts each bounded call with `run_in_background: true`. That second part has a cost that HW-PD-0007 did not measure.

A stage whose only work left is a background wait ends its turn. The harness then sends the parent a task notification that says the result "may be interim". The parent reads that notification in a full turn at its whole context, and it has nothing to do. Each re-issue of the wait repeats this every four minutes, for each stage that waits.

Session `b5554ef1` was the parent of run `20260927-0443`. It ran for 18 hours and made 1,368 API calls. The parent read 245M context tokens, an average of 179k tokens for each call. About 310 of its wakes came from interim notifications, and they used about 70M tokens, which is 28% of all that the parent read. The integrator of one batch woke the parent 25 times. The release agent woke it 21 times, and single builders woke it up to 19 times. A typical reply was "The #1053 fork has restarted its CI wait again. Nothing to act on yet."

## Decision

A stage runs `sh tools/run/wait-for.sh '<condition>'` in the foreground, with a Bash `timeout` of `300000`. The script already ends inside four minutes, so the call never reaches the ten-minute foreground cap. On `RE-ISSUE`, the stage runs the identical call again in the foreground. The stage keeps its turn for all of the wait, and the parent hears from it one time, when it reports.

A stage never starts a wait with `run_in_background: true`. A long job that is not a wait, such as a build or a test suite, still goes in the background. The stage then waits for the exit marker of that job with `wait-for.sh` in the foreground.

The four-minute cap of HW-PD-0007 does not change. Each foreground attempt returns before the subagent cache expires, so the next turn reads the cache and does not write it again. The parent does not wait at all: it ends its turn, and the report of the stage wakes it.

## Consequences

`hw-run-policy`, `hw-build`, `hw-verify` and `hw-integrate` state the foreground form. These three stages are the agents that run `wait-for.sh`.

`hw-iterate` ([HW-PD-0022](0022-the-verify-and-rework-loop-for-one-issue-runs-below-the-parent.md)) runs no `wait-for.sh`. It waits on a live child agent, the builder or a verifier, and it waits by ending its turn. This decision does not apply to that wait. A wait on a child agent is not a background shell wait, and the completion of the child wakes `hw-iterate` one time. The builder and the verifier below `hw-iterate` still wait in the foreground, so that they do not wake `hw-iterate` at each re-issue. The refusal text of `.claude/hooks/wait.sh` names the foreground form and this cost. The header of `tools/run/wait-for.sh` states the same.

In both forms, each re-issue costs one subagent turn. The saving is the parent turn that each re-issue caused. For run `20260927-0443`, that was about 310 parent turns at about 200k tokens each.

The next run measures this decision. If interim notifications from a waiting stage are still a large share of the parent's tokens, this decision is reopened.
