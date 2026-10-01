---
id: HW-PD-0013
status: current
status_since: 2026-09-27
summary: "Only github.event_name decides whether a CI job may run on the self-hosted pool. A push is eligible on every branch, and a pull request never is."
last_verified: 2026-09-30
title: "Self-hosted eligibility in CI is decided by the event alone, and a push to any branch is eligible"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: reconstructed
relations:
  governs:
    - to: .github/workflows/ci.yml
      verified_revision: sha256:5f93740a98eb9b2dc3943e391f8c87246aff2a2e14bfd37e21aaf05e9cfe240b
---

# Self-hosted eligibility in CI is decided by the event alone, and a push to any branch is eligible

## Context

Before this record, the reasons lived in comments of `.github/workflows/ci.yml` on 2026-09-27. The comments were header rule 1 (lines 32 to 37) and the comment on `on:` (lines 81 to 84). [Two triggers, one verdict](../evaluations/two-triggers-one-verdict-how-this-repository-s-ci-decides-where-a-job-runs-and-what-it-trusts.md#the-only-boundary-is-the-event-and-it-is-the-only-one-that-can-be) states the design as a whole and holds the measurements. This record holds the decision and cites that evaluation for the evidence.

A pull request from a fork runs the fork's own copy of `ci.yml`. The fork can delete, invert or replace any condition in that copy. So no condition in this file can keep a fork off the self-hosted runner.

## Decision

**The event decides.** The `runs-on` expression of `engine` and of `headwater` reads `github.event_name` to decide whether the job may run on the self-hosted pool. The `runs-on` of `route` is the literal `ubuntu-latest` ([HW-PD-0018](0018-a-router-sends-a-push-or-merge-group-run-to-a-hosted-runner-when-the-self-hosted-pool-is-full-and-it-can-only-take-work-away.md)). It reads no other field for that choice. It never reads which repository opened a pull request, or any field that the author of a pull request can change.

**Every branch.** `on.push.branches` is `['**', '!gh-readonly-queue/**']` and not `[main]`. The one exclusion is the merge queue's own branches, which run under `merge_group` instead ([HW-PD-0020](0020-merges-go-through-the-github-merge-queue-one-squash-commit-per-pull-request.md)). A person with commit access writes each push to a feature branch, and nobody reads it yet. That is the case the self-hosted runner exists to make fast. So a push is eligible on every branch.

**The router cannot grant.** The router of [HW-PD-0018](0018-a-router-sends-a-push-or-merge-group-run-to-a-hosted-runner-when-the-self-hosted-pool-is-full-and-it-can-only-take-work-away.md) can send an eligible job to a hosted runner. It never makes a job eligible.

## Consequences

A pull request always runs on a hosted runner. A push runs on the self-hosted pool only when `CI_RUNNER` names it ([HW-PD-0016](0016-ci-runner-is-an-opt-in-that-only-a-push-or-a-merge-group-reads-and-an-unset-value-falls-back-to-ubuntu-latest.md)) and the pool has space. Since [HW-PD-0020](0020-merges-go-through-the-github-merge-queue-one-squash-commit-per-pull-request.md), a `merge_group` run is eligible in the same way as a push, because only a person with write access can queue a pull request.

A `runs-on` that reads a field of a pull request is a defect, even when it looks safer. A fork removes that condition in its own copy, so the condition protects nothing and hides where the boundary is ([HW-PD-0014](0014-the-boundary-against-a-fork-is-the-approval-of-outside-runs-and-a-person-reads-a-github-diff-before-approving-one.md)).

This record reopens if GitHub starts to run the base repository's copy of a workflow for a pull request from a fork.
