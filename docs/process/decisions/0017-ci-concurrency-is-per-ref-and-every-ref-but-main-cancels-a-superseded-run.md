---
id: HW-PD-0017
status: current
status_since: 2026-09-27
summary: "A newer push cancels the older run on the same branch or pull request. Runs on main never cancel, because each merge needs its own verdict."
last_verified: 2026-09-27
title: "CI concurrency is per ref, and every ref but main cancels a superseded run"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: reconstructed
relations:
  governs:
    - to: .github/workflows/ci.yml
      verified_revision: sha256:f4b3c176cedc9d969614ad7755edc324f1dc78423d8b55162ab05beb8595f57e
---

# CI concurrency is per ref, and every ref but main cancels a superseded run

## Context

Before this record, the reasons lived in comments of `.github/workflows/ci.yml` on 2026-09-27. The comments were the comment on `concurrency` (lines 89 to 107) and the block itself (lines 108 to 110). That comment recorded one measurement. Three pushes to one branch in ten minutes queued three full runs of both jobs, at three to fourteen minutes each.

A pool of three self-hosted slots serves this repository, two until 2026-09-23. A run is two jobs. So a run that nobody reads holds most of the pool, and every other branch waits behind it. On a hosted runner the cost is the minutes and not the queue.

## Decision

**Per ref.** The group is `${{ github.workflow }}-${{ github.ref }}`. `github.ref` is `refs/pull/N/merge` for a pull request and `refs/heads/<branch>` for a push. So two pull requests never cancel each other, and no second key is necessary.

**Cancel on every ref but `main`.** `cancel-in-progress` is true for every ref except `refs/heads/main`. A newer run on a branch cancels the older run, because nobody merges the superseded commit.

**`main` keeps its runs.** A run on `main` is the last check that reads the composition of merges. Since [HW-PD-0020](0020-merges-go-through-the-github-merge-queue-one-squash-commit-per-pull-request.md), the merge queue reads it first, on the group tip. Two merges that land a minute apart still need two verdicts.

**Key on the ref and never on the event.** A key on the event once exempted exactly the push runs that hold the self-hosted runner.

## Consequences

A branch that gets several pushes quickly holds at most one run's share of the pool. On `main`, a second run waits in its group for the first to finish.

This record reopens if the pool is large enough that a superseded run delays no other branch.
