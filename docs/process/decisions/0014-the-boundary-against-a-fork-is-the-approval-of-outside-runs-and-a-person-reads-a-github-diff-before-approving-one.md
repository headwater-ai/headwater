---
id: HW-PD-0014
status: current
status_since: 2026-09-27
summary: "No expression in ci.yml defends against a fork. The approval setting for outside contributors is the boundary, and the reader of a .github diff enforces it."
last_verified: 2026-09-28
title: "The boundary against a fork is the approval of outside runs, and a person reads a .github diff before approving one"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: reconstructed
relations:
  governs:
    - to: .github/workflows/ci.yml
      verified_revision: sha256:6ca215002e8df8dcebe41117d4c7cce4bc53f52bf44fc9083dfa09ace4d32460
---

# The boundary against a fork is the approval of outside runs, and a person reads a .github diff before approving one

## Context

Before this record, the reasons lived in comments of `.github/workflows/ci.yml` on 2026-09-27. The comments were header rule 2 (lines 39 to 42). [Two triggers, one verdict](../evaluations/two-triggers-one-verdict-how-this-repository-s-ci-decides-where-a-job-runs-and-what-it-trusts.md#the-only-boundary-is-the-event-and-it-is-the-only-one-that-can-be) holds the argument and the measurements.

A fork runs its own copy of `ci.yml` ([HW-PD-0013](0013-self-hosted-eligibility-in-ci-is-decided-by-the-event-alone-and-a-push-to-any-branch-is-eligible.md)). So the boundary against a hostile fork cannot be an expression in that file.

## Decision

**The setting is the boundary.** The repository setting `fork_pr_contributor_approval: all_external_contributors` holds every run from an outside contributor until a maintainer approves it. That setting keeps outside code off the runners, and no expression in `ci.yml` does.

**Read the diff first.** A maintainer does not approve a run whose diff touches `.github/` before they read that diff. No expression in `ci.yml` does this work for them.

## Consequences

The protection of the runners depends on one repository setting and on one review by a person. A change to the setting changes the boundary, and no file in this repository shows that change.

A condition added to `ci.yml` to stop a fork is not a second boundary. The fork deletes it in its own copy.

This record reopens if the approval setting changes, or if a second maintainer approves outside runs.
