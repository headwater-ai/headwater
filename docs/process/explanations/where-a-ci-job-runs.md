---
id: HW-PEXP-where-a-ci-job-runs
status: current
status_since: 2026-09-27
summary: "A push or a merge queue run can use the self-hosted pool when CI_RUNNER opts in and the pool has room. A pull request from a fork runs on a hosted runner, and one from a branch here is skipped."
last_verified: 2026-09-27
title: "Where a CI job runs"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: reconstructed
relations:
  draws_on:
    - to: HW-PD-0013
    - to: HW-PD-0014
    - to: HW-PD-0015
    - to: HW-PD-0016
    - to: HW-PD-0017
    - to: HW-PD-0018
    - to: HW-PD-0019
---

# Where a CI job runs

## Scope

This page states how `.github/workflows/ci.yml` chooses a runner for each job of one run, and which runs it cancels. It is for a contributor who reads a CI result and wants to know where the job ran and why. It states no new rule. Each fact comes from one of the seven process decisions that it draws on, or from `ci.yml` itself. Where this page and `ci.yml` disagree, `ci.yml` is correct and this page is stale.

The runner containers are out of scope. They are provisioned outside this repository ([HW-PD-0019](../decisions/0019-ci-runs-actions-from-the-actions-organization-only-and-carries-no-build-state-between-self-hosted-jobs.md)).

## How it works

A run has three jobs: `route`, `engine` and `headwater`. The workflow starts on a push to any branch, on a pull request and on a merge queue run (`merge_group`). A push to a `gh-readonly-queue/` branch of the merge queue starts no run.

1. **`route` measures the pool.** It always runs on `ubuntu-latest`. For any other event, it outputs `overflow=true`. For a push or a `merge_group` run, it counts the queued and running jobs that ask for the `headwater` label. When no job waits, and the running jobs and the two jobs of this run fit in `CI_SELF_HOSTED_SLOTS`, it outputs `overflow=false`. Otherwise it outputs `overflow=true`. The default pool size is 3. Two errors are possible. When the script sees an error, it outputs `overflow=false`. A failed API call and a `CI_SELF_HOSTED_SLOTS` that is not a count are errors of this type. When the step fails in a way that the script cannot see, or passes its limit of two minutes, the output is empty. In both cases the run routes by `CI_RUNNER` alone. The router never fails the run ([HW-PD-0018](../decisions/0018-a-router-sends-a-push-or-merge-group-run-to-a-hosted-runner-when-the-self-hosted-pool-is-full-and-it-can-only-take-work-away.md)).
2. **The job-level `if:` removes a duplicate run.** `engine` and `headwater` skip the `pull_request` run for a branch of this repository, because the `push` run on the same commit already gives the result. A pull request from a fork keeps its run ([HW-PD-0015](../decisions/0015-a-condition-in-ci-may-take-work-away-and-never-grant-it-so-one-run-per-commit-comes-from-a-job-level-if.md)).
3. **`runs-on` chooses the runner.** The expression is the same for `engine` and `headwater`. A job gets the labels in `vars.CI_RUNNER` only when three conditions are true. The event is a `push` or a `merge_group`, the router did not output `overflow=true`, and `CI_RUNNER` is set. In every other case the job gets `ubuntu-latest` ([HW-PD-0013](../decisions/0013-self-hosted-eligibility-in-ci-is-decided-by-the-event-alone-and-a-push-to-any-branch-is-eligible.md), [HW-PD-0016](../decisions/0016-ci-runner-is-an-opt-in-that-only-a-push-or-a-merge-group-reads-and-an-unset-value-falls-back-to-ubuntu-latest.md)).
4. **A newer run cancels an older one on the same ref.** The concurrency group is the workflow and `github.ref`. A newer run cancels the older run on every ref except `refs/heads/main`. On `main`, every run completes ([HW-PD-0017](../decisions/0017-ci-concurrency-is-per-ref-and-every-ref-but-main-cancels-a-superseded-run.md)).

The table gives the result for each case.

| event | `CI_RUNNER` | router output | runner of `engine` and `headwater` |
|---|---|---|---|
| push to any branch, or `merge_group` | set | `overflow=false` or empty | the labels in `CI_RUNNER` |
| push to any branch, or `merge_group` | set | `overflow=true` | `ubuntu-latest` |
| push to any branch, or `merge_group` | not set | any | `ubuntu-latest` |
| pull request from a branch of this repository | any | any | skipped, and the push run answers |
| pull request from a fork | any | any | `ubuntu-latest`, after a maintainer approves the run |

A step that must act differently on the two runners reads `runner.environment`. No self-hosted job gets `engine/target` from an earlier job ([HW-PD-0019](../decisions/0019-ci-runs-actions-from-the-actions-organization-only-and-carries-no-build-state-between-self-hosted-jobs.md)).

To find which runner a push or a `merge_group` run uses today, run `gh variable list` and read `CI_RUNNER`.

## Why it is this way

Each record below holds the reasons for one part of the design. This page does not repeat them.

- Only the event decides eligibility, so no field that the author of a pull request can change reaches `runs-on` ([HW-PD-0013](../decisions/0013-self-hosted-eligibility-in-ci-is-decided-by-the-event-alone-and-a-push-to-any-branch-is-eligible.md)).
- The approval setting for outside contributors is the boundary against a fork, and no expression in `ci.yml` is ([HW-PD-0014](../decisions/0014-the-boundary-against-a-fork-is-the-approval-of-outside-runs-and-a-person-reads-a-github-diff-before-approving-one.md)).
- A condition may take work away and never grant it ([HW-PD-0015](../decisions/0015-a-condition-in-ci-may-take-work-away-and-never-grant-it-so-one-run-per-commit-comes-from-a-job-level-if.md)).
- The self-hosted pool is an opt-in, and the default is a hosted runner ([HW-PD-0016](../decisions/0016-ci-runner-is-an-opt-in-that-only-a-push-or-a-merge-group-reads-and-an-unset-value-falls-back-to-ubuntu-latest.md)).
- Runs on `main` never cancel, because each merge needs its own result ([HW-PD-0017](../decisions/0017-ci-concurrency-is-per-ref-and-every-ref-but-main-cancels-a-superseded-run.md)).
- The router can send a job away from a full pool and never fails a run ([HW-PD-0018](../decisions/0018-a-router-sends-a-push-or-merge-group-run-to-a-hosted-runner-when-the-self-hosted-pool-is-full-and-it-can-only-take-work-away.md)).
- Only actions from the `actions` organization run, and no job carries build state to the next ([HW-PD-0019](../decisions/0019-ci-runs-actions-from-the-actions-organization-only-and-carries-no-build-state-between-self-hosted-jobs.md)).

When one of these records is superseded or withdrawn, `headwater check` reports this page through its `draws_on` edge, and this page must change with it.
