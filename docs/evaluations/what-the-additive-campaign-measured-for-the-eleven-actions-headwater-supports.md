---
id: HW-EVAL-what-the-additive-campaign-measured-for-the-eleven-actions-headwater-supports
status: current
status_since: 2026-11-22
summary: "On held-out probes, what `headwater init` ships gained over the documents alone on two code rows and lost on supersede or retire. Seven rows showed no detectable effect. Change taxonomy is not measurable without the engine. Of the tuned layers, only the engine showed a gain."
last_verified: 2026-11-22
title: "What the additive campaign measured for the eleven actions Headwater supports"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0112
    - HW-EVAL-what-the-tuning-round-of-the-additive-campaign-measured-layer-by-layer
    - HW-RUN-additive-final-round-of-2026-11-09-batch-a-scenario-0
    - HW-RUN-additive-final-round-of-2026-11-09-batch-a-scenario-2
    - HW-RUN-additive-final-round-of-2026-11-09-batch-a-scenario-1
    - HW-RUN-additive-final-round-of-2026-11-09-batch-a-scenario-6
    - HW-RUN-additive-final-round-of-2026-11-09-batch-b-scenario-1
    - HW-RUN-additive-final-round-of-2026-11-09-batch-b-scenario-6
    - HW-RUN-additive-final-round-of-2026-11-10-batch-b2-scenario-1
    - HW-RUN-additive-final-round-of-2026-11-10-batch-b2-scenario-6
    - HW-RUN-additive-final-round-of-2026-11-12-batch-b3-scenario-1
    - HW-RUN-additive-final-round-of-2026-11-12-batch-b3-scenario-6
    - HW-RUN-additive-tuning-of-2026-11-02-base-re-measure-of-scenario-1
    - HW-RUN-additive-tuning-of-2026-11-02-engine-round-1-best-so-far-scenario-2
    - HW-RUN-additive-tuning-of-2026-11-03-skills-round-1-best-so-far-scenario-3
    - HW-RUN-additive-tuning-of-2026-11-05-hooks-round-1-best-so-far-scenario-4
    - HW-RUN-additive-tuning-of-2026-11-08-mcp-round-2-confirmation-scenario-5
---

# What the additive campaign measured for the eleven actions Headwater supports

## Summary

