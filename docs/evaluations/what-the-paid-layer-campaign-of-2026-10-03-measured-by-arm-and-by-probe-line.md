---
id: HW-EVAL-what-the-paid-layer-campaign-of-2026-10-03-measured-by-arm-and-by-probe-line
status: current
status_since: 2026-10-09
summary: "Removing docs/ cut the sufficiency rate by about 46 points, no layer component shows a gain on more than one selection, and the owner accepted the outside-workspace count as measured."
last_verified: 2026-10-09
title: "What the paid layer campaign of 2026-10-03 measured, by arm and by probe line"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-sonnet-5.5
  activity: measure+draft
  evidence_basis: reconstructed
relations:
  traces_to:
    - HW-RUN-campaign-of-2026-10-03-campaign-tier-absent-arm-navigability
    - HW-RUN-campaign-of-2026-10-03-campaign-tier-absent-arm-sufficiency-four-probes
    - HW-RUN-campaign-of-2026-10-03-campaign-tier-absent-arm-sufficiency-leak-kept-probes
    - HW-RUN-campaign-of-2026-10-03-campaign-tier-mcp-arm-navigability
    - HW-RUN-campaign-of-2026-10-03-campaign-tier-mcp-arm-sufficiency-four-probes
    - HW-RUN-campaign-of-2026-10-03-campaign-tier-mcp-arm-sufficiency-leak-kept-probes
    - HW-RUN-campaign-of-2026-10-03-campaign-tier-no-claude-md-arm-navigability
    - HW-RUN-campaign-of-2026-10-03-campaign-tier-no-claude-md-arm-sufficiency-four-probes
    - HW-RUN-campaign-of-2026-10-03-campaign-tier-no-claude-md-arm-sufficiency-leak-kept-probes
    - HW-RUN-campaign-of-2026-10-03-campaign-tier-no-hook-arm-navigability
    - HW-RUN-campaign-of-2026-10-03-campaign-tier-no-hook-arm-sufficiency-four-probes
    - HW-RUN-campaign-of-2026-10-03-campaign-tier-no-hook-arm-sufficiency-leak-kept-probes
    - HW-RUN-campaign-of-2026-10-03-campaign-tier-no-skills-arm-navigability
    - HW-RUN-campaign-of-2026-10-03-campaign-tier-no-skills-arm-sufficiency-four-probes
    - HW-RUN-campaign-of-2026-10-03-campaign-tier-no-skills-arm-sufficiency-leak-kept-probes
    - HW-RUN-campaign-of-2026-10-03-campaign-tier-present-arm-navigability
    - HW-RUN-campaign-of-2026-10-03-campaign-tier-present-arm-sufficiency-four-probes
    - HW-RUN-campaign-of-2026-10-03-campaign-tier-present-arm-sufficiency-leak-kept-probes
    - HW-RUN-campaign-of-2026-10-03-documentation-tier-absent-arm-sufficiency-four-probes
    - HW-RUN-campaign-of-2026-10-03-documentation-tier-absent-arm-sufficiency-leak-kept-probes
    - HW-RUN-campaign-of-2026-10-05-campaign-tier-absent-arm-discovery
    - HW-RUN-campaign-of-2026-10-05-campaign-tier-mcp-arm-discovery
    - HW-RUN-campaign-of-2026-10-05-campaign-tier-no-claude-md-arm-discovery
    - HW-RUN-campaign-of-2026-10-05-campaign-tier-no-hook-arm-discovery
    - HW-RUN-campaign-of-2026-10-05-campaign-tier-no-skills-arm-discovery
    - HW-RUN-campaign-of-2026-10-05-campaign-tier-present-arm-discovery
---

# What the paid layer campaign of 2026-10-03 measured, by arm and by probe line

