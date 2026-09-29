---
id: HW-RUN-status-probe-pilot-of-2026-09-29-campaign-tier-absent-arm-sufficiency
status: current
status_since: 2026-09-29
summary: "The absent arm of the campaign tier over the status probe alone, in the #1294 pilot: 10 sessions, 179 cents, the intent hook live in 0, and 8 of 10 answered current HW-DR-0052."
last_verified: 2026-09-29
tier: campaign
arm: absent
title: "Status probe pilot of 2026-09-29, campaign tier, absent arm, sufficiency"
---

# Status probe pilot of 2026-09-29, campaign tier, absent arm, sufficiency

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
arm: absent
at: 2026-09-29
cost_cents: 179
```

**This is a pilot of one probe and not the campaign.** `tools/probe/campaign.sh` recorded it on 2026-09-29 for clause 4 of [#1294](https://github.com/headwater-ai/headwater/issues/1294). It ran the probe `HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request` alone, at 10 repetitions per arm, after the changes to the authoring skill, to the summary of HW-DR-0052 and to the probe itself had merged. The owner ruled on #1294 on 2026-09-28 that this pilot runs after those changes, at no more than 10 repetitions per arm. No comparison pools it with the campaign of 2026-09-28, because the probe and its read set moved between the two.

**One batch, one tree.** Every session ran in a fresh copy of one `git archive` of commit `d3b1b16e`, with the engine that commit builds, in one order that seed 980 shuffled, four at a time, on `claude-sonnet-5` with a cap of 60 turns. Each workspace lost the instrument, every record under `docs/` that names the probe, and its answer key. Each absent workspace also lost the ablation of the campaign tier. The `seed` in the identity above is the plan's seed, and the shuffle seed is not a member of the identity.

## Events

```yaml
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L2-campaign-absent-p1-r1"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"find /home/james/probe-batches/1294-pilot-2026-09-29/ws/L2-campaign-absent-p1-r1 -maxdepth 3 | head -100\",\"description\":\"List directory contents up to depth 3\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"ls -la \\\"/home/james/probe-batches/1294-pilot-2026-09-29/ws/L2-campaign-absent-p1-r1/docs/decisions\\\" | head -100\",\"description\":\"List decision records directory\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/1294-pilot-2026-09-29/ws/L2-campaign-absent-p1-r1/docs/decisions/0052-a-document-is-proposed-at-the-state-it-will-hold-and-the-merge-activates-it.md"
      result: "sha256:ee87b97f4c18e94cf39371da0fa68d28f7bd1b7275813ef6955340c81ac42b6d"
  produced: []
  answer: null
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L2-campaign-absent-p1-r10"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"find /home/james/probe-batches/1294-pilot-2026-09-29/ws/L2-campaign-absent-p1-r10 -maxdepth 3\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"find . -iname \\\"*decision*\\\" -not -path \\\"*/node_modules/*\\\" | head -50\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/1294-pilot-2026-09-29/ws/L2-campaign-absent-p1-r10/docs/taxonomies/decision-record/templates/decision.md"
      result: "sha256:92472c6e54d089e648bea5c738dd08bfdd0d27aff369aff17703b7f8b33894e8"
    - tool: "Bash"
      argument: "{\"command\":\"cd /home/james/probe-batches/1294-pilot-2026-09-29/ws/L2-campaign-absent-p1-r10/docs/decisions && grep -liE \\\"status.*draft|draft.*current|merge|pull request\\\" *.md | head -50\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/1294-pilot-2026-09-29/ws/L2-campaign-absent-p1-r10/docs/decisions/0052-a-document-is-proposed-at-the-state-it-will-hold-and-the-merge-activates-it.md"
      result: "sha256:ee87b97f4c18e94cf39371da0fa68d28f7bd1b7275813ef6955340c81ac42b6d"
    - tool: "Read"
      argument: "/home/james/probe-batches/1294-pilot-2026-09-29/ws/L2-campaign-absent-p1-r10/docs/decisions/0034-q34-whether-acceptance-means-merged-to-main-and-what-an-agent-may-write-before-that.md"
      result: "sha256:6de683b661a06755c335d85abbf92235bf188d5832a9ef657f7aa8b66fedf1d4"
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L2-campaign-absent-p1-r2"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"find /home/james/probe-batches/1294-pilot-2026-09-29/ws/L2-campaign-absent-p1-r2 -maxdepth 3 | head -100\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"ls /home/james/probe-batches/1294-pilot-2026-09-29/ws/L2-campaign-absent-p1-r2/docs/decisions | head -100\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/1294-pilot-2026-09-29/ws/L2-campaign-absent-p1-r2/docs/decisions/0052-a-document-is-proposed-at-the-state-it-will-hold-and-the-merge-activates-it.md"
      result: "sha256:ee87b97f4c18e94cf39371da0fa68d28f7bd1b7275813ef6955340c81ac42b6d"
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L2-campaign-absent-p1-r3"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"find /home/james/probe-batches/1294-pilot-2026-09-29/ws/L2-campaign-absent-p1-r3 -type f | head -200\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"find /home/james/probe-batches/1294-pilot-2026-09-29/ws/L2-campaign-absent-p1-r3/docs -maxdepth 2 -type d\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"ls /home/james/probe-batches/1294-pilot-2026-09-29/ws/L2-campaign-absent-p1-r3/docs/taxonomies/decision-record/\\necho ---\\nls /home/james/probe-batches/1294-pilot-2026-09-29/ws/L2-campaign-absent-p1-r3/docs/decisions/ | head -100\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/1294-pilot-2026-09-29/ws/L2-campaign-absent-p1-r3/docs/decisions/0052-a-document-is-proposed-at-the-state-it-will-hold-and-the-merge-activates-it.md"
      result: "sha256:ee87b97f4c18e94cf39371da0fa68d28f7bd1b7275813ef6955340c81ac42b6d"
  produced: []
  answer: null
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L2-campaign-absent-p1-r4"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"find . -iname \\\"*.md\\\" | xargs grep -liE \\\"^status:|status:\\\" 2>/dev/null | head -50\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rliE \\\"status:.*(draft|current)\\\" docs/decisions/ | head -30; echo \\\"---\\\"; grep -rliE \\\"front.?matter\\\" docs/decisions/ | head -30\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rliE \\\"status\\\" docs/decisions/*.md | xargs grep -liE \\\"pull request|PR |merge\\\" | head -30\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/1294-pilot-2026-09-29/ws/L2-campaign-absent-p1-r4/docs/decisions/0052-a-document-is-proposed-at-the-state-it-will-hold-and-the-merge-activates-it.md"
      result: "sha256:ee87b97f4c18e94cf39371da0fa68d28f7bd1b7275813ef6955340c81ac42b6d"
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L2-campaign-absent-p1-r5"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"find . -maxdepth 4 -iname \\\"*.md\\\" | head -100 && echo --- && ls -la\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"find ./docs -maxdepth 2 -type d; echo ---; ls docs/decisions 2>/dev/null | head -50\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"ls docs/taxonomies/decision-record; echo ---; grep -rn \\\"status\\\" docs/taxonomies/decision-record | head -50\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cat \\\"docs/decisions/0034-q34-whether-acceptance-means-merged-to-main-and-what-an-agent-may-write-before-that.md\\\"\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rln \\\"status: draft\\\\|status.*draft\\\\|merge.*status\\\\|status.*merge\\\" docs/decisions | xargs grep -l \\\"status\\\" 2>/dev/null | head -30\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cat \\\"docs/decisions/0052-a-document-is-proposed-at-the-state-it-will-hold-and-the-merge-activates-it.md\\\"\"}"
      result: ""
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L2-campaign-absent-p1-r6"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"find . -maxdepth 4 -iname \\\"*decision*\\\" -o -iname \\\"*adr*\\\" | head -50\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"ls docs/decisions | head -50; echo ---; ls docs/process/decisions; echo ---; ls docs/taxonomies/decision-record\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/1294-pilot-2026-09-29/ws/L2-campaign-absent-p1-r6/docs/spec/09-decisions.md"
      result: "sha256:51565d3e4d876685ebd1b7aa0eb072e1066612513b33db8f00f9ca5555975c4a"
    - tool: "Read"
      argument: "/home/james/probe-batches/1294-pilot-2026-09-29/ws/L2-campaign-absent-p1-r6/docs/taxonomies/decision-record/doctrine.md"
      result: "sha256:caa0064ce28e35f545766d4d97dd1f131852345d0b2beaa0800667f8a61823b1"
    - tool: "Bash"
      argument: "{\"command\":\"cat \\\"docs/decisions/0034-q34-whether-acceptance-means-merged-to-main-and-what-an-agent-may-write-before-that.md\\\"\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rl \\\"status\\\" docs/decisions | xargs grep -l \\\"merge\\\\|pull request\\\\|PR \\\" | sort\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cat \\\"docs/decisions/0052-a-document-is-proposed-at-the-state-it-will-hold-and-the-merge-activates-it.md\\\"\"}"
      result: ""
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L2-campaign-absent-p1-r7"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"find . -maxdepth 3 -iname \\\"*.md\\\" | head -100\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cat docs/decisions/README.md\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rl \\\"status\\\" docs/decisions/*.md | xargs grep -l \\\"pull request\\\\|merge\\\\|draft\\\" | head -30\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cat docs/decisions/0052-a-document-is-proposed-at-the-state-it-will-hold-and-the-merge-activates-it.md\"}"
      result: ""
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L2-campaign-absent-p1-r8"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"find /home/james/probe-batches/1294-pilot-2026-09-29/ws/L2-campaign-absent-p1-r8 -maxdepth 3 2>&1 | head -100\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"ls /home/james/probe-batches/1294-pilot-2026-09-29/ws/L2-campaign-absent-p1-r8/docs/decisions | head -100\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cat \\\"/home/james/probe-batches/1294-pilot-2026-09-29/ws/L2-campaign-absent-p1-r8/docs/decisions/0052-a-document-is-proposed-at-the-state-it-will-hold-and-the-merge-activates-it.md\\\"\"}"
      result: ""
  produced: []
  answer: "current HW-DR-0052"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L2-campaign-absent-p1-r9"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"find /home/james/probe-batches/1294-pilot-2026-09-29/ws/L2-campaign-absent-p1-r9 -maxdepth 4 | head -200\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"ls /home/james/probe-batches/1294-pilot-2026-09-29/ws/L2-campaign-absent-p1-r9/docs/decisions | head -100\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/1294-pilot-2026-09-29/ws/L2-campaign-absent-p1-r9/docs/decisions/0052-a-document-is-proposed-at-the-state-it-will-hold-and-the-merge-activates-it.md"
      result: "sha256:ee87b97f4c18e94cf39371da0fa68d28f7bd1b7275813ef6955340c81ac42b6d"
  produced: []
  answer: "current HW-DR-0052"
```
