---
id: HW-PD-0023
status: current
status_since: 2026-09-28
summary: "A loop script restarts the build-order parent when a call passes 130k tokens of context or the session reaches four merges. The parent drains to zero before it exits, and the next session resumes from the handover file of each issue."
last_verified: 2026-09-28
title: "A build-order parent restarts every few merges, drains to zero first, and resumes from the handover files on disk"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  governs:
    - to: tools/run/supervise.sh
      verified_revision: sha256:f791fc4db614b65541ae5274876e42c60090ceada49f3ce1976490ff6d5ff80a
    - to: tools/run/supervise-fixtures.sh
      verified_revision: sha256:da6aa4d501707f8d3d2be107a3e11b4efcc434f34db7a0ec55e6dac00b7c8c98
---

# A build-order parent restarts every few merges, drains to zero first, and resumes from the handover files on disk

## Context

The parent of a build-order run reads its whole context again on every call. [HW-PD-0003](0003-a-dispatch-pays-when-it-retires-more-parent-turns-than-it-costs.md) makes one parent turn at full context the unit of cost. A parent that runs a whole build order in one session therefore pays more for each call than the call before it.

Run `20260927-0443` measured that cost in session `b5554ef1`. The parent made 1,368 calls and read 245M tokens of context, a mean of 179k for each call. Its first call was 52k. Its context grew by about 1.2k for each call between compactions, and each of the 7 compactions was made by hand. The parent made about 30 calls for each closed issue. The owner pays the 5-hour and 7-day windows of the plan, and those windows cap how many issues a run can close.

A subagent runs inside the process of its parent and ends with it. A test with a headless `claude -p` parent showed that nothing reattaches to a new session. Only what is on disk survives a restart. Issue #1275 sets the bar: a mean under 120k for each call, over all parent sessions of one run.

The same run measured the time of each stage, as median and 75th percentile. Adjudicate took 2.8 and 3.4 minutes. A first build took 27.8 and 37.9 minutes, and a rework build took 15.6 and 24.1 minutes. Verify took 6.0 and 10.0 minutes, and integrate took 29.2 and 35.2 minutes. A drain to zero, sampled at each merge, took 25 and 35 minutes.

## Decision

The owner posted three rulings on #1275 on 2026-09-28, in the comment titled "Design, with the owner's rulings (2026-09-28)". They are quoted here in full. One thing changes: the target of the HW-PD-0007 link. The comment wrote it relative to the issue page, and this copy writes it relative to this file.

> 1. **Stop and drain.** A parent session drains to zero in flight before it exits. An overlapping handover, where the next session starts while the old one drains, is not built now.
> 2. **Rework after a restart goes to a fresh `hw-build`.** Doctrine line 7 sends a FAIL back to the agent that built the branch. After a handover that agent no longer exists, so a fresh builder gets the verifier's finding and the `## Follow-up` of the old `build.md`. The token cost is about the same: verify takes 6.0 min at the median, which is past the subagent's five-minute cache lifetime ([HW-PD-0007](0007-a-background-wait-caps-below-the-cache-lifetime-and-re-issues-itself.md)), so a resumed builder already writes its whole context again. Line 7 is amended to say so.
> 3. **The handover recovers the per-issue manager of #1276 as well as a bare stage.** The handover file has one shape whether the parent or a manager runs the stages.

The manager of #1276 is `hw-iterate`. Ruling 2 narrows [HW-PD-0022](0022-the-verify-and-rework-loop-for-one-issue-runs-below-the-parent.md), which resumes a builder by its id, to one parent session. After a restart, a fresh `hw-iterate` reads the handover file and gives the rework to a fresh builder.

`tools/run/supervise.sh` runs one parent session at a time, as `claude -p "/next-run --resume <run-id>"`. It reads the context of each call from the stream. It creates `<run>/drain` when a call passes 130k tokens, or when the session has made 4 merges. The first session of a run stays interactive, because it puts the rulings of the product owner to the owner.

`tools/run/run-dir.sh` carries the state that a restart keeps:

- `stage` writes `handover/<issue>`: the stage that the issue reached, and the paths to resume it. The actor that ends a stage writes it.
- `next` gives the next issue to dispatch. It skips an issue that has a claim, a handover or a log line. It also skips an issue that the run ruled gated, deferred or refused, and a `ruling` line of the queue that no `OWNER` line answers.
- `resume` gives one line for each open handover, with the action that the issue needs next.
- `rule` writes a ruling on one issue into `decisions.md`, in the form that `next` reads.
- While `drain` exists, `claim`, `stage`, `next` and `resume` each print `DRAIN`. The parent and `hw-iterate` see the signal on calls that they already make.

In drain, nothing starts a new stage. A stage that ends writes its checkpoint and stops. `hw-iterate` writes its checkpoint and returns `HANDOVER` at its next stage boundary. The parent rules a `PASS` and adds it to the integrator queue. An integrator that is already in flight finishes, and the parent dispatches no new integrator.

An adjudicate that reports during a drain is not claimed, because `claim` writes nothing in drain. The parent writes `stage adjudicated` with the note and the footprint. The next session claims that footprint from `resume` and dispatches `hw-iterate`. So no issue is adjudicated two times.

## Consequences

Tokens alone favor frequent restarts. A restart costs about 50k tokens of one-hour cache write, and about 20 calls at a smaller context pay it back. Throughput limits the rate. A drain empties the slots for about 25 minutes. At 2.5 merges for each hour and a restart every 4 merges, stop and drain loses an estimated 13 to 15% of throughput. The comment of 2026-09-28 states the owner's position on it: "We accept that loss for the simpler design."

A session that starts near 60k and drains from about 130k reaches a peak near 142k and a mean near 100k. That mean is under the bar. K is a ceiling behind the threshold. It is 4 at about 30 parent calls for each issue, and about 6 is correct after #1274 and #1276.

The bar is not measured yet. `run-census.sh --context` reads the mean over all parent sessions of a run. A run after this decision must meet the bar before #1275 closes.

The byte ceilings of [HW-PD-0003](0003-a-dispatch-pays-when-it-retires-more-parent-turns-than-it-costs.md) do not change. The doctrine gains the drain exception and the rework rule after a restart. `next-run.md` gains the resume form in a short section. The detail of the resume form stays in the header of `supervise.sh` and in `hw-run-policy`. `next-run.md` has the parent load that skill at the start of every session, the first included, so its full `stage` form and its intake rule bind from the first session. The ceiling of `next-run.md` stays at 8192 bytes: two clauses that `hw-run-policy` also states made the room.

Three points stay open. `claude -p` may write the calls of subagents into its stream, and then the drain threshold would count them. The headless permissions of a parent session are not tested. A signal to the loop ends the session that it started, but no test showed that its subagents end with it.
