---
id: HW-PD-0016
status: current
status_since: 2026-09-27
summary: "The self-hosted labels reach runs-on only when the CI_RUNNER variable names them and the event is a push or a merge group. The default is a hosted runner."
last_verified: 2026-09-30
title: "CI_RUNNER is an opt-in that only a push or a merge group reads, and an unset value falls back to ubuntu-latest"
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

# CI_RUNNER is an opt-in that only a push or a merge group reads, and an unset value falls back to ubuntu-latest

## Context

Before this record, the reasons lived in comments of `.github/workflows/ci.yml` on 2026-09-27. The comments were on the `runs-on` of the `engine` job (lines 213 to 222) and at the end of the header (lines 73 to 77). The `runs-on` expressions are at lines 223 and 493. [Two triggers, one verdict](../evaluations/two-triggers-one-verdict-how-this-repository-s-ci-decides-where-a-job-runs-and-what-it-trusts.md#the-only-boundary-is-the-event-and-it-is-the-only-one-that-can-be) holds the argument.

The variable lives in the repository settings and not in the tree. So a reader of the tree cannot see its value, and a document that states the value goes stale when a person changes it.

## Decision

**An opt-in.** `runs-on` reads `vars.CI_RUNNER` only when the event is a `push` and the router did not report overflow ([HW-PD-0018](0018-a-router-sends-a-push-or-merge-group-run-to-a-hosted-runner-when-the-self-hosted-pool-is-full-and-it-can-only-take-work-away.md)). Since [HW-PD-0020](0020-merges-go-through-the-github-merge-queue-one-squash-commit-per-pull-request.md), a `merge_group` run reads it in the same way as a `push`. When the variable is unset, the expression falls back to `["ubuntu-latest"]`. The self-hosted labels reach a job only when the variable names them, with `gh variable set CI_RUNNER --body '["self-hosted", "headwater"]'`.

**A pull request never reads it.** A `pull_request` run takes the last branch of the expression, `ubuntu-latest`, whatever the variable holds.

**Read the live value.** Run `gh variable list` before you state which runner a push uses.

## Consequences

`gh variable delete CI_RUNNER` sends every run to a hosted runner, with no commit. A statement that the self-hosted labels are the default is wrong.

A pull request that is `CONFLICTING` against a moved `main` gets no check suite on either runner. From outside, that looks the same as a dead runner. So read `mergeable` before you read a missing check suite as a runner fault.
