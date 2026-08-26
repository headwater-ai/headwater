---
id: HW-OBL-0145
status: draft
status_since: 2026-08-26
last_verified: 2026-08-26
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

An agent building issue #314 read a pull request's CI result and checked which of two jobs ran the shell suites. The second job, defined at .github/workflows/ci.yml:80, carries the name headwater check (advisory). The build order's recorded environment notes take that name at face value. They record that when main went red at commit 01861df, the advisory headwater check job stayed green. They give that as the reason a red baseline reads as hard to interpret. The name tells a reader the job is not signal, and eleven of its sixteen steps say otherwise.

## Obligation

The job carries sixteen steps. Three are setup: checkout, the toolchain, and cargo build. Two of the working steps carry an advisory posture in their own comments. One is named `Check this corpus` and the other `The change this pull request carries`. No step anywhere in the file has continue-on-error. Every step, advisory ones included, fails the job when its command exits non-zero. So advisory here means only that those two steps exit zero in the presence of findings, not that the job tolerates their failure.

- The commit gate fixture suite, `sh .githooks/fixtures.sh` at line 111, covers seventeen cases over the gate and its change-manifest producer.
- The lock check at line 112 runs `taxonomy resolve --check` against the committed lock.
- The projection check at line 118 runs `generate --check` against every generated file.
- The harness hook fixture suite, `sh .claude/hooks/fixtures.sh` at line 303, covers all four hook positions.
- The skills fixture suite, `sh .claude/skills/fixtures.sh` at line 320, covers forty-nine cases.
- The tutorial fixture suite, `sh .claude/tutorial/fixtures.sh` at line 336, covers fifty-nine claims.

The workflow's own header comment gets closer than the job name does, and is still wrong. It says only one of the second job's steps is advisory, but two steps carry that posture, each in its own comment. Only the job's name and its pass or fail color reach the pull request page.

## Discharge

What closes this is a rename inside .github/workflows/ci.yml, not a policy change. The job's name should stop describing the whole job as advisory, so a reader can tell from the name alone that it blocks. The two steps that are advisory already say so in place, so nothing is lost by dropping the word from the job name. The header comment's count should read two, not one. A grep for advisory in the workflow file should then return only the header comment and the two advisory steps, never the job name. None of this moves continue-on-error, --strict, or any run line, so no step gains or loses its blocking posture. The fix itself is mechanical. What it waits on is a reader outside this repository, the same precondition every adopter-waiting record on this shelf waits behind.
