---
id: HW-OBL-0183
status: current
status_since: 2026-09-11
summary: "Every figure check of this repository reads text. A number frozen inside a committed GIF or cast is held by nothing, and the recording bars of #601 stay open."
last_verified: 2026-09-11
title: "No mechanism holds a committed binary asset against a run"
waiting_on: build
relations:
  traces_to:
    - HW-DR-0061
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
---

# No mechanism holds a committed binary asset against a run

## Context

[HW-DR-0061](../decisions/0061-q61-how-a-recorded-terminal-demonstration-is-held-against-a-run.md) rules that a recorded terminal demonstration is a figure under principle 11, so it may show no output that a live run would move. The ruling is enforceable only where something compares the artifact against a run.

`tools/refresh-figures.sh` is the comparison HW-DR-0039 built, and it reads text. It writes a measured number into a `data-figure` element of an HTML file, and `--check` compares what it would write against what the tree carries. Nothing in that path opens a GIF, a cast file or a PNG. So a number frozen inside a binary asset is invisible to the one instrument that holds a figure to a run.

## Obligation

State what holds a committed binary asset against a run, and build it. Two bars of [#601](https://github.com/headwater-ai/headwater/issues/601) wait on this: a blocking step that regenerates the recording, and the committed recording in the README. Neither can land while the comparison does not exist.

The shape is a regenerate and a compare, in the way `--check` already works for a text figure. Replay a recorded tape against the tutorial's scratch corpus. Fail the build when the result is not the committed bytes. A recorder must be on the runner for that step to run at all. The repository variable `CI_RUNNER` sends jobs to GitHub's hosted images, which install one on request.

## Discharge

This record discharges when a step of `.github/workflows/ci.yml` regenerates a committed binary asset, compares it against the tree, and fails the build on a difference. Until then, no recording may carry a number of this corpus, and HW-DR-0061 confines a recording to the tutorial's scratch corpus for that reason.
