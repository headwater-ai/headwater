---
id: HW-PD-0019
status: current
status_since: 2026-09-27
summary: "Only actions from GitHub's own organization run in ci.yml. No self-hosted job inherits engine/target from another, and sccache makes a cold target cheap."
last_verified: 2026-09-27
title: "CI runs actions from the actions organization only, and carries no build state between self-hosted jobs"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: reconstructed
relations:
  governs:
    - to: .github/workflows/ci.yml
      verified_revision: sha256:f43f09ae7167f09120459b0e332f3e8cdc6a32c6eb8aafaa003933e67368a904
---

# CI runs actions from the actions organization only, and carries no build state between self-hosted jobs

## Context

Before this record, the reasons lived in comments of `.github/workflows/ci.yml` on 2026-09-27. The comments were the paragraph on actions (lines 21 to 24), header rule 5 (lines 52 to 54), and the paragraph on the two runners (lines 68 to 71). [Two triggers, one verdict](../../evaluations/two-triggers-one-verdict-how-this-repository-s-ci-decides-where-a-job-runs-and-what-it-trusts.md#the-two-runners-cache-different-things-because-they-are-different-shapes) holds the account of what each runner caches.

## Decision

**Actions from the `actions` organization only.** A step uses an action only from the `actions` organization, which is GitHub's own. An action from any other source runs code that nobody here reviews, on every push, with a token in the environment. The bar is the organization, and not a count of actions.

**No carried build state.** No self-hosted job carries `engine/target` from an earlier job. Two attempts broke `actions/checkout` and then `.githooks/fixtures.sh`. sccache makes a cold target directory cheap.

**A step reads the runner, not a guess.** A step that must act differently on the two runners reads `runner.environment`, which is `self-hosted` on the container. Each `pip install --user` passes `--break-system-packages`, which the Python of the self-hosted image needs under PEP 668. pip accepts the flag where PEP 668 does not apply.

**The runner itself is not here.** The self-hosted containers are provisioned outside this repository. This record states nothing about their lifecycle, their threat model or their incident runbook.

## Consequences

An action from another source needs a record that supersedes this one.

A cache that carries `engine/target` between self-hosted jobs is a defect under this record, even when it makes one run faster.
