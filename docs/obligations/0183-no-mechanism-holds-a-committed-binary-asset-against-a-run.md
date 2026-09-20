---
id: HW-OBL-0183
status: current
status_since: 2026-09-11
summary: "The recording thread of this record is narrowed by HW-DR-0078: disclosure, not a compare, closes #601's binary asset. What holds a DIFFERENT committed binary asset against a run, one with no ruling of its own, is still nothing."
last_verified: 2026-09-20
title: "No mechanism holds a committed binary asset against a run"
waiting_on: build
relations:
  traces_to:
    - HW-DR-0061
    - HW-DR-0078
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
---

# No mechanism holds a committed binary asset against a run

## Context

[HW-DR-0061](../decisions/0061-q61-how-a-recorded-terminal-demonstration-is-held-against-a-run.md) ruled that a recorded terminal demonstration is a figure under principle 11, so it may show no output that a live run would move. That ruling was enforceable only where something compares the artifact against a run.

`tools/site/refresh-figures.sh` is the comparison HW-DR-0039 built, and it reads text. It writes a measured number into a `data-figure` element of an HTML file, and `--check` compares what it would write against what the tree carries. Nothing in that path opens a GIF, a cast file or a PNG. A number frozen inside a binary asset is invisible to the one instrument that holds a figure to a run.

[HW-DR-0078](../decisions/0078-a-recorded-terminal-demonstration-may-show-a-frozen-number-behind-a-recorded-on-date-marker.md) answered the #601 half of this record a different way than the shape below assumed. The owner chose a visible `recorded-on-<date>` marker over a regenerate-and-compare step, for the one asset #601 names. A text check now holds the marker's presence, and nothing holds the frozen number itself.

## Obligation

**For #601's asset, this obligation is narrowed rather than discharged as first framed.** The comparison this record asked for is not what closes the recording bars of #601 now. HW-DR-0078 accepts the frozen number as a deviation, disclosed rather than checked. A step that regenerates and compares the recording is not owed for that one asset.

**The general question stays open, and it is what this record now tracks.** Nothing in this corpus states what holds a committed binary asset against a run, where that asset carries no ruling of its own. A second recording, a screenshot, or any other binary a future issue proposes meets the same gap HW-DR-0061 first measured. It meets that gap unless it too receives a ruling like HW-DR-0078's, or the comparison this record describes gets built.

The shape a compare-and-regenerate mechanism would take is unchanged from the original obligation. Replay a recorded artifact against a scratch corpus, in the way `--check` already works for a text figure. Fail the build when the result is not the committed bytes. A recorder must be on the runner for that step to run at all. The repository variable `CI_RUNNER` sends jobs to GitHub's hosted images, which install one on request.

## Discharge

**The #601-tied thread closes here.** A front-page recording's frozen number is held by disclosure under HW-DR-0078. The blocking text check for its marker is the mechanism that ruling asks for.

**The general record stays open.** It discharges under either of two conditions. One: a step of `.github/workflows/ci.yml` regenerates a committed binary asset and compares it against the tree, failing the build on a difference. Two: every committed binary asset in this corpus carries a ruling of its own, the way #601's recording now does. Until one of those holds, a new committed binary asset with no ruling meets the same gap this record first measured.
