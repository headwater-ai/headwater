---
id: HW-PD-0015
status: current
status_since: 2026-09-27
summary: "The job-level if skips a duplicate pull_request run and grants nothing. No trigger filter replaces it, and each job name stays a literal."
last_verified: 2026-09-27
title: "A condition in CI may take work away and never grant it, so one run per commit comes from a job-level if"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: reconstructed
relations:
  governs:
    - to: .github/workflows/ci.yml
      verified_revision: sha256:9a388898944b9ec3a6af4f28b10c5d8d069b21bc1cb81b0e04bba1cbdf6cfb57
---

# A condition in CI may take work away and never grant it, so one run per commit comes from a job-level if

## Context

Before this record, the reasons lived in comments of `.github/workflows/ci.yml` on 2026-09-27. The comments were header rules 3 and 4 (lines 44 to 50) and the comments on the `if:` of the `engine` job (lines 196 to 207). [Two triggers, one verdict](../../evaluations/two-triggers-one-verdict-how-this-repository-s-ci-decides-where-a-job-runs-and-what-it-trusts.md#a-condition-that-only-takes-work-away-is-safe-and-that-is-why-one-run-per-commit-is-safe) holds the argument, and [Two triggers, one verdict](../../evaluations/two-triggers-one-verdict-how-this-repository-s-ci-decides-where-a-job-runs-and-what-it-trusts.md#two-check-runs-of-one-name-and-what-settles-between-them) holds the measurement of two check runs on one commit.

A push to a branch of this repository that has an open pull request starts two runs on one commit. One is the `push` run and one is the `pull_request` run.

## Decision

**Take away, never grant.** A condition in `ci.yml` may skip work. It never gives a job a runner or an event that the job does not have without the condition.

**One run per commit.** The job-level `if:` of `engine` and `headwater` skips the `pull_request` run for a branch of this repository. The `push` run on the same commit already answers. A pull request from a fork keeps its run. When the `if:` is deleted, a fork gets the run that it had before. That is why the `if:` may read a field of a pull request and `runs-on` may not ([HW-PD-0013](0013-self-hosted-eligibility-in-ci-is-decided-by-the-event-alone-and-a-push-to-any-branch-is-eligible.md)).

**No trigger filter.** The `pull_request` trigger stays without a filter. A skipped workflow leaves a required check pending forever.

**Literal job names.** Each job `name:` is a literal. GitHub does not evaluate an expression in the name of a job that it skips, and it shows the source text instead.

## Consequences

A pull request from a branch of this repository shows two check runs of each required name on one commit. GitHub reads the pair together, so a skipped run never hides a failed one.

A new condition that grants work, a filter on a trigger, or an expression in a job name is a defect under this record.
