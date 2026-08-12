---
description: Work one iteration of the Headwater build order, then stop
argument-hint: "[issue number or milestone, optional]"
---

Work the Headwater build order (org project "Headwater build order", `headwater-ai/headwater`). Do exactly one iteration, then stop.

`$ARGUMENTS`, if given, names the issue or the milestone to work. Otherwise select by the rules in step 2.

## 1. Read the board first

Never work from memory of the roadmap, because it moves under you.

- `gh project item-list 1 --owner headwater-ai --format json` for status
- `gh issue list --milestone "<lowest M with open issues>" --state open`
- the milestone epic, and any issue the candidate says it depends on

## 2. Pick one issue

Apply these rules in order:

- anything already In Progress and unfinished: finish that first
- anything labeled `correctness-root`, because [spec 12](docs/spec/12-check-layer.md) says every check trusts it silently, so a wrong one is expensive later
- the item that unblocks the most others
- otherwise the lowest issue number in the milestone

State the pick and a one-line reason before you start. If the issue's "Done when" does not parse into a checkable bar, ask rather than guess.

## 3. Check the issue against reality before building

The body was written at a point in time. Confirm its premise still holds against `docs/spec/`, [9 — The decision register](docs/spec/09-decisions.md) and [13 — Open obligations](docs/spec/13-open-obligations.md). If it does not, say what changed and stop for confirmation rather than implement a stale ask.

## 4. Do the work

Follow `CLAUDE.md`. Branch, small commits, a pull request that says `Closes #N`. The bar is the issue's "Done when", not a reading of the title.

## 5. Write back

This is the part that compounds, and it is not optional. Before reporting:

- comment on the issue wherever it was wrong: interfaces that came out different, assumptions that failed, cost that surprised you
- edit the downstream issue bodies this work invalidated, the way #1 and #2 were revised when the decision register closed. Say in the edit what changed and why
- if the work contradicted or sharpened a closed decision, route the finding to [13 — Open obligations](docs/spec/13-open-obligations.md). Spec 9 is a register of settled decisions and accepts no new questions
- if the work supplied or killed an instrument for an unmeasured claim, update the `discharges:Qn` label and the matching entry in spec 13
- if the milestone order is now wrong, say so with the reason. Do not re-plan in silence

## 6. Report

Four lines: what shipped, the pull request, what the next iteration should pick and why, and what you learned that is written down nowhere yet. If that last line is empty, say so. It rarely is, and an empty answer usually means step 5 was rushed.

Do not start a second issue.
