---
id: HW-EVAL-what-the-counterfactual-campaign-of-2026-09-30-measured-by-component
status: current
status_since: 2026-09-30
summary: "The documents under docs/ raised the sufficiency rate by 57.5 points, and the hooks and skills showed no detectable gain, at a cost of $304.46."
last_verified: 2026-09-30
title: "What the counterfactual campaign of 2026-09-30 measured, by component"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-RUN-campaign-of-2026-09-30-campaign-tier-present-arm-sufficiency
    - HW-RUN-campaign-of-2026-09-30-campaign-tier-absent-arm-sufficiency
    - HW-RUN-campaign-of-2026-09-30-documentation-tier-absent-arm-sufficiency
    - HW-RUN-campaign-of-2026-09-30-campaign-tier-present-arm-navigability
    - HW-RUN-campaign-of-2026-09-30-campaign-tier-absent-arm-navigability
    - HW-RUN-campaign-of-2026-09-30-campaign-tier-present-arm-discovery
    - HW-RUN-campaign-of-2026-09-30-campaign-tier-absent-arm-discovery
---

# What the counterfactual campaign of 2026-09-30 measured, by component

This page is for a reader who wants to know what Headwater changes in what an agent does, and what it cost to find out. It reads one batch of paid sessions and splits the result by component. The owner ruled on 2026-09-30 that the results stand as evidence, with the limits below stated, and that the write-up splits by component ([#1384](https://github.com/headwater-ai/headwater/issues/1384)).

## What ran

The batch recorded 658 of 660 planned sessions on 2026-09-30, on `claude-sonnet-5`, with a cap of 80 turns. Every session ran in a fresh copy of one archive of the pin `109abba7`. Seven transcripts hold the batch, and `headwater generate` grades each one into a result page under `docs/probe-results/`.

There are two ablations. The `campaign` tier's absent arm removes the governance layer: `CLAUDE.md`, `.claude/`, `.githooks/` and `.headwater/`. It keeps `docs/`. The `documentation` tier's absent arm removes the governance layer and `docs/` too. So three comparisons are available:

- Present against the `campaign` absent arm is what the governance layer changes. That layer is the hooks, the skills, the standing instructions and the engine binary.
- The `campaign` absent arm against the `documentation` absent arm is what the documents alone change ([HW-OBL-0223](../obligations/0223-no-run-measures-whether-the-documents-under-docs-change-what-an-agent-builds.md)).
- Present against the `documentation` absent arm is what both change together.

## The finding, by component

**The documents have a large measured effect.** On the sufficiency selection, the `campaign` absent arm satisfied 114 of 119 graded sessions (95.8%, 90.5% to 98.2%). The `documentation` absent arm satisfied 46 of 120 (38.3%, 30.1% to 47.3%). The difference is +57.5 points, in a 95% Newcombe interval of +47.1 to +66.0 points ([the result](../probe-results/campaign-of-2026-09-30-documentation-tier-absent-arm-sufficiency.md)).

**The hook and skills layer shows no detectable gain.** Each comparison below is present against the `campaign` absent arm, from the result pages.

- Sufficiency: 114 of 119 against 114 of 119, +0.0 points, -5.8 to +5.8. Both arms are at the ceiling, so this selection cannot show a gain ([the result](../probe-results/campaign-of-2026-09-30-campaign-tier-present-arm-sufficiency.md)).
- Navigability: 69 of 90 (76.7%, 66.9% to 84.2%) against 68 of 90 (75.6%, 65.8% to 83.3%), +1.1 points, -11.3 to +13.5. So the batch excludes a gain of more than about 13 points ([the result](../probe-results/campaign-of-2026-09-30-campaign-tier-present-arm-navigability.md)).
- Discovery: 9 of 60 (15.0%, 8.1% to 26.1%) against 5 of 60 (8.3%, 3.6% to 18.1%), +6.7 points, -5.3 to +18.7. The interval contains zero ([the result](../probe-results/campaign-of-2026-09-30-campaign-tier-present-arm-discovery.md)).

**Two sufficiency probes measured the always-loaded context and not the corpus.** The description of the `headwater-authoring` skill names HW-DR-0052 and `status: current`, and every present-arm session loads it. On the status probe, 30 of 30 present-arm sessions answered with no tool call. On the accepted-event probe, 29 of 30 did. So these two present-arm rates read the skill description. The absent arm has no `.claude/`, and 29 of its 30 status-probe sessions named `0052` in a call before they answered.

**Two components are not measured.** No arm isolates the MCP server, and no probe grades the quality of governance itself. [#1472](https://github.com/headwater-ai/headwater/issues/1472) designs the arms and the harder probes that would measure them. The claims of HW-OBL-0012, HW-OBL-0013, HW-OBL-0014 and HW-OBL-0016 stay open until it runs.

### Each probe

The result pages list each verdict and print a rate for each line. They do not print a rate for each probe. The rates below are the counts of those verdicts, with a 95% Wilson interval, as the build of #1384 posted them on the issue.

| Line | Probe | Present | Absent |
|---|---|---|---|
| sufficiency | tombstone | 29/30 (83.3-99.4) | 30/30 (88.6-100.0) |
| sufficiency | accepted event | 30/30 (88.6-100.0) | 27/30 (74.4-96.5) |
| sufficiency | status of a settled decision | 30/30 (88.6-100.0) | 29/30 (83.3-99.4) |
| sufficiency | unmeasured claim | 25/29 (69.4-94.5) | 28/29 (82.8-99.4) |
| navigability | register (`not_opened`) | 28/30 (78.7-98.2) | 26/30 (70.3-94.7) |
| navigability | adjudication (`opened`) | 30/30 (88.6-100.0) | 30/30 (88.6-100.0) |
| navigability | cited | 11/30 (21.9-54.5) | 12/30 (24.6-57.7) |
| discovery | corpus descriptor | 7/30 (11.8-40.9) | 4/30 (5.3-29.7) |
| discovery | pointer | 2/30 (1.8-21.3) | 1/30 (0.6-16.7) |

The `documentation` absent arm gave tombstone 30/30 (88.6-100.0), accepted event 0/30 (0.0-11.4), status 12/30 (24.6-57.7) and unmeasured claim 4/30 (5.3-29.7).

## The cost of each measurement

Each row is one claim, the arms behind it, the sessions those arms recorded, and what they cost. The cents come from the `cost_cents` of each transcript.

| Claim | Arms | Sessions | Cost |
|---|---|---|---|
| What the governance layer changes on sufficiency (HW-OBL-0013) | `campaign` present and absent, sufficiency | 119 + 119 | 6557 + 5640 = 12197 cents ($121.97) |
| What the documents alone change (HW-OBL-0223) | `campaign` absent and `documentation` absent, sufficiency | 119 + 120 | 5640 + 6001 = 11641 cents ($116.41) |
| What the governance layer changes on navigability (HW-OBL-0014, HW-OBL-0016, HW-OBL-0197) | `campaign` present and absent, navigability | 90 + 90 | 3678 + 2393 = 6071 cents ($60.71) |
| What the governance layer changes on discovery (HW-OBL-0012, HW-OBL-0010) | `campaign` present and absent, discovery | 60 + 60 | 3686 + 2020 = 5706 cents ($57.06) |

The `campaign` absent sufficiency arm serves two claims, so its 5640 cents are in two rows. Each arm counted once, the seven lines recorded 658 sessions for 29975 cents ($299.75).

**The batch cost more than its recorded sessions.** Two sessions failed at the recorder and spent 471 cents, so the driver summed 30446 cents ($304.46). The batch was stopped and started once when the owner raised its cap. Four sessions were in flight at the stop, and the tokens they spent are in no cost file.

**The program cost more than the batch.** Four earlier recordings led to this one.

| Recording | Sessions | Cost |
|---|---|---|
| The first campaign pilot of 2026-09-28, seven lines, now `deprecated` | 66 | 1704 cents |
| The second campaign pilot of 2026-09-28, seven lines, now `deprecated` | 22 | 791 cents |
| The batch of 2026-09-28, five lines, withdrawn by the owner on 2026-09-29 | 540 | 18010 cents recorded, about 18863 spent |
| The status-probe pilot of 2026-09-29, two lines | 20 | 277 cents |
| The tombstone-probe pilot of 2026-09-29, two lines | 20 | 601 cents |
| The batch of 2026-09-30, seven lines | 658 | 29975 cents recorded, 30446 spent |

The six recordings hold 1326 sessions and 51358 recorded cents ($513.58). With the cost that the transcripts do not record, the program spent about 52682 cents ($526.82). No transcript records 853 cents of the 2026-09-28 batch. They are four sessions that the cap stopped and that ran again, as [#980](https://github.com/headwater-ai/headwater/issues/980) reports. The pilots' cents come from the `cost_cents` of their transcripts under `docs/probe-runs/`.

## Limits

**Some sessions reached this repository from outside their workspace.** Nothing held a session inside its workspace. This page counts a session as a leak when a call names a source of this repository that its arm did not give it. There are three channels:

- A copy of this repository on the host, or the batch directory that held the pilot's trees. The host holds four copies: the owner's checkout, a copy under `/mnt/new-projects/`, the checkout of the CI runner, and a full tree under `/tmp/v1384r3/base`.
- A `headwater` binary on the host, outside the workspace, in an arm that holds none.
- This repository on GitHub, through `gh api` or a web search or fetch.

By that rule, 58 of 658 sessions leaked. By line: present sufficiency 9/119, absent sufficiency 5/119, present navigability 8/90, absent navigability 0/90, present discovery 5/60, absent discovery 3/60, and `documentation` absent 28/120. 17 of the 58 named a copy on the host, and 2 of those ran the CI runner's binary. 35 named only the batch directory, 5 used GitHub, and 1 ran a host binary that is in no copy. HW-DR-0052 is the answer of the status probe. 6 `documentation` absent sessions read it from a copy on the host, and 1 fetched it from GitHub.

The rule is narrower than "a path outside the workspace". 118 sessions name such a path. Most of those paths are scratch files under `/tmp` or the harness's own `~/.claude`, which hold no copy of this repository. The transcripts blank every tool result. So a counted session is one whose call names the source, and nothing shows that it read content there.

The count has three more limits. It knows only the four copies above, so a session that read a copy somewhere else on the host is not counted. It reads absolute paths only. Three sessions listed the batch directory through a relative path, such as `find .. -maxdepth 2`, and this page does not count them. With them the count is 61, and no separation changes. Three more sessions walked up out of the workspace with `cd $d` and a filtered listing. Nobody checked what those listings named, so they are not counted either.

Without the 58, the rates that move are these:

- present unmeasured claim 16/20 (58.4-91.9), and absent unmeasured claim 23/24 (79.8-99.3)
- present cited 7/22 (16.4-52.7), present descriptor 7/25 (14.3-47.6), absent descriptor 4/29, and absent pointer 1/28
- `documentation` absent: tombstone 28/28, accepted event 0/26, status 2/19 (2.9-31.4), and unmeasured claim 1/19 (0.9-24.6)

No separation and no non-separation above changes without them. Where the leak moved a rate, it raised the `documentation` absent arm, so the effect of the documents is larger without it. The committed transcripts state 44 sessions, because their count read two channels of the first kind and no other. [#1467](https://github.com/headwater-ai/headwater/issues/1467) gives each session a sandbox.

**The model pin is a name.** Every transcript records `served_version: claude-sonnet-5`, which equals the model name, because the provider exposed nothing finer. The sessions ran under Claude Code 2.1.285, and no transcript records that version.

**The host configuration was loaded in both arms.** Each session loaded the owner's user-level Claude Code configuration, which connects mail and drive tools and bypasses permissions. It was the same in both arms. #1467 removes it.

**The corpus was not frozen between the two arms' sessions.** This repository merged changes while the batch ran. Each session read one archive of the pin, so both arms of a pair read the same bytes. Since the recording, `docs/spec/09-open-questions.md` and `docs/spec/12-check-layer.md` changed. So the navigability and discovery results say that the read set moved, and each verdict is graded over the documents that the session met.

**The sealed present arm lists the probes in `nav.yml`.** 69 of the 529 entries of `.headwater/nav.yml` in the sealed present tree name a file on the three instrument shelves, which the ablation removes. So a present-arm session can read the title of each probe. None of the 69 names a document that the seal deleted. Nothing controls this difference, and each transcript states it.

**The `cited` probe does not separate the arms.** It gave 11/30 against 12/30, -3.3 points, -26.4 to +20.2. On 2026-09-28 it gave 8/30 against 4/30, before the grader saw a file written through Bash. Its validity stays an open limit on that probe.

**The tombstone probe is the control reading.** It exercises no tombstone, because this corpus declares no export profile that filters a document. [HW-OBL-0013](../obligations/0013-no-probe-tests-whether-a-counted-tombstone-stops-a-confident.md) states what that means for the claim.
