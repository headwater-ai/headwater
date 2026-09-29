---
id: HW-OBL-0145
status: current
status_since: 2026-09-06
last_verified: 2026-09-21
title: "The CI job named \\\"(advisory)\\\" carries every blocking shell suite in the repository"
summary: "The CI job named advisory blocks on eleven of its own steps, not the two its name describes."
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: draft
  accepted_by: j.baxter
  evidence_basis: evidenced
waiting_on: adopter
---

# The CI job named \"(advisory)\" carries every blocking shell suite in the repository

## Context

An agent building issue #314 read a pull request's CI result and checked which of two jobs ran the shell suites. The second job is the one keyed `headwater` in the `jobs` block of .github/workflows/ci.yml, and it carries the name headwater check (advisory). The build order's recorded environment notes take that name at face value. They record that when main went red at commit 01861df, the advisory headwater check job stayed green. They give that as the reason a red baseline reads as hard to interpret. The name tells a reader the job is not signal, and 48 of its 55 steps say otherwise.

This record named the job by a line number until 2026-09-21, and by then the job had moved from line 114 to line 426. It names the job by its key now, and it names each suite below by the path of the script that runs it. A line number in this file goes stale on every edit above it. Each of the counts in the next section had gone stale the same way.

## Obligation

The job carries 55 steps. Five are setup: the checkout, the cargo cache, the pip cache, the toolchain, and the engine build. Two of the working steps carry an advisory posture in their own comments. One is named `Check this corpus` and the other `The change this branch carries`. No step anywhere in the file has continue-on-error. Every step, advisory ones included, fails the job when its command exits non-zero. So advisory here means only that those two steps exit zero in the presence of findings, not that the job tolerates their failure.

- The commit gate fixture suite, `sh .githooks/fixtures.sh`, covers 106 cases over the gate and its change-manifest producer.
- The lock check runs `taxonomy resolve --check` against the committed lock.
- The projection check runs `generate --check` against every generated file.
- The harness hook fixture suite, `sh .claude/hooks/fixtures.sh`, covers 129 cases over all four hook positions.
- The skills fixture suite, `sh .claude/skills/fixtures.sh`, covers 67 cases.
- The tutorial fixture suite, `sh .claude/tutorial/fixtures.sh`, covers 66 claims.

Every count above is a measurement of 2026-09-21, taken by running each suite. The job held 16 steps when this record was first written, and the four suites then reported 17, four positions, 49 and 59. The direction of that drift is the point: each reading since has made the gap between the name and the job wider, never narrower.

The workflow's own header comment gets closer than the job name does, and is still wrong. It says only one of the second job's steps is advisory, but two steps carry that posture, each in its own comment. Only the job's name and its pass or fail color reach the pull request page.

## Discharge

What closes this is a rename inside .github/workflows/ci.yml, not a policy change. The job's name should stop describing the whole job as advisory, so a reader can tell from the name alone that it blocks. The two steps that are advisory already say so in place, so nothing is lost by dropping the word from the job name. The header comment's count should read two, not one. A grep for advisory in the workflow file should then return only the header comment and the two advisory steps, never the job name. The new name has to be a literal. An expression-valued job name reaches the pull request page as its own source text, on any run that skips the job. A change on 2026-09-21 measured that and undid it. None of this moves continue-on-error, --strict, or any run line, so no step gains or loses its blocking posture. The fix itself is mechanical. What it waits on is a reader outside this repository, the same precondition every adopter-waiting record on this shelf waits behind.
