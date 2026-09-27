---
id: HW-PD-0018
status: current
status_since: 2026-09-27
summary: "The route job counts the jobs on the self-hosted label and outputs overflow. A failure in it gives an empty output and changes no routing."
last_verified: 2026-09-27
title: "A router sends a push or merge group run to a hosted runner when the self-hosted pool is full, and it can only take work away"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: reconstructed
relations:
  governs:
    - to: .github/workflows/ci.yml
      verified_revision: sha256:9a6951a6d4a23cf366cf6046f17e651868521231af26008a8dec2927b10b0f59
---

# A router sends a push or merge group run to a hosted runner when the self-hosted pool is full, and it can only take work away

## Context

Before this record, the reasons lived in comments of `.github/workflows/ci.yml` on 2026-09-27. The comments were header rule 6 (lines 55 to 66), the comments on the `route` job (lines 116 to 194), and the `CI_SELF_HOSTED_SLOTS` commands (lines 76 and 77). [Two triggers, one verdict](../../evaluations/two-triggers-one-verdict-how-this-repository-s-ci-decides-where-a-job-runs-and-what-it-trusts.md#a-full-pool-sends-a-push-run-to-a-hosted-runner-and-the-router-can-only-take-work-away) holds the measurements.

The comment on the router recorded them in short. A self-hosted job is faster than a hosted job by about 2.5 minutes for each of the two jobs. A queued job waits 3 to 7 minutes for a slot to free. So a run that cannot start both jobs at once finishes sooner on a hosted runner.

## Decision

**What the router does.** The `route` job runs on `ubuntu-latest`, because it cannot wait in the queue of the pool that it measures. For a `push`, it counts the queued and running jobs that ask for the `headwater` label. When no job waits, and the running jobs plus the two jobs of this run fit in `CI_SELF_HOSTED_SLOTS`, it outputs `overflow=false`. Otherwise it outputs `overflow=true`, and `runs-on` sends both jobs to `ubuntu-latest`. The default pool size is 3.

**A `merge_group` run is measured as a push is.** Since [HW-PD-0020](0020-merges-go-through-the-github-merge-queue-one-squash-commit-per-pull-request.md), the router treats a `merge_group` run as it treats a `push`.

**One word and one comparison.** The router outputs one word. `runs-on` compares it with the literal `'true'`. So a job can get only `vars.CI_RUNNER` or `ubuntu-latest` ([HW-PD-0016](0016-ci-runner-is-an-opt-in-that-only-a-push-or-a-merge-group-reads-and-an-unset-value-falls-back-to-ubuntu-latest.md)). The router can take the self-hosted pool away from a job, and it never grants the pool ([HW-PD-0015](0015-a-condition-in-ci-may-take-work-away-and-never-grant-it-so-one-run-per-commit-comes-from-a-job-level-if.md)).

**The router never fails a run.** Its step continues on an error and stops after two minutes, so an error gives an empty output. The `if:` of `engine` and `headwater` opens with `!cancelled()`, so both jobs run when `route` fails as a job. An empty output routes the run by `CI_RUNNER` alone. `route` has no `if:` of its own.

## Consequences

A failed or skipped `needs:` skips a job by default, and a skipped job reports success to a required check. So removing `!cancelled()`, or giving `route` an `if:`, can turn a failure into a green check. Both are defects under this record.

Two routers that read the pool at the same moment can both choose it. That costs one wait for a slot.

`gh variable set CI_SELF_HOSTED_SLOTS --body 0` sends every push run to a hosted runner, with no commit.
