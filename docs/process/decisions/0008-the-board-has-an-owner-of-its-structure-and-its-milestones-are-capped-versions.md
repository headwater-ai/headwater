---
id: HW-PD-0008
status: current
status_since: 2026-09-24
summary: "One agent owns where work sits on the board and never what the work is, and each milestone is a capped version."
last_verified: 2026-09-24
title: "The board has an owner of its structure, and its milestones are capped versions"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5-5
  activity: draft
  evidence_basis: evidenced
relations:
  governs:
    - .claude/agents/headwater-product-owner.md
---

# The board has an owner of its structure, and its milestones are capped versions

## Context

The verification of each iteration asks whether a branch is correct. It does not ask whether the branch was worth building. It does not ask whether the branch belonged to the plan, or whether the plan is still the work. One iteration at a time, these three questions are not visible. Across ten iterations, the answers are clear. An agent in the middle of a run has every reason to find the next thing to build. So it cannot also weigh that thing against the plan.

Five failures on this board show the gap. Nothing in them was bad work. Each was good work that entered the board with no comparison against other good work.

**The reader failure.** On 2026-08-15 this repository filed 45 issues and closed 16. Of the new issues, 23 were findings from a run of the engine over its own corpus. Every one was real, and none had a reader outside this repository. Nobody compared any of them against the tutorial, the install path or the first adopter, which were open the whole time.

**The plan failure.** Over 2026-08-12 and 2026-08-13, 41 issues closed, and every one carried a milestone. From 2026-08-15 to 2026-08-17, 33 issues closed, and 28 of them carried none. So 85% of three days of work sat outside the plan. In those days, M5, M6, M7 and the library track stood still. The value rule caught the reader half: 22 of those closes carried the `self-audit` label. Nothing measured the other half.

**The intake failure.** The value rule already kept work with no outside reader off the tracker. But the rule acted at the top of a run and at every fifth merge, and a run files an issue at neither moment. Over the 30 days to 2026-09-20, the board opened 279 issues and closed 259, so the open count did not move. Of the 28 issues with no milestone, 21 were ten days old or less.

**The bookkeeping failure.** On 2026-08-20, four of the eight milestones, M1 to M4, held zero open issues and were still open. The build order had a policy to close an epic and none to close the milestone around it. A board that shows eight milestones in progress, when four are finished, tells a reader that the plan has barely started.

**The single-issue failure.** On the same day, one issue, #73, held M5 open. Its skills and its maintainer agent were on disk. Its only open half was a measurement that answered to the bar of M7. So one misfiled issue held a milestone open and moved the selection order past it to M6 and M7.

The milestones themselves were part of the cause. The M-sequence was a bootstrap plan: it named what the system must do, in the order that the parts depend on each other. Its themes had no end that a release could name and no date, so none of the later ones closed. Version 0.1 shipped on 2026-09-07.

## Decision

**One agent owns the structure of the board, and never its scope.** Structure is where work sits and how urgent it is: milestones, milestone membership, folds, and the `bug` and `adopter-blocking` labels. Scope is what the work is and when it is done, and scope stays with the owner. The agent is `headwater-product-owner`. It runs outside the context of any builder. It runs standalone, at the top of a run, at every fifth merge and at the end of a run. It names every write with its reason, in a form that the owner can reverse.

**A run files no issue.** A stage writes a finding that is not its own issue as one line in the intake file of the run. The product owner rules on each line with one of four rulings: fold, record, admit or backlog. So the comparison against other work occurs before the filing, and not after it.

**Every open milestone is a version.** The owner moved the board to version milestones on 2026-09-20. M7, M8 and M9 closed, and their open issues moved forward. R1 took the name 0.1, and M6b took the name 0.7 with its bar unchanged. The library track and the ecosystem track became `track:` labels over the backlog. Neither has an end that a release can name.

**A version milestone is a statement, a cap and a date.** The statement is one sentence about what an adopter can do after the release that they could not do before. The cap is 15 open issues. The date is the owner's. An issue enters a version only when the statement is false without it. Other real work carries no milestone, and that is the backlog. The versions are worked in numeric order.

**The product owner writes questions, and the owner answers them.** Each decision that the owner owes becomes one `RULING` block with a recommended answer and a `defer` option. The agent that dispatched the product owner puts the blocks to the owner.

## Consequences

`.claude/agents/headwater-product-owner.md` carries the rules that the agent obeys, and this record carries the evidence for them. The five failures map to the five parts of its report. Part 1 reports the share of closes with no milestone, which is the number that would have shown the plan failure while it occurred. Part 2 closes a finished milestone. Part 3 finds a misfiled issue.

A closed version milestone is a release that the owner has not cut. The product owner reports it first, and only the owner starts a release. No agent pushes a tag.

A cap that only grows is not a cap. So an admission into a full version names the issue that it moves to the backlog.

This record does not give the product owner scope. It does not close an issue as done, and it does not edit a Done-when clause or a bar. A version statement that the owner has not accepted is a proposal. A window where the intake rulings admit more than they fold and record is a finding against this decision. The report states it.
