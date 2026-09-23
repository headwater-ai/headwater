---
id: HW-OBL-0207
status: current
status_since: 2026-09-23
summary: "A build-order run reports the points of the owner's 5-hour and 7-day plan windows that it spent, divided by the issues it closed. A figure for one issue needs each dispatch priced by model and token type, then matched to the issue its pull request closes."
last_verified: 2026-09-23
title: "The plan usage of one issue is known only as the average of its run"
waiting_on: measurement
---

# The plan usage of one issue is known only as the average of its run

## Context

The owner plans delivery against the two rate-limit windows of the plan: 5 hours and 7 days. The question is how many issues fit in what is left of each window.

`tools/run/run-dir.sh` records one sample of both windows when a run starts, at each ledger line, and when the run ends. `run-dir.sh usage` then divides the points that the run spent by the issues it closed. That figure is an average over one run. It cannot tell that one issue cost more than another, and it cannot forecast by the size or the type of an issue.

An investigation on 2026-09-23 found what a figure per issue needs. The notes are on the branch `worktree-cost-attribution-handoff`, in `.claude/notes/cost-attribution-handoff.md`. Four findings bear on this record:

- The harness writes one transcript per dispatched agent, with a `.meta.json` beside it. Each assistant line of a transcript carries its token usage, and one response is written as several lines that share one usage record.
- The `description` of a dispatch names its issue in several forms: `Build issue #833`, `Verify issue 479 PR`, `Merge PR 1017 for issue 479`. Nine of 42 `hw-verify` dispatches name only a pull request, and those nine include every re-verification after a veto.
- A parent turn is the unit that [HW-PD-0003](../process/decisions/0003-a-dispatch-pays-when-it-retires-more-parent-turns-than-it-costs.md) prices, and it measures orchestration waste. The plan windows do not count turns. They count tokens, weighted by model and by token type.
- A `pr-link` record in a session transcript is a pointer to the current pull request, and a parent session writes it on most turns. A parent session that ran a whole build order has no `cost-state` record at all.

## Obligation

The corpus owes a measure of plan usage for each closed issue, in points of each window. It has five parts:

1. Price each dispatch transcript by model and by token type: input, cache read, cache write and output. Count each `message.id` once, as `tools/run/run-census.sh` does.
2. Match each dispatch to one issue. Resolve a dispatch that names only a pull request through the `Closes #N` text of that pull request. Keep a `Refs #N` pull request apart, because it did not finish the issue.
3. Charge a vetoed build, a re-verification and a failed attempt to the issue that the branch closed at last. The cost of rework is part of the cost of an issue.
4. Share the usage of the parent session among the issues that its run closed.
5. Calibrate the priced usage against the samples in `usage.jsonl`. Match each rise of a window against the priced usage of all sessions in the same interval.

Three limits apply to the result. The windows also count usage outside this repository, so the calibration holds only when this repository is most of the usage. Some models have a separate limit, so a mix of models does not reduce to one figure. A veto makes the cost of an issue very uneven, so the report gives a median and an 80th percentile and not a mean.

## Discharge

This record waits on the average that `run-dir.sh usage` reports over at least five runs. When those runs exist, the owner compares the spread of the average with the precision that delivery planning needs.

This discharges in one of two ways. A tool reports the five parts above for each closed issue, and the report of a run quotes its median and 80th percentile. Or the owner rules that the average of a run is sufficient, and writes that ruling into this record.
