---
id: HW-RUN-campaign-of-2026-10-03-campaign-tier-no-skills-arm-sufficiency-leak-kept-probes
status: current
status_since: 2026-10-08
summary: "The no-skills arm of the campaign tier over the sufficiency selection (leak-kept probes), in batch A of the paid layer campaign: 60 sessions, 482 cents, the intent hook live in 60, none stopped at the turn cap, none at the session budget."
last_verified: 2026-10-08
tier: campaign
arm: no-skills
title: "Campaign of 2026-10-03, campaign tier, no-skills arm, sufficiency, leak-kept probes"
relations:
  records:
    - HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted
    - HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request
provenance:
  warrant: asserted
---

# Campaign of 2026-10-03, campaign tier, no-skills arm, sufficiency, leak-kept probes

## Run identity

```yaml
model: claude-sonnet-5
served_version: claude-sonnet-5
tree: sha256:81d5f62acb289022175d192df0a61faf5f71eb4b79e8dd1f4efae4532ebdd916
lock: sha256:b768fe795c8d44a882e8604a5012c2eb5f2b4d584e7445367dfb24bb21c3585d
selection: sha256:18ec419cfebb740a16b74470a04e90ca181b3c78bef48e6ec5969aff76f1f37d
read_set: sha256:f565abcbde3c4a2d20c3e27c799dda32c5d6fe6fa691eedf3012d5c5df417004
seed: 0
harness: 0.5.0
tier: campaign
arm: no-skills
at: 2026-10-03
cost_cents: 482
```

