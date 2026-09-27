---
id: HW-PD-0020
status: current
status_since: 2026-09-27
summary: "The integrator hands each ruled pull request to the GitHub merge queue. The queue tests up to five at once on the group tip, and lands one squash commit per pull request."
last_verified: 2026-09-27
title: "Merges go through the GitHub merge queue, one squash commit per pull request"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: evidenced
relations:
  governs:
    - to: .github/workflows/ci.yml
      verified_revision: sha256:3462e1570a42ce661d356e5e25f615343a6abe3c68a0ab81b382ffb5e0656837
    - to: tools/run/queue-done.sh
      verified_revision: sha256:e1d5b097918a1f58af5cdd9610b321f7fe949b0e7f5f2a408643605f2b4e709d
    - to: tools/run/queue-done-fixtures.sh
      verified_revision: sha256:c85c8ec0e3240a77e5e6ded53daad36b5d5fe005e2a29a30ef6144b0b548ad0e
---

# Merges go through the GitHub merge queue, one squash commit per pull request

## Context

On 2026-09-27 the owner gave this direction: "Make the necessary config changes then implement merge queue." The direction has three parts. Merges go through GitHub's merge queue. Each pull request lands as one squash commit. No commit carries several issues.

Before this record, the integrator merged one pull request at a time. The `Protect main` ruleset sets `strict_required_status_checks_policy` to `true`, so a pull request whose head did not contain the tip of `main` could not merge. Each merge therefore put every other ruled pull request behind `main`. The integrator merged `origin/main` into each branch by hand, rebuilt, regenerated, pushed, and waited for a full CI run again. With four rulings in the queue, the last one waited for four CI runs that tested nothing new about it.

Earlier in run `20260927-0443`, pull request #1211 tried a batch branch. It merged four ruled pull requests into one branch, regenerated once at the tip, and landed the four as one squash commit. That saved the CI runs. It also put four issues into one commit on `main`, and the owner ruled against that.

## Decision

**Every merge goes through the merge queue.** The integrator takes every pull request that the parent ruled MERGE. For each one, it runs `gh pr merge <PR> --squash --match-head-commit <sha>`, which adds the pull request to the queue. Then it waits with `sh tools/run/wait-for.sh 'sh tools/run/queue-done.sh <PR>'` until the pull request lands or the queue ejects it.

**The queue tests a group on the group tip, and it lands one squash commit for each pull request.** GitHub builds a temporary branch under `gh-readonly-queue/main/` from `main` and the queued pull requests, in queue order. CI runs on the `merge_group` event against the tip of that branch. When the two required checks pass, each pull request in the group lands as its own squash commit.

**An ejection is reported, and nothing enqueues it again.** The integrator reports an ejected pull request with the reason that `queue-done.sh` prints, which names the failing check or the conflict. A new attempt is a new ruling, and the parent makes it.

**A branch that is only behind `main` is not brought current by hand.** The queue tests the composition, so the strict up-to-date rule has no work left to do. The ruleset drops it. The integrator merges `origin/main` into a branch only when the queue reports a conflict. GitHub merges with no custom merge driver, so it can write a derived artifact wrong. That artifact fails the projection step of the `headwater` job on the group tip, and the queue ejects the pull request.

**The batch branch of #1211 is retired.** No integrator merges several pull requests into one branch again.

**The parent applies the ruleset after this record merges.** This record states the settings, and nothing in this change applies them. The `Protect main` ruleset gets one new rule, `merge_queue`, with these parameters:

| parameter | value |
|---|---|
| `merge_method` | `SQUASH` |
| `grouping_strategy` | `ALLGREEN` |
| `max_entries_to_build` | 5 |
| `min_entries_to_merge` | 1 |
| `min_entries_to_merge_wait_minutes` | 5 |
| `max_entries_to_merge` | 5 |
| `check_response_timeout_minutes` | 90 |

The `required_status_checks` rule stays as it is, with one change: `strict_required_status_checks_policy` becomes `false`. The two required contexts stay `Engine tests` and `headwater check (advisory)`.

## Consequences

`.github/workflows/ci.yml` runs on `merge_group` with the type `checks_requested`. Both required jobs run on that event under the same names, so the queue always gets the two checks that it waits for. A `merge_group` ref is built from branches of this repository that a person with write access queued. So the workflow trusts it as it trusts a `push`, and the job can use the self-hosted runner. The `route` job runs on `merge_group` too, and it measures the pool as it does for a `push`. No job is skipped on the new event. The queue branches are excluded from the `push` trigger, so one queued commit starts one run.

This record amends four others on one point each, and each of them now says so. [HW-PD-0013](0013-self-hosted-eligibility-in-ci-is-decided-by-the-event-alone-and-a-push-to-any-branch-is-eligible.md) and [HW-PD-0016](0016-ci-runner-is-an-opt-in-that-only-a-push-or-a-merge-group-reads-and-an-unset-value-falls-back-to-ubuntu-latest.md) named `push` as the only eligible event. [HW-PD-0018](0018-a-router-sends-a-push-or-merge-group-run-to-a-hosted-runner-when-the-self-hosted-pool-is-full-and-it-can-only-take-work-away.md) measured the pool only for a `push`. [HW-PD-0017](0017-ci-concurrency-is-per-ref-and-every-ref-but-main-cancels-a-superseded-run.md) called the run on `main` the only composition check.

The concurrency group is keyed on the ref. Each queue entry has its own ref, so the run of one entry never cancels the run of another entry.

Before the parent applies the ruleset, `gh pr merge --squash` merges at once, as it did before. Then `queue-done.sh` reports `merged` on its first call. So the integrator procedure is correct on both sides of the ruleset change, and a pull request merged the old way still lands.

The squash message comes from the repository's squash-merge settings when the queue merges, and not from flags on `gh pr merge`. The parent sets the squash title to the pull request title and the message to the pull request body when it applies the ruleset.

This record is reversible. To reverse it, delete the `merge_queue` rule, set `strict_required_status_checks_policy` to `true` again, and restore the procedure that brings a branch current by hand. This record opens again when the queue ejects more than one group in a week for a reason that is not in the change itself.
