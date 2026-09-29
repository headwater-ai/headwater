---
id: HW-RUN-status-probe-pilot-of-2026-09-29-campaign-tier-present-arm-sufficiency
status: current
status_since: 2026-09-29
summary: "The present arm of the campaign tier over the status probe alone, in the #1294 pilot: 10 sessions, 98 cents, the intent hook live in 10, and 10 of 10 answered current HW-DR-0052."
last_verified: 2026-09-29
tier: campaign
arm: present
title: "Status probe pilot of 2026-09-29, campaign tier, present arm, sufficiency"
---

# Status probe pilot of 2026-09-29, campaign tier, present arm, sufficiency

## Run identity

```yaml
model: claude-sonnet-5
served_version: claude-sonnet-5
tree: sha256:7c8d9c1568c93bd1747f64dbda46742cf0c82d9e27b795aa2302de8bba2433fd
lock: sha256:badb09836f1099bd2d72472dc6370ac7fa0d14e93001264d38a69b5e59210e44
selection: sha256:002ab25221c821cea88b42dbbdb3e831304d41c12788a9557d7a1da2a01efa17
read_set: sha256:c97c613e433de3010d256895bc70c07bca1047d280d23f2dd841dc7712e0f2b7
seed: 0
harness: 0.4.1
tier: campaign
arm: present
at: 2026-09-29
cost_cents: 98
```

**This is a pilot of one probe and not the campaign.** `tools/probe/campaign.sh` recorded it on 2026-09-29 for clause 4 of [#1294](https://github.com/headwater-ai/headwater/issues/1294). It ran the probe `HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request` alone, at 10 repetitions per arm, after the changes to the authoring skill, to the summary of HW-DR-0052 and to the probe itself had merged. The owner ruled on #1294 on 2026-09-28 that this pilot runs after those changes, at no more than 10 repetitions per arm. No comparison pools it with the campaign of 2026-09-28, because the probe and its read set moved between the two.

**One batch, one tree.** Every session ran in a fresh copy of one `git archive` of commit `d3b1b16e`, with the engine that commit builds, in one order that seed 980 shuffled, four at a time, on `claude-sonnet-5` with a cap of 60 turns. Each workspace lost the instrument, every record under `docs/` that names the probe, and its answer key. Each absent workspace also lost the ablation of the campaign tier. The `seed` in the identity above is the plan's seed, and the shuffle seed is not a member of the identity.

## Events

```yaml
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L1-campaign-present-p1-r1"
  calls: []
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L1-campaign-present-p1-r10"
  calls: []
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L1-campaign-present-p1-r2"
  calls: []
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L1-campaign-present-p1-r3"
  calls: []
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L1-campaign-present-p1-r4"
  calls: []
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L1-campaign-present-p1-r5"
  calls: []
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L1-campaign-present-p1-r6"
  calls: []
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L1-campaign-present-p1-r7"
  calls: []
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L1-campaign-present-p1-r8"
  calls: []
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L1-campaign-present-p1-r9"
  calls: []
  produced: []
  answer: "current HW-DR-0052"
```