**This line is one of the paid layer campaign that [#1659](https://github.com/headwater-ai/headwater/issues/1659) asked for.** `tools/probe/campaign.sh` recorded it as line 11 of the plan of batch A, a plan that draws its lines from `tools/probe/layer-campaign.spec`. It is the no-skills arm of the campaign tier over the sufficiency selection, on the leak-kept probes, at 30 repetitions per probe, on `claude-sonnet-5` under Claude Code 2.1.288 with a cap of 80 turns for each session. The two probes of this line keep their leak string in the present arm, so they run on a line of their own and are never pooled into the rate of the other sufficiency probes.

**One tree, with one move of the pin.** Every session ran in a fresh copy of a `git archive` of the pinned tree, and the engine of that tree. The pin was `682c1e2a` at the start of the batch. It moved to `debe0c70` at session 650 of the batch, at 2026-10-04T09:00:26Z, and the sessions after it ran on `debe0c70`. The move changed five files under `tools/probe/` and no document, and the assembly accepted it. The `tree` digest in the identity above is the digest the assembly recorded, and a reader should not take it to state that all 2150 sessions of the batch read one pin.

**What the recorder counted.** The line holds 60 sessions and cost 482 cents. The intent hook was live in 60 of them. None stopped at the turn cap, and none at the session budget. The recorder reports 0 paths outside the workspace named in a call of this line, 0 sessions uncounted, and 0 web-tool calls.

**The contract of this batch.** The sessions ran under the recorder contract of spec 15: a `bwrap` sandbox with no network of its own, an empty configuration directory, `--setting-sources project,local`, the permission mode `dontAsk`, and `WebSearch` and `WebFetch` denied. The batches of 2026-09-28 and 2026-09-30 ran under `bypassPermissions` and the configuration of the host, so a rate from this batch does not compare with a rate from those batches.

**Where the figures are.** The rates, the intervals and the comparisons of this line are in the generated result that reads this transcript, and this document states none of them.

## Events

```yaml
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L11-campaign-no-skills-p1-r1"
  calls: []
  produced: []
  answer: "merge"
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L11-campaign-no-skills-p1-r10"
  calls: []
  produced: []
  answer: "merge"
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L11-campaign-no-skills-p1-r11"
  calls: []
  produced: []
  answer: "merge"
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L11-campaign-no-skills-p1-r12"
  calls: []
  produced: []
  answer: "merge"
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L11-campaign-no-skills-p1-r13"
  calls: []
  produced: []
  answer: "merge"
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L11-campaign-no-skills-p1-r14"
  calls: []
  produced: []
  answer: "merge"
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L11-campaign-no-skills-p1-r15"
  calls: []
  produced: []
  answer: "merge"
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L11-campaign-no-skills-p1-r16"
  calls: []
  produced: []
  answer: "merge"
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L11-campaign-no-skills-p1-r17"
  calls: []
  produced: []
  answer: "merge"
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L11-campaign-no-skills-p1-r18"
  calls: []
  produced: []
  answer: "merge"
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L11-campaign-no-skills-p1-r19"
  calls: []
  produced: []
  answer: "merge"
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L11-campaign-no-skills-p1-r2"
  calls: []
  produced: []
  answer: "merge"
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L11-campaign-no-skills-p1-r20"
  calls: []
  produced: []
  answer: "merge"
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L11-campaign-no-skills-p1-r21"
  calls: []
  produced: []
  answer: "merge"
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L11-campaign-no-skills-p1-r22"
  calls: []
  produced: []
  answer: "merge"
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L11-campaign-no-skills-p1-r23"
  calls: []
  produced: []
  answer: "merge"
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L11-campaign-no-skills-p1-r24"
  calls: []
  produced: []
  answer: "merge"
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L11-campaign-no-skills-p1-r25"
  calls: []
  produced: []
  answer: "merge"
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L11-campaign-no-skills-p1-r26"
  calls: []
  produced: []
  answer: "merge"
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L11-campaign-no-skills-p1-r27"
  calls: []
  produced: []
  answer: "merge"
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L11-campaign-no-skills-p1-r28"
  calls: []
  produced: []
  answer: "merge"
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L11-campaign-no-skills-p1-r29"
  calls: []
  produced: []
  answer: "merge"
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L11-campaign-no-skills-p1-r3"
  calls: []
  produced: []
  answer: "merge"
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L11-campaign-no-skills-p1-r30"
  calls: []
  produced: []
  answer: "merge"
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L11-campaign-no-skills-p1-r4"
  calls: []
  produced: []
  answer: "merge"
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L11-campaign-no-skills-p1-r5"
  calls: []
  produced: []
  answer: "merge"
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L11-campaign-no-skills-p1-r6"
  calls: []
  produced: []
  answer: "merge"
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L11-campaign-no-skills-p1-r7"
  calls: []
  produced: []
  answer: "merge"
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L11-campaign-no-skills-p1-r8"
  calls: []
  produced: []
  answer: "merge"
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L11-campaign-no-skills-p1-r9"
  calls: []
  produced: []
  answer: "merge"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L11-campaign-no-skills-p2-r1"
  calls: []
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L11-campaign-no-skills-p2-r10"
  calls: []
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L11-campaign-no-skills-p2-r11"
  calls: []
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L11-campaign-no-skills-p2-r12"
  calls: []
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L11-campaign-no-skills-p2-r13"
  calls: []
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L11-campaign-no-skills-p2-r14"
  calls: []
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L11-campaign-no-skills-p2-r15"
  calls: []
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L11-campaign-no-skills-p2-r16"
  calls: []
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L11-campaign-no-skills-p2-r17"
  calls: []
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L11-campaign-no-skills-p2-r18"
  calls: []
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L11-campaign-no-skills-p2-r19"
  calls: []
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L11-campaign-no-skills-p2-r2"
  calls: []
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L11-campaign-no-skills-p2-r20"
  calls:
    - tool: "Read"
      argument: "/var/tmp/hw-1659/a/ws/L11-campaign-no-skills-p2-r20/docs/decisions/0052-a-document-is-proposed-at-the-state-it-will-hold-and-the-merge-activates-it.md"
      result: "sha256:ee87b97f4c18e94cf39371da0fa68d28f7bd1b7275813ef6955340c81ac42b6d"
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L11-campaign-no-skills-p2-r21"
  calls: []
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L11-campaign-no-skills-p2-r22"
  calls: []
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L11-campaign-no-skills-p2-r23"
  calls: []
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L11-campaign-no-skills-p2-r24"
  calls: []
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L11-campaign-no-skills-p2-r25"
  calls: []
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L11-campaign-no-skills-p2-r26"
  calls: []
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L11-campaign-no-skills-p2-r27"
  calls:
    - tool: "Read"
      argument: "/var/tmp/hw-1659/a/ws/L11-campaign-no-skills-p2-r27/docs/decisions/0052-a-document-is-proposed-at-the-state-it-will-hold-and-the-merge-activates-it.md"
      result: "sha256:ee87b97f4c18e94cf39371da0fa68d28f7bd1b7275813ef6955340c81ac42b6d"
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L11-campaign-no-skills-p2-r28"
  calls: []
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L11-campaign-no-skills-p2-r29"
  calls: []
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L11-campaign-no-skills-p2-r3"
  calls: []
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L11-campaign-no-skills-p2-r30"
  calls: []
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L11-campaign-no-skills-p2-r4"
  calls: []
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L11-campaign-no-skills-p2-r5"
  calls: []
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L11-campaign-no-skills-p2-r6"
  calls: []
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L11-campaign-no-skills-p2-r7"
  calls: []
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L11-campaign-no-skills-p2-r8"
  calls: []
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L11-campaign-no-skills-p2-r9"
  calls: []
  produced: []
  answer: "current HW-DR-0052"
```