Phase B of the additive campaign measured how often a session completes each of the eleven actions that Headwater supports. It ran on probes that no tuning change saw, and it compares what `headwater init` ships with the documents alone. [HW-DR-0112](../decisions/0112-the-additive-campaign-measures-eleven-actions-by-five-layers-in-seven-scenarios.md) designs the campaign, and [#1882](https://github.com/headwater-ai/headwater/issues/1882) asks for this report. The record also states the pooled result of each layer that phase A tuned.

**What it found.**

- Two actions gained: refactor across files, +33.3 points, and change governed code, +22.9 points.
- One action lost: supersede or retire, -33.3 points.
- Seven actions showed no detectable effect.
- Change taxonomy has no comparison, because a session cannot do it with the documents alone.
- Of the four tuned layers, only the engine showed a gain, and those figures are tuned.

**What it does not show.** Every session ran one model on this repository, and the code sessions had no compiler. [Limits](#limits) lists each caveat. This record does not discharge an open claim of spec 13. HW-OBL-0012, HW-OBL-0013, HW-OBL-0014 and HW-OBL-0016 each ask about a property that none of the eleven actions measures, so all four stay open.

## The method

**The scenarios.** Scenario 0 has no layer. Scenario 1 adds the documents under `docs/`. Scenario 2 adds the engine to scenario 1. Scenario 6 adds the skills, the hooks and the MCP server to scenario 2, which is what `headwater init` ships. HW-DR-0112 states each scenario and its base.

**The probes.** Phase B ran the `held-out` set of `.headwater/probe.yml`, 22 probes, 2 for each action. No screen and no confirmation of phase A ran this set. Each probe has two or three phrasings of one task, and each session reads one phrasing.

**The counts.** Each row ran 48 sessions in scenario 1 and 48 in scenario 6. A cell stopped at 12 sessions when all 12 passed or all 12 failed. So a cell at 12/12 can stand for a rate of about 95%, and a cell at 0/12 for about 5%. Scenarios 0 and 2 each ran 66 sessions over all the rows, which is 6 for each row. The owner set these counts on [#1881](https://github.com/headwater-ai/headwater/issues/1881) on 2026-11-09, and HW-DR-0112 records them.

**The revision.** Phase B measured the pinned commit `74d8f1baf721368eebffdd800ec4fbb55595f485`, from 2026-11-09 to 2026-11-12. The subject model is `claude-sonnet-5`, on harness 0.6.1.

**The grade.** The cells below come from grader 0.5.3, regraded from the recorded transcripts. No paid session ran again. Three regrades move a cell, each from the one before it: [#1951](https://github.com/headwater-ai/headwater/issues/1951), [#1978](https://github.com/headwater-ai/headwater/issues/1978) and [#1980](https://github.com/headwater-ai/headwater/issues/1980). The frontmatter of this record names the ten transcripts, and each one has a result page under `docs/probe-results/`. The verdicts on those ten pages, summed by action and scenario, give every cell of the table.

**The figures.** A rate is the passes over the graded sessions, with the Wilson 95% interval that a result page prints. A difference is in points, with its Newcombe 95% interval. Each one is the output of `headwater probe compare` over the sessions of one row. The verdict rule of HW-DR-0112 gives each difference one verdict. It is `gain` when the interval lies above zero, `loss` when it lies below zero, and `no detectable effect` when it holds zero.

## The held-out table

Every number in this table is held-out. Scenarios 0 and 2 are the anchors, at 6 sessions for each row.

| # | action | scenario 0 | scenario 2 | scenario 1, documents | scenario 6, what `init` ships | scenario 6 − scenario 1, points [95%] | verdict |
|---|---|---|---|---|---|---|---|
| 1 | ask or locate | 3/6 | 5/6 | 47/48, 97.9% [89.1, 99.6] | 12/12, 100.0% [75.8, 100.0] | +2.1 [-22.2, +10.9] | no detectable effect |
| 2 | change governed code | 1/6 | 3/6 | 21/48, 43.8% [30.7, 57.7] | 32/48, 66.7% [52.5, 78.3] | +22.9 [+3.0, +40.4] | gain |
| 3 | refactor across files | 0/6 | 5/6 | 30/48, 62.5% [48.4, 74.8] | 46/48, 95.8% [86.0, 98.8] | +33.3 [+17.6, +47.8] | gain |
| 4 | add a document | 0/6 | 4/6 | 12/12, 100.0% [75.8, 100.0] | 44/48, 91.7% [80.4, 96.7] | -8.3 [-19.6, +16.4] | no detectable effect |
| 5 | edit a document others depend on | 0/5 | 6/6 | 38/47, 80.9% [67.5, 89.6] | 12/12, 100.0% [75.8, 100.0] | +19.1 [-6.6, +32.5] | no detectable effect |
| 6 | supersede or retire | 0/6 | 5/6 | 9/12, 75.0% [46.8, 91.1] | 20/48, 41.7% [28.8, 55.7] | -33.3 [-53.9, -1.8] | loss |
| 7 | move a document | 0/6 | 6/6 | 47/48, 97.9% [89.1, 99.6] | 12/12, 100.0% [75.8, 100.0] | +2.1 [-22.2, +10.9] | no detectable effect |
| 8 | change taxonomy | 0/6 | 5/6 | not measurable without the engine | 39/48, 81.2% [68.1, 89.8] | no comparison | none |
| 9 | repair a check | 0/6 | 5/6 | 44/48, 91.7% [80.4, 96.7] | 46/48, 95.8% [86.0, 98.8] | +4.2 [-6.9, +15.8] | no detectable effect |
| 10 | review a change | 3/6 | 6/6 | 42/48, 87.5% [75.3, 94.1] | 46/48, 95.8% [86.0, 98.8] | +8.3 [-3.5, +20.9] | no detectable effect |
| 11 | ungoverned control | 6/6 | 6/6 | 47/48, 97.9% [89.1, 99.6] | 12/12, 100.0% [75.8, 100.0] | +2.1 [-22.2, +10.9] | no detectable effect |
| | pooled anchors | 13/65, 20.0% [12.1, 31.3] | 56/66, 84.8% [74.3, 91.6] | | | | |

The pooled anchors give scenario 2 against scenario 0 as +64.8 [+49.4, +75.2], a gain. With no layer, sessions passed 13 of 65. With the documents and the engine, sessions passed 56 of 66. Scenario 0 graded 65 sessions and not 66, because the provider refused one session. Scenarios 1 and 6 have no pooled rate, because the cells that stopped at 12 give the rows unequal weight.

**Raw beside regraded.** Three rows moved under a regrade, and the owner asked for both numbers.

| action | regrade | scenario 0 | scenario 2 | scenario 1 | scenario 6 |
|---|---|---|---|---|---|
| repair a check | raw, before #1978 | 0/6 | 4/6 | 26/48 | 30/48 |
| repair a check | regraded | 0/6 | 5/6 | 44/48 | 46/48 |
| review a change | raw, before #1951 | 0/6 | 4/6 | 20/48 | 14/48 |
| review a change | regraded | 3/6 | 6/6 | 42/48 | 46/48 |
| ask or locate | raw, before #1951 | 3/6 | 5/6 | 43/48 | 12/12 |
| ask or locate | regraded | 3/6 | 5/6 | 47/48 | 12/12 |

**The #1980 corrections.** Grader 0.5.3 moved four cells from the figures that #1978 gave. Change governed code went from 21/46 to 21/48 in scenario 1 and from 32/47 to 32/48 in scenario 6. Three sessions that wrote a scratch file under `/tmp` were outside the denominator, and they now count as failures. Review a change went from 41/48 to 42/48 in scenario 1 and from 45/48 to 46/48 in scenario 6. Two sessions ended with an identifier and a reason, and the recorder now reads the identifier.

## What each row means

1. **Ask or locate.** The documents alone answer almost every question, so the shipped layers have no room to gain.
2. **Change governed code.** The shipped layers help a session change code that a document governs, with a gain of +22.9 points over the documents alone. Two probe defects of [#1979](https://github.com/headwater-ai/headwater/issues/1979) hold both cells down. The workspace had no compiler, as the disclosures state. On the numbered-heading probe, many sessions wrote the right code and did not raise the version of the rule. [#1976](https://github.com/headwater-ai/headwater/issues/1976), merged after phase B, now states that rule beside each rule version.
3. **Refactor across files.** The shipped layers help a session rename across files and fix the callers, with a gain of +33.3 points. The 2 failures in scenario 6 are a missed caller that a compiler would show.
4. **Add a document.** The documents alone pass every session, and the shipped layers show no detectable loss. The failures show the gap of [#1977](https://github.com/headwater-ai/headwater/issues/1977): nothing offers the typed relation to a record that the task names.
5. **Edit a document others depend on.** The shipped layers pass every session, but the difference from the documents alone is not clear of zero.
6. **Supersede or retire.** The shipped layers make this action worse, with a loss of -33.3 points. Sessions trust `headwater check --fix`, leave the new record at `draft`, see a clean check and stop. [#1973](https://github.com/headwater-ai/headwater/issues/1973), still open, is that defect. [#1974](https://github.com/headwater-ai/headwater/issues/1974), merged after phase B, removes a stop rule that halted a session the person had already answered. No session ran again after either change, by the owner's ruling, so this row is the pin above as measured.
7. **Move a document.** The documents alone move a document and repair its links in almost every session.
8. **Change taxonomy.** A session cannot do this action with the documents alone. The oracle edits `.headwater/overlay.yml` and needs the lock, which is part of the engine layer. So the row has no scenario 1 number and no comparison. With what `init` ships, sessions passed 39 of 48. The scenario 6 failures show the gap of [#1975](https://github.com/headwater-ai/headwater/issues/1975): nothing says that the overlay is newer than the lock.
9. **Repair a check.** Both scenarios repair almost every check, so the shipped layers show no detectable effect. Before #1978, one oracle demanded exact words where the task asked to keep the meaning, and the raw cells show that defect, not the product.
10. **Review a change.** This row measures whether a session cites the one record that governs a change, and not a review. Both scenarios cite the right record in most sessions.
11. **Ungoverned control.** The shipped layers do no detectable harm on a task that no document governs.

## Phrasing

Each number above is pooled over the phrasings of its probes. In one held-out cell, the phrasings differ by more than a third: edit a document others depend on, in scenario 1. There, phrasing 1 passed 15 of 16, phrasing 2 passed 14 of 16, and phrasing 3 passed 9 of 15. In every other cell of scenarios 1 and 6, the phrasings of a row differ by a third or less. The nearest is supersede or retire in scenario 6, at 9 of 16, 7 of 16 and 4 of 16. The next is change taxonomy in scenario 6, at 23 of 24 and 16 of 24. The cells of scenarios 0 and 2 have 1 to 4 sessions for each phrasing, which is too few to compare.

The phrasing of each session comes from the task file that its recorded command line names. The verdicts come from the ten result pages. All 934 graded sessions have a phrasing. The forensics of 2026-11-20 gave a larger spread for the shelf-layout probe, 0 of 12 against 8 of 12. That figure was from before #1978 and #1980, and the regraded probe passes 23 of 24 in each scenario.

## The tuned layers, pooled from phase A

Phase A tuned four layers on the 22 tuning probes, pooled at 154 sessions, 7 for each probe. These numbers are tuned: each change was chosen on these probes. Each one is the comparison with its base that HW-DR-0112 registers.

| layer | scenario | passed | against its base | verdict |
|---|---|---|---|---|
| none: the documents | 1 | 81/154 | base of phase A | |
| engine | 2 | 112/154 | +20.1 [+9.3, +30.3] against scenario 1 | gain |
| skills | 3 | 120/154 | +5.2 [-4.4, +14.7] against scenario 2 | no detectable effect |
| hooks | 4 | 126/154 | +9.1 [-0.3, +18.3] against scenario 2 | no detectable effect |
| MCP server | 5 | 122/154 | +6.5 [-3.1, +15.9] against scenario 2 | no detectable effect |

The MCP row is the best so far by the owner's ruling 6721106418 on [#1880](https://github.com/headwater-ai/headwater/issues/1880), and not by the keep rule. Its confirmation against the earlier best so far was +3.9 [-5.5, +13.2].

**The cell indication, unconfirmed.** Each cell below is one action of the tuning probes, at 14 sessions. These cells are tuned and unconfirmed. No phase bought a held-out cell for the skills, hooks or MCP alone. The held-out scenario 6 cell stands beside each row.

| action | held-out scenario 6 | engine, tuned | skills, tuned | hooks, tuned | MCP, tuned |
|---|---|---|---|---|---|
| ask or locate | 12/12 | 14/14 | 13/14 | 12/14 | 13/14 |
| change governed code | 32/48 | 9/14 | 11/14 | 10/14 | 9/14 |
| refactor across files | 46/48 | 11/14 | 13/14 | 13/14 | 14/14 |
| add a document | 44/48 | 5/14 | 8/14 | 11/14 | 8/14 |
| edit a document others depend on | 12/12 | 9/14 | 11/14 | 8/14 | 10/14 |
| supersede or retire | 20/48 | 4/14 | 6/14 | 13/14 | 12/14 |
| move a document | 12/12 | 14/14 | 14/14 | 14/14 | 14/14 |
| change taxonomy | 39/48 | 14/14 | 12/14 | 14/14 | 11/14 |
| repair a check | 46/48 | 7/14 | 10/14 | 9/14 | 6/14 |
| review a change | 46/48 | 11/14 | 8/14 | 8/14 | 11/14 |
| ungoverned control | 12/12 | 14/14 | 14/14 | 14/14 | 14/14 |

On supersede or retire, the hooks passed 13 of 14 and the MCP server 12 of 14, against 4 of 14 for the engine alone. That indication is the opposite of the held-out loss in scenario 6, which has all three layers. Nothing here resolves the difference.

## Disclosures

**Probe defects, from #1979.** Some failures come from a defect in the probe alone, and HW-DR-0112 states the counts. In change governed code, scenario 6, the probes alone account for 8 of 16 failures. In 3 more, a probe defect and a real miss coincide. In scenario 1 of that row, the numbered-heading probe alone accounts for 1 of 27 failures. In 11 more, its defect and a real miss coincide. In supersede or retire, scenario 6, the figure-page probe alone accounts for 4 of 28 failures, and in scenario 1 for 1 of 3. The table counts each of these as a failure. The held-out set opens again for these three probes before any rerun.

**No compiler, from #1981.** The workspace of a code session could not run `cargo build` or `cargo test`. So change governed code and refactor across files each measure a session with no compiler.

**Oracle scripts in the workspace, from #1892.** The seal left the generic oracle scripts and the oracle entries of `.headwater/probe.yml` in each workspace. In scenario 6, 12 sessions read an oracle script, 2 ran one, and all 12 passed. The held argument files were removed, so no answer leaked, but those sessions saw what the oracle would check.

## The spend

Each paid issue had its own ceiling. The figures come from each issue's ledger under its run root, and for #1881 from the owner's comment at the end of the run.

| issue | what it paid for | spent | ceiling |
|---|---|---|---|
| [#1877](https://github.com/headwater-ai/headwater/issues/1877) | the engine layer and the phase A base | $544.96 | $1,300 |
| [#1878](https://github.com/headwater-ai/headwater/issues/1878) | the skills layer | $137.01 | $1,300 |
| [#1879](https://github.com/headwater-ai/headwater/issues/1879) | the hooks layer | $136.24 | $1,300 |
| [#1880](https://github.com/headwater-ai/headwater/issues/1880) | the MCP layer | $575.33 | $1,300 |
| [#1881](https://github.com/headwater-ai/headwater/issues/1881) | phase B | $671.44 | $1,050 |
| | total | $2,064.98 | $6,250 |

The ten phase B transcripts record $653.00 of the phase B spend.

## Limits

- **Tuned beside held-out.** The layer table and the cell indication are tuned. Only the held-out table is a measurement on probes that no change saw.
- **One subject model.** Every session ran `claude-sonnet-5`. Another model can give another table.
- **One corpus.** Every probe ran on this repository. Corpus size, tasks that no document governs and an adopter's own repository are deferred axes, and each is now an obligation record.
- **No compiler.** The two code rows ran with no compiler, as the disclosures state.
- **Small anchors.** Scenarios 0 and 2 have 6 sessions for each row, so a cell there gives a direction and not a rate.
- **Saturated cells.** A cell that stopped at 12 sessions reads as 100% or 0%, but its rate can be near 95% or 5%.
- **The held-out set opens again.** It opens for the three probes of #1979 before any rerun, so a rerun does not use the same set.