This page is for a reader who wants to know which parts of the Headwater layer change what an agent does, and what it cost to find out. It reads the paid campaign that [#1659](https://github.com/headwater-ai/headwater/issues/1659) ran, in two batches, and reports each arm against the `present` arm for each probe line. It evaluated commit `42851acabde36707eca485710a568296fce9ecb1` (`42851aca`) on 2026-10-09, and it was revised the same day after the 10 jobs that the documentation tier ceiling had refused were drawn and added to the transcripts of lines 7 and 14. The 26 transcripts and 26 results of the campaign are the source. Every figure below comes from those committed files, and no figure comes from an uncommitted file under `/var/tmp/hw-1659`, except the check of the raw streams that clause 2 states and marks as not committed.

## The finding

The documents under `docs/` have a large effect, and the layer around them has a small one that this campaign cannot place with confidence. Removing the documents lowered the sufficiency rate by 44.2 points on the four-probe line and by 50.0 points on the leak-kept line. No component of the layer shows a gain on more than one selection. Three defects of the instrument stop the campaign from supporting a stronger sentence, and the bullets below name them.

- The `governs line` probe has an inverted key. The `present` arm satisfied it in 0 of 30 sessions and the `absent` arm in 26 of 30.
- The `mcp` arm made no call to the MCP server in 224 of 330 batch A sessions and in 140 of 354 batch B sessions.
- Eight of the eleven batch A probes sit at 30 of 30 in the `present` arm, so they cannot show a gain.

Done-when clause 2 of #1659 was not met as written. The recorder counted 316 paths outside the workspace in batch A and 177 in batch B, and the owner re-ruled the clause as measured on 2026-10-09. Clause 3 is complete, and clauses 1 to 6 are met or re-ruled by the owner. So the pull request that carries this page uses `Closes #1659`.

## What ran

The campaign ran 4,280 recorded sessions on `claude-sonnet-5` under Claude Code 2.1.288, with a cap of 80 turns for each session. The plan of `tools/probe/layer-campaign.spec` priced 4,284 sessions at about $2,142, and the owner approved that figure on 2026-10-03.

- **Batch A** recorded 2,156 sessions of 2,160 planned, over 20 lines: the sufficiency, leak-kept and navigability selections in the six `campaign` arms, and the sufficiency and leak-kept selections in the `documentation` absent arm. It ran from 2026-10-03.
- **Batch B** recorded 2,124 sessions of 2,124 planned, over 6 lines: the discovery selection in the six `campaign` arms, at 118 repetitions for each probe. Its transcripts carry the date 2026-10-05.

Each session ran in a fresh copy of an archive of the pinned tree, inside a `bwrap` sandbox with no network of its own, an empty configuration directory, the permission mode `dontAsk`, and `WebSearch` and `WebFetch` denied. The batches of 2026-09-28 and 2026-09-30 ran under other settings, so no rate here compares with a rate from those batches. The pin moved once in batch A, from `682c1e2a` to `debe0c70` at session 650. The move changed five files under `tools/probe/` and no document.

The six `campaign` arms differ from the `present` arm as follows. The `absent` arm removes the whole layer and keeps `docs/`. The `no-hook` arm removes the intent hook, and the recorder reports the hook live in 0 of its sessions. The `no-skills` arm removes the skills. The `no-claude-md` arm removes `CLAUDE.md`. The `mcp` arm adds the MCP server. The `documentation` tier's `absent` arm removes the layer and `docs/` too, so it measures the documents.

## The Done-when clauses of #1659

1. **The owner's ruling is quoted.** In run 20261002-1233 the owner chose "Approve 2,142": "the campaign plan of 4,284 sessions at about 2,142 dollars stands." In run 20261003-1026, on 2026-10-03, he ruled: "I am giving the go ahead for 1659 so do it when ready." Met.
2. **The recorder reports 0 outside-workspace calls and 0 web-tool calls.** The web half is met: every one of the 26 lines reports 0 web-tool calls. The outside-workspace half was not met as written, and the owner re-ruled it on 2026-10-09: "I accept the recommendation." The ruling takes the count as measured and runs nothing again. As measured, the count is distinct absolute paths named in a call, for each session, so it counts tries and not reads, and it is 316 in batch A and 177 in batch B. The committed transcripts blank every tool result, so success cannot be read from them. A check of the uncommitted raw streams of the 4,284 sessions on disk (the 4,280 recorded sessions and the 4 unrecorded ones) found 45 calls that name a host path under `/opt/headwater-egress`, `/opt/headwater-harness`, `/root` or `/home`. Of those 45 calls, 3 were refused and 42 returned content. Most were `ls` or `find` of the harness directories or of an empty `/root` or `/home`. The exceptions are one `grep` of `/opt/headwater-harness/egress-proxy.py` in `L7-documentation-absent-p2-r11`, which returned matching lines of the recorder's proxy source, and four `Agent` and `SendMessage` calls in three sessions that name `/home` and `/root` only in their text. The four whole-filesystem searches (`L7-documentation-absent-p2-r18` and `-p2-r27`) found only the session's own configuration directory. No session named another session's workspace, the pinned tree or the host repository. The scan covered `/opt/headwater*`, `/root`, `/home`, `/mnt` and `/etc/headwater`, other sessions' directories and the pinned tree, and not every outside path. This evidence is not committed. It comes from the raw streams under `/var/tmp/hw-1659`, which a reader of the committed files cannot read. Met by the owner's re-ruling.
3. **Every transcript is committed with its result.** All 26 lines have a transcript under `docs/probe-runs/` and a result under `docs/probe-results/`, and every recorded session has a transcript. The 10 jobs that the documentation tier ceiling refused were drawn on 2026-10-09, after the owner ruled "go ahead with the 800 cents", and they are in the transcripts of lines 7 and 14. The 4 sessions that ended with status 5 (`L5-campaign-no-claude-md-p3-r12`, `-p3-r19` and `-p3-r28`, and `L18-campaign-no-skills-p5-r11`) are accepted as unrecorded, and the owner ruled that they not be drawn again, to avoid selection bias. The one redraw of `L7-documentation-absent-p3-r4` is disclosed in the section on selection. Nothing else is outstanding. Met.
4. **An evaluation reports each arm against `present`, per probe line.** This page. It names the commit and the date.
5. **The restatements of HW-OBL-0012, 0013, 0014 and 0016 and of the spec 13 Unmeasured claims are made.** See the section on the obligations. Each record stays open, with a reason. Met.
6. **The corpus states what the campaign found about each component arm.** See the section on the component arms. Met by this page, by the four records, and by the paragraph in spec 13.

## The plan against the record

Batch A lost 4 of 2,160 sessions. Batch B lost none.

| line | planned | recorded | missing | why |
|---|---|---|---|---|
| `no-claude-md`, four probes (line 5) | 120 | 117 | 3 | 3 sessions ended with status 5 |
| `no-skills`, navigability (line 18) | 150 | 149 | 1 | 1 session ended with status 5 |

**The documentation tier reached its 13,000-cent ceiling in the first run, and the jobs the ceiling refused were drawn afterward.** The ceiling refused 10 jobs before they started: 6 of line 7 (`p1-r24`, `p2-r29`, `p3-r5`, `p3-r17`, `p4-r1` and `p4-r5`) and 4 of line 14 (`p1-r10`, `p1-r12`, `p1-r25` and `p1-r29`). At that point the two lines recorded 9,089 and 3,831 cents, 12,920 cents together, and the tier had spent 12,878 cents when the owner lifted the ceiling by 300 cents for one job. On 2026-10-09 the owner ruled "go ahead with the 800 cents", and the 10 jobs were drawn with the ceiling lifted by 1,020 cents for that run, so that no job started once 800 cents were spent. They ran on the same pin `debe0c70` and the same Claude Code 2.1.288, each ended with status 0, and together they cost 513 cents: 428 on line 7 and 85 on line 14. The two lines now record 9,517 and 3,916 cents, 13,433 cents together, and each holds its planned count, 120 and 60. The transcripts of both lines state the draws. No ceiling refusal is outstanding for the plan. The first-drawn session, `L7-documentation-absent-p2-r29`, cost 232 cents against a tier average near 76; this page reports the figure and does not explain it.

**Four sessions ended with status 5 and are unrecorded.** They are `L5-campaign-no-claude-md-p3-r12`, `-p3-r19`, `-p3-r28` and `L18-campaign-no-skills-p5-r11`. The transcripts of those two lines report 3 and 1 sessions uncounted, and the results lack exactly those repetitions. The status code itself is in the decisions file and in the commit message of #1697, and not in a committed transcript. The three `no-claude-md` sessions all belong to the probe that records an unmeasured claim, so that cell has 27 sessions and not 30. The owner accepted all four as unrecorded on 2026-10-09 and ruled that they not be drawn again.

**Attrition by arm** over both batches, for the campaign arms planned at 684 sessions each:

| arm | recorded | missing | share of the arm |
|---|---|---|---|
| `present` | 684 | 0 | 0.0% |
| `absent` | 684 | 0 | 0.0% |
| `no-hook` | 684 | 0 | 0.0% |
| `no-skills` | 683 | 1 | 0.1% |
| `no-claude-md` | 681 | 3 | 0.4% |
| `mcp` | 684 | 0 | 0.0% |
| `documentation` absent | 180 | 0 | 0.0% |

The 4 missing sessions are in two `campaign` arms. The `documentation` arm lost none once the 10 refused jobs were drawn.

## The outside-workspace paths

**Clause 2 was not met as written, and the owner re-ruled it as measured.** The recorder counted 316 paths outside the workspace in batch A and 177 in batch B, 493 in all. Clause 2 above states the check of the raw streams. The number is a count of tries and not of reads. For each session, the recorder takes each absolute path in the input of a call that reads or writes a file, and each absolute path in the text of a `Bash` command, that the confinement does not bind. It counts each distinct path once for each session, and the batch figure sums the sessions. The sandbox mounts none of those paths from the host, so the transcript says that no file of the host at any of them was readable. The count does not say that a session read the repository outside its workspace.

| arm | batch A | batch B |
|---|---|---|
| `present` | 16 | 21 |
| `absent` | 24 | 46 |
| `no-hook` | 31 | 28 |
| `no-skills` | 24 | 28 |
| `no-claude-md` | 29 | 43 |
| `mcp` | 31 | 11 |
| `documentation` absent | 161 | not run |

The `documentation` absent arm accounts for 161 of the 316 in batch A, and 137 of those 161 are on line 7. That arm has no `docs/`, so a session looks for the documents elsewhere. Batch B has 0 sessions uncounted, and batch A has 4, the status-5 sessions. Both batches report 0 web-tool calls. The campaign did not meet the bar of 0 as written. The owner accepted the count as measured on 2026-10-09, so the clause is re-ruled and closed.

## The results, by probe line

Each cell reads `k/n (difference; interval)`. The difference is the arm minus the `present` arm in points, with a 95% Newcombe interval, computed from the committed verdicts. The whole-line figure for each of the 26 lines agrees with the rate that the result prints for its transcript, except that the discovery line here leaves out the pointer probe. The `mcp` navigability cell agrees with the result's own difference of -0.7 points, -11.5 to +10.2. A cell in which both arms are 30 of 30 reads +0.0 with an interval of -11.4 to +11.4, which means the probe cannot separate them.

### Sufficiency, four probes

| probe (line) | present | absent | no-hook | no-skills | no-claude-md | mcp | documentation absent |
|---|---|---|---|---|---|---|---|
| tombstone | 30/30 | 30/30 (+0.0; -11.4 to +11.4) | 30/30 (+0.0; -11.4 to +11.4) | 30/30 (+0.0; -11.4 to +11.4) | 30/30 (+0.0; -11.4 to +11.4) | 30/30 (+0.0; -11.4 to +11.4) | 30/30 (+0.0; -11.4 to +11.4) |
| citation | 30/30 | 30/30 (+0.0; -11.4 to +11.4) | 30/30 (+0.0; -11.4 to +11.4) | 30/30 (+0.0; -11.4 to +11.4) | 30/30 (+0.0; -11.4 to +11.4) | 30/30 (+0.0; -11.4 to +11.4) | 8/30 (-73.3; -85.8 to -52.2) |
| unmeasured claim | 30/30 | 29/30 (-3.3; -16.7 to +8.3) | 30/30 (+0.0; -11.4 to +11.4) | 30/30 (+0.0; -11.4 to +11.4) | 26/27 (-3.7; -18.3 to +8.0) | 29/30 (-3.3; -16.7 to +8.3) | 4/30 (-86.7; -94.7 to -66.8) |
| discharge | 24/30 | 7/30 (-56.7; -72.3 to -32.0) | 27/30 (+10.0; -8.8 to +28.5) | 20/30 (-13.3; -34.1 to +9.0) | 25/30 (+3.3; -16.6 to +23.0) | 25/30 (+3.3; -16.6 to +23.0) | 1/30 (-76.7; -87.5 to -54.8) |
| **whole line** | 114/120 | 96/120 (-15.0; -23.5 to -6.7) | 117/120 (+2.5; -2.8 to +8.2) | 110/120 (-3.3; -10.2 to +3.3) | 111/117 (-0.1; -6.3 to +6.0) | 114/120 (+0.0; -6.1 to +6.1) | 43/120 (-59.2; -67.6 to -48.7) |

### Sufficiency, the two leak-kept probes

These two probes keep their leak string in the `present` arm, and the plan never pools them with the four.

| probe (line) | present | absent | no-hook | no-skills | no-claude-md | mcp | documentation absent |
|---|---|---|---|---|---|---|---|
| accepted event | 30/30 | 23/30 (-23.3; -40.9 to -7.1) | 7/30 (-76.7; -88.2 to -55.7) | 30/30 (+0.0; -11.4 to +11.4) | 30/30 (+0.0; -11.4 to +11.4) | 30/30 (+0.0; -11.4 to +11.4) | 1/30 (-96.7; -99.4 to -79.2) |
| status in pull request | 30/30 | 30/30 (+0.0; -11.4 to +11.4) | 30/30 (+0.0; -11.4 to +11.4) | 30/30 (+0.0; -11.4 to +11.4) | 30/30 (+0.0; -11.4 to +11.4) | 30/30 (+0.0; -11.4 to +11.4) | 22/30 (-26.7; -44.4 to -9.8) |
| **whole line** | 60/60 | 53/60 (-11.7; -22.2 to -3.2) | 37/60 (-38.3; -51.0 to -25.6) | 60/60 (+0.0; -6.0 to +6.0) | 60/60 (+0.0; -6.0 to +6.0) | 60/60 (+0.0; -6.0 to +6.0) | 23/60 (-61.7; -72.9 to -47.7) |

### Navigability

| probe (line) | present | absent | no-hook | no-skills | no-claude-md | mcp |
|---|---|---|---|---|---|---|
| register | 30/30 | 30/30 (+0.0; -11.4 to +11.4) | 30/30 (+0.0; -11.4 to +11.4) | 30/30 (+0.0; -11.4 to +11.4) | 30/30 (+0.0; -11.4 to +11.4) | 30/30 (+0.0; -11.4 to +11.4) |
| governs line | 0/30 | 26/30 (+86.7; +66.8 to +94.7) | 4/30 (+13.3; -0.6 to +29.7) | 2/30 (+6.7; -5.7 to +21.3) | 12/30 (+40.0; +20.9 to +57.7) | 1/30 (+3.3; -8.3 to +16.7) |
| figure on the site | 30/30 | 29/30 (-3.3; -16.7 to +8.3) | 22/30 (-26.7; -44.4 to -9.8) | 30/30 (+0.0; -11.4 to +11.4) | 30/30 (+0.0; -11.4 to +11.4) | 30/30 (+0.0; -11.4 to +11.4) |
| adjudication | 30/30 | 30/30 (+0.0; -11.4 to +11.4) | 30/30 (+0.0; -11.4 to +11.4) | 30/30 (+0.0; -11.4 to +11.4) | 30/30 (+0.0; -11.4 to +11.4) | 30/30 (+0.0; -11.4 to +11.4) |
| ruling cited | 5/30 | 6/30 (+3.3; -16.6 to +23.0) | 2/30 (-10.0; -27.6 to +7.4) | 8/29 (+10.9; -10.3 to +31.3) | 13/30 (+26.7; +3.4 to +46.5) | 3/30 (-6.7; -24.8 to +11.5) |
| **whole line** | 95/150 | 121/150 (+17.3; +7.2 to +27.0) | 88/150 (-4.7; -15.5 to +6.3) | 100/149 (+3.8; -7.0 to +14.4) | 115/150 (+13.3; +3.0 to +23.3) | 94/150 (-0.7; -11.5 to +10.2) |

### Discovery, without the pointer probe

The whole-line figure here covers the two probes other than the pointer probe. The pointer probe has its own section.

| probe (line) | present | absent | no-hook | no-skills | no-claude-md | mcp |
|---|---|---|---|---|---|---|
| cold agent | 17/118 | 36/118 (+16.1; +5.5 to +26.3) | 16/118 (-0.8; -9.8 to +8.2) | 33/118 (+13.6; +3.1 to +23.7) | 16/118 (-0.8; -9.8 to +8.2) | 21/118 (+3.4; -6.1 to +12.8) |
| finds a ruling | 113/118 | 115/118 (+1.7; -3.6 to +7.3) | 113/118 (+0.0; -5.8 to +5.8) | 112/118 (-0.8; -6.9 to +5.1) | 100/118 (-11.0; -19.0 to -3.5) | 52/118 (-51.7; -60.7 to -41.2) |
| **whole line** | 130/236 | 151/236 (+8.9; +0.0 to +17.6) | 129/236 (-0.4; -9.3 to +8.5) | 145/236 (+6.4; -2.5 to +15.1) | 116/236 (-5.9; -14.8 to +3.1) | 73/236 (-24.2; -32.5 to -15.3) |

### Notes on reading the tables

- **The `documentation` absent arm is far below every other arm.** Against the `absent` arm, which keeps `docs/`, the documents add +44.2 points on the four-probe line (96 of 120 against 43 of 120, +32.2 to +54.3) and +50.0 points on the leak-kept line (53 of 60 against 23 of 60, +33.6 to +62.7). Pooled over both lines, it is 149 of 180 against 66 of 180, +46.1 points, +36.6 to +54.4. The batch of 2026-09-30 measured +57.5 points. The two do not compare, because the contract of the recorder changed between them.
- **Removing the whole layer lowered sufficiency by 15.0 points** (-23.5 to -6.7), and nearly all of it comes from two probes. The `discharge` probe fell from 24 of 30 to 7 of 30, and the `accepted event` probe fell from 30 of 30 to 23 of 30.
- **Removing the hook alone lowered the `accepted event` probe from 30 of 30 to 7 of 30.** That is the largest single-component effect in the campaign (-76.7 points, -88.2 to -55.7). The probe keeps its leak string in the `present` arm. The `no-skills`, `no-claude-md` and `mcp` arms hold at 30 of 30. The campaign did not grade why the hook matters here, so the finding names an effect and no mechanism.
- **The navigability result for `absent` (+17.3 points) comes from the inverted probe.** Without the `governs line` probe, the `absent` arm is 95 of 120 against 95 of 120 for `present`, +0.0 points (-10.3 to +10.3). The same exclusion gives `no-hook` -9.2 (-19.9 to +1.9), `no-skills` +3.2 (-6.9 to +13.2), `no-claude-md` +6.7 (-3.0 to +16.3) and `mcp` -1.7 (-12.1 to +8.8). This exclusion was chosen after the data was read, so it is a sensitivity reading and not a result.
- **Removing `CLAUDE.md` lowered one discovery probe.** The `finds a ruling` probe fell from 113 of 118 to 100 of 118 (-11.0 points, -19.0 to -3.5).

## The pointer probe, apart from the rest

The pointer probe asks whether the document a session opens is the document that the corpus offers as a pointer for the task. [An earlier evaluation](what-a-blind-grade-of-the-two-routing-paths-offers-showed.md) and [another](the-pointer-probe-grades-a-session-against-the-router-s-own-first-pick-so-a-correct-session-that-declines-a-wrong-pointer-fails.md) say what the probe grades. It runs at 118 repetitions in each arm, and its rates are small, so it is the least powered line of the campaign.

| probe | present | absent | no-hook | no-skills | no-claude-md | mcp |
|---|---|---|---|---|---|---|
| pointer | 7/118 | 1/118 (-5.1; -10.9 to -0.2) | 0/118 (-5.9; -11.7 to -1.6) | 30/118 (+19.5; +10.4 to +28.6) | 16/118 (+7.6; -0.1 to +15.6) | 15/118 (+6.8; -0.8 to +14.6) |

The `present` arm satisfied it in 7 of 118 sessions (5.9%). The `no-skills` arm satisfied it in 30 of 118 (25.4%), a difference of +19.5 points (+10.4 to +28.6). The `absent` and `no-hook` arms satisfied it in 1 and 0 sessions, and the intervals of both differences lie below zero. Every cell is at most 25.4%, and each interval spans 10 points or more. The task that asked for this page put the need at about 400 repetitions for each cell. This page did not re-derive that figure, and no committed file states it. So the table above is underpowered at 118, and nothing here should be read as a ranking of the arms on this probe.

## What the campaign found about each component arm

These statements are the corpus's statement of the findings for #1659. Each one quotes the whole-line figure against the `present` arm. A figure whose interval contains zero is stated as no detectable effect.

**No hook (the intent hook removed).** The effect is large on one probe and absent on the rest. Sufficiency, four probes: +2.5 points, no detectable effect. Leak-kept: -38.3 points (-51.0 to -25.6), all of it from the `accepted event` probe at 7 of 30. Navigability: -4.7 points, no detectable effect, although the `figure on the site` probe fell from 30 of 30 to 22 of 30. Discovery: -0.4 points, no detectable effect. The pointer probe fell to 0 of 118.

**No skills (the skills removed).** No detectable effect on sufficiency (-3.3), leak-kept (+0.0) or navigability (+3.8). Discovery rose by 10.7 points over all three probes (+3.4 to +17.9), and 19.5 of the pointer probe's points and 13.6 of the `cold agent` probe's points are that rise. The skills may lower discovery in this corpus. The campaign did not test why.

**No `CLAUDE.md` (the standing instructions removed).** No detectable effect on sufficiency (-0.1) or leak-kept (+0.0). Navigability rose by 13.3 points (+3.0 to +23.3), and the inverted `governs line` probe supplies it: without that probe the figure is +6.7, and it contains zero. Discovery was -1.4 points over the three probes, and the `finds a ruling` probe fell by 11.0 points.

**The MCP server added.** No detectable effect on sufficiency (+0.0), leak-kept (+0.0) or navigability (-0.7). Discovery fell by 13.8 points (-20.5 to -7.0) over the three probes, and the `finds a ruling` probe fell from 113 of 118 to 52 of 118 (-51.7 points, -60.7 to -41.2). Among the `mcp` sessions on that probe, 13 of 76 that called the server satisfied it, and 39 of 42 that did not. The probe expects that the session opens the document. A session that gets its answer from the server often does not open it. So the arm measured how a session uses the server, and the finding is that the server displaces the reading of documents in this probe. The next section lists how rarely the server was called at all.

**The whole layer removed.** Sufficiency fell by 15.0 points (-23.5 to -6.7), and leak-kept fell by 11.7 points (-22.2 to -3.2). Navigability rose by 17.3 points, and that rise is the inverted probe. Discovery rose by 4.2 points, with an interval from -3.0 to +11.4, so no detectable effect.

**The documents.** Large. +44.2 and +50.0 points on the two sufficiency lines, as the notes above state.

## Defects that the next test must fix first

These are findings, and this page fixes none of them.

1. **The `governs line` probe key is inverted.** The `present` arm satisfied it in 0 of 30 sessions and the `absent` arm in 26 of 30. A probe that the layer fails and the stripped arm passes is not a measure of the layer.
2. **The `mcp` arm barely calls the server.** The count is the sessions with no tool name that begins `mcp__` in the transcript. By line: 47 of 120 on the four-probe line, 60 of 60 on the leak-kept line, 117 of 150 on navigability, and 140 of 354 on discovery. That is 224 of 330 in batch A (68%) and 140 of 354 in batch B (40%). On the navigability probes `figure on the site` and `adjudication`, no session called the server. A comparison of the `mcp` arm with `present` is therefore mostly a comparison of two arms that did not use it.
3. **Eight of the eleven batch A probes sit at the ceiling.** In the `present` arm, `tombstone`, `citation`, `unmeasured claim`, `accepted event`, `status in pull request`, `register`, `figure on the site` and `adjudication` are each 30 of 30. A probe at the ceiling in the `present` arm cannot show a gain from the layer, and five of them are also 30 of 30 in the `absent` arm: `tombstone`, `citation`, `status in pull request`, `register` and `adjudication`.
4. **The documentation tier is undigested.** Its two lines carry 161 of the 316 batch A paths. Its comparison with the `absent` arm is the strongest result of the campaign. The check in clause 2 read the calls that name a host path, and nobody has read the 137 paths of line 7 for any other pattern.

## The cost

The recorded cost of the 26 lines is 149,681 cents, which is $1,496.81. Batch A recorded 74,701 cents and batch B recorded 74,980 cents. The `campaign` tier recorded 61,268 cents in batch A and the `documentation` tier recorded 13,433. The owner approved about $2,142, a cap of 214,200 cents. So the recorded cost is 69.9% of the approval. The cost excludes the discarded session of the next section, the sessions that ended with status 5, and any spend that the recorder did not write into a transcript, so it is a floor under the money spent and not a statement of it.

| arm, both batches | recorded cents |
|---|---|
| `present` | 24,078 |
| `absent` | 22,382 |
| `no-hook` | 22,233 |
| `no-skills` | 24,345 |
| `no-claude-md` | 22,630 |
| `mcp` | 20,580 |
| `documentation` absent | 13,433 |

## A note on selection

**Exactly one session was drawn again.** The session `L7-documentation-absent-p3-r4` was recorded first and then discarded under an owner ruling. It wrote three auto-memory files under its configuration directory, and the recorder listed them as produced artifacts. Those paths carry no `findings` key, so the grader could not grade the session, and `headwater generate` would not compare the line while it stood. The owner ruled that the session be recorded once more, and no more. The documentation tier had spent 12,878 cents against a ceiling of 13,000, so the owner lifted the ceiling by 300 cents for that one job. The new session ended with status 0 and cost 42 cents. The transcript keeps the discarded session outside the repository, and its cost is not in the 9,517 cents of line 7. The cost of the discarded session is in no committed file, so this page leaves it out.

The redraw is a selection effect on one cell of one arm. The first session was discarded because of what the recorder did, and not because of its answer, so the bias runs in no stated direction. But the discarded session is not committed, and a reader cannot check that. The cell is `p3` of line 7, the unmeasured-claim probe, where the arm is 4 of 30.

**The ten later draws are not a redraw.** The ceiling refused 10 jobs before they started, and no answer chose which jobs they were. They were drawn on 2026-10-09 under the owner's approval of 800 cents, they complete the plan of 30 repetitions for each probe, and they are not a conditional redraw. The owner also ruled that the 4 sessions that ended with status 5 are accepted as unrecorded and not drawn again, to avoid selection bias, because a session drawn again after a failure could differ from one that did not fail. The first-drawn of the 10 cost 232 cents against a tier average near 76; this page reports the figure and does not explain it.

## The obligations and spec 13

Each of the four records stays open, and the reason replaces the earlier reason, "pending #1472". The campaign that #1472 designed has now run. This page restates the records, and it does not discharge any.

- **HW-OBL-0012 (the human half of public presence).** The machine half has a powered reading now. On discovery, the `present` arm satisfied 137 of 354 sessions and the `absent` arm 152 of 354, a difference of +4.2 points for `absent` (-3.0 to +11.4). The interval contains zero, so the layer is not shown to help a cold session find a document. The human half still has no instrument, and that is why the record stays open. It waits on a ruling.
- **HW-OBL-0013 (the counted tombstone).** All six `campaign` arms answered `absent` in 30 of 30 sessions, and the `documentation` absent arm answered `absent` in 30 of 30. No arm met a tombstone, because this corpus serves none. The reading stays the control reading. The record stays open until an export profile serves a counted tombstone.
- **HW-OBL-0014 (the adjudication from the losing document).** The `adjudication` probe is 30 of 30 in all six `campaign` arms. Every arm is at the ceiling, and the pair is still a supersession and not an adjudication. The record stays open until a document carries `overrides`.
- **HW-OBL-0016 (a cue and traversal precision).** The navigability line shows +17.3 points for `absent` against `present`, and the inverted probe supplies it. Without that probe the figure is +0.0 (-10.3 to +10.3). The corpus authors no cue for a session to follow, so the campaign graded the summary fallback and not a cue. The record stays open.

The paragraph in spec 13 that begins "The counterfactual campaign of 2026-09-30 reads four entries above" is restated to cover this campaign in the same pull request. The records HW-OBL-0012, 0013, 0014 and 0016 each gain a `traces_to` edge to the runs that this page reads.

**A sentence owed to spec 15.** A refused write is not in `produced`, so a session with no artifact is a scored failure. The intake line of run 20261003-1026 said to land the sentence after the last session of batch B, because a documentation change moves the plan's tree digest. Batch B has ended, so the sentence lands in spec 15 with this page.

## The deferred owner question

The product owner filed a ruling, `#1659-pull-direction`: pull instead of push as the direction for prompt-time routing. The owner answered on 2026-10-04: "1659 - remind me then", which defers it until the `mcp` arm reports. The `mcp` arm has now reported, and this page does not answer the question. The facts that bear on it are the 224 of 330 batch A sessions and 140 of 354 batch B sessions with no call to the server, and the fall of the `finds a ruling` probe from 113 of 118 to 52 of 118 when the server was available.

## What this page could not derive

- The reason a repetition is missing from a line. The committed files show the gap and not the cause.
- The status code of the four unrecorded sessions, as committed text.
- The cost of the discarded session. No committed file states it.
- The power calculation behind "about 400 repetitions for each cell" in the pointer probe.
- Whether an outside-workspace try succeeded, from committed files. The transcripts blank every tool result, so the check in clause 2 rests on raw streams that are not committed.

## Figures that other sources state differently

The results win. The commit message of #1697 counts 314 paths outside the workspace in batch A and 135 on line 7. It was written before the redraw of line 7. The committed results at that commit counted 313 and 134, and the transcripts now count 316 and 137, because the ten later draws added 3 paths to line 7. The same message counts 214 of 354 discovery sessions with at least one MCP call, which agrees with the 140 sessions without one that this page reports.
