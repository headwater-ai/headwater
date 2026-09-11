---
id: HW-DR-0061
status: current
status_since: 2026-09-11
summary: "A recorded terminal demonstration is a figure under HW-DR-0039. It may show only the tutorial's scratch corpus, where a blocking step already holds every printed output against a run."
last_verified: 2026-09-11
title: "Q61 — How a recorded terminal demonstration is held against a run"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  constrains:
    - HW-DR-0039
  traces_to:
    - docs/tutorials/your-first-governed-corpus.md
---

# Q61 — How a recorded terminal demonstration is held against a run

## Context

**[#601](https://github.com/headwater-ai/headwater/issues/601) asks what holds a recorded terminal demonstration true, before anybody records one.** A cast of `headwater check` is attractive on a README, because it shows the tool working rather than describing it. It is also a picture of numbers, and this corpus already rules on a picture of numbers.

**Principle 11 of [spec 0](../spec/00-vision-and-scope.md#design-principles) forbids a published number that no run produced, and [spec 4](../spec/04-assurance-model.md) extends it to the public surface.** A hand-written figure on the site is a finding there, in the way that a hand-edited shelf index is. [HW-DR-0039](0039-q39-how-a-figure-reaches-a-hand-built-page.md) is the clause that carries it into practice. A script writes every figure on a hand-built page from a run. A figure that no run produces stays off the page.

**A recording is a figure that no script can refresh.** `tools/refresh-figures.sh` writes a number into a `data-figure` element of a text file, and it compares what it wrote against the tree. It cannot reach inside a GIF or a cast file. A recorded number is frozen at the moment of capture, and no check reads it again.

**The numbers a cast of this corpus would freeze move in days, and this branch moved them itself.** On `origin/main` at `58f46a8d`, `headwater check --root .` printed 449 files under the corpus root. It typed and checked 318 of them, over 5757 check instances, and reported 166 findings at the default level. Adding the single document you are reading moved that to 450 seen and 319 classified over 5772 instances in one commit. Five days earlier the same command printed 397 seen and 285 checked. A cast is a claim that ages by the week.

**No recorder is on this developer host.** Measured on 2026-09-11 with `command -v`: `asciinema`, `agg`, `vhs`, `svg-term`, `termtosvg`, `ffmpeg`, `termsvg` and `gifsicle` are all absent, which is 0 of 8. `convert` and `magick` are present and neither records a terminal. This is a fact about one laptop. Continuous integration runs on GitHub's hosted images, because the repository variable `CI_RUNNER` is set to `["ubuntu-latest"]`. The self-hosted list inside `.github/workflows/ci.yml` is only the fallback for an unset variable. A hosted job can install a recorder, as the job at line 192 already installs `bubblewrap`.

## Decision

**A recorded terminal demonstration is a figure under principle 11.** It may show no output that a live run of the same command would move. This is HW-DR-0039 applied to a new medium rather than a new rule. The record constrains that decision, so the next reader of the figure clause meets this one.

**The only surface a recording may show is the tutorial's scratch corpus.** `.claude/tutorial/drive.py` reads the commands out of `docs/tutorials/your-first-governed-corpus.md`. It runs each one against a scratch repository under a temporary directory. It then diffs each result against the block the document prints. `.claude/tutorial/fixtures.sh` runs that script, and `.github/workflows/ci.yml` line 1285 runs the fixture script as a blocking step. So every output a recording of those commands shows is already held against a run by a step that fails the build when it drifts.

**A recording of `headwater check` on this corpus is forbidden by the clause above.** It freezes 449, 5757 and 166, and no mechanism refreshes any of them. Confine the recording to the tutorial commands, in a scratch corpus small enough that its counts are stable by construction.

**The tool is `vhs` and the embed format is an animated GIF.** Two facts decide this pair. GitHub renders no asciinema player inside a README, so a `.cast` file needs a third-party page to play it. GitHub's image proxy blocks an animated third-party SVG, so the SVG output of a cast converter does not animate in a README either. An animated GIF is the one embed that renders. `vhs` is the recorder, because it reads a checked-in tape file, which is a source that a later run replays. A recorder driven by a human at a keyboard produces an artifact nothing can reproduce.

**The question this record does not answer stays with the product owner.** Whether principle 11 may be relaxed for a marketing asset is not decided here, and nothing in this record relaxes it.

## Consequences

**Done-when bars 3 and 4 of #601 stay open.** No committed recording lands from this record, and no blocking step for one exists. `tools/refresh-figures.sh` structurally cannot refresh a committed binary asset, which [spec 13](../spec/13-open-obligations.md) now records.

**A recording of the tutorial inherits an existing gate rather than needing a new one.** The drive script already holds each printed output against a run. What it does not do is compare the recording against the tutorial. A cast that falls behind the document is invisible to every check. Whoever lands a recording owes that comparison, and a tape file is what makes one possible.

**The condition that reopens the owner's question is written here, where the next reader meets it.** A recording may carry a live number of this corpus when two things are true together. A recorder is installed on the runner that continuous integration uses. A regenerate-and-compare step for the recording runs beside `tools/refresh-figures.sh --check`. Until both hold, the answer above stands.
