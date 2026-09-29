---
id: HW-PD-0021
status: current
status_since: 2026-09-28
summary: "A stage runs its bounded wait in the foreground and keeps its turn. A background wait ends the stage's turn, and each attempt then wakes the parent at its full context."
last_verified: 2026-09-29
title: "A subagent waits in the foreground, because a background wait wakes its parent"
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  governs:
    - to: .claude/hooks/wait.sh
      verified_revision: sha256:e36e5d743d4167c4762a7a4730cf3164fbe23634a07bc7d8018236e517c7152e
    - to: tools/run/wait-for.sh
      verified_revision: sha256:d1e99759d7f8c50c387018ac8837f7005aee28c999952a29be977e3392bf2e3e
    - to: .claude/skills/hw-run-policy/SKILL.md
      verified_revision: sha256:4fb63f39a523bf9dd45dbf59caff456f2eb5203fb4cdf93ba0f9ebdd63643f8e
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

`hw-run-policy`, `hw-build`, `hw-verify` and `hw-integrate` state the foreground form. These three stages are the agents that run `wait-for.sh`. `.claude/hooks/wait.sh` refuses a wait that a subagent starts with `run_in_background: true`. It knows a subagent by the `agent_id` field of the hook payload, which the harness sets only on a call from a subagent. A person's own session and the parent of a run have no parent to wake, so their background waits pass (ruled by the parent of the run, 2026-09-28). The hook also refuses a foreground `wait-for.sh` whose Bash `timeout` is under `300000`, in every session. Its refusal text and its header state this cost. The header of `tools/run/wait-for.sh` states the same, and the script bounds each run of the condition by the time left under its cap.

`hw-iterate` ([HW-PD-0022](0022-the-verify-and-rework-loop-for-one-issue-runs-below-the-parent.md)) runs no `wait-for.sh`. It waits on a live child agent, the builder or a verifier, and it waits by ending its turn. This decision does not rule on that wait. [HW-PD-0022](0022-the-verify-and-rework-loop-for-one-issue-runs-below-the-parent.md) asserts that `hw-iterate` does not wake the parent when it ends its turn with a live child. Run `20260928-1109` measured that it does not ([the evaluation of the build order](../evaluations/the-build-order-as-a-multi-agent-system.md#what-the-first-run-with-the-loop-below-the-parent-measured)). The measurement above points the other way for a background shell wait, which is a different wait. The builder and the verifier below `hw-iterate` still wait in the foreground, so that they do not wake `hw-iterate` at each re-issue. A fresh builder that `hw-iterate` dispatches from a handover file waits in the same way.

A drain of [HW-PD-0023](0023-a-build-order-parent-restarts-every-few-merges-drains-to-zero-first-and-resumes-from-the-handover-files-on-disk.md) does not change this rule. A stage that waits in the foreground completes its wait and its stage. `hw-iterate` then reads `DRAIN` at the stage boundary that follows, and it returns `HANDOVER`. The integrator that is in flight completes its foreground wait on the merge queue before the parent exits.

In both forms, each re-issue costs one subagent turn. The saving is the parent turn that each re-issue caused. For run `20260927-0443`, that was about 310 parent turns at about 200k tokens each.

The next run measures this decision, from the parent transcript, as session `b5554ef1` was measured. If interim notifications from a waiting stage are still a large share of the parent's tokens, this decision is reopened. The parent can wake on an interim notification from `hw-iterate` while its child is live. In that case the claim of HW-PD-0022 is false, and `hw-iterate` waits some other way.
