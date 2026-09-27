---
id: HW-DR-0089
status: current
status_since: 2026-09-27
summary: "The taxonomy release runs taxonomy publish on taxonomy-source/ at the tag and never zips .headwater/packages/, so the artifact and the tag make one claim."
last_verified: 2026-09-27
title: "A taxonomy release publishes the source at the tagged commit and never ships the vendored copy"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: reconstructed
relations:
  governs:
    - to: .github/workflows/release-taxonomy.yml
      verified_revision: sha256:cc62f8db1425466922c618475983155ec51572a77c846a4634a730e18a804ae4
---

# A taxonomy release publishes the source at the tagged commit and never ships the vendored copy

## Context

Before this record, the reason lived only in the header comment of `.github/workflows/release-taxonomy.yml` (lines 21 to 33 on 2026-09-27).

This repository keeps two copies of `headwater/standard`. `taxonomy-source/headwater-standard` is the source. `.headwater/packages/headwater-standard` is the output of the last `taxonomy publish` that a maintainer ran by hand, vendored and committed next to the source. The header of `.headwater/packages/headwater-standard/package.yml` gives that four-step loop.

A release workflow could attach a zip of the vendored copy. That zip tells a reader when a maintainer last ran the second step of the loop. It does not tell a reader what the tagged commit publishes.

## Decision

The taxonomy release builds the engine at the tagged commit and runs `headwater taxonomy publish` against `taxonomy-source/headwater-standard` at that commit. It attaches the output of that run. It never attaches a copy of `.headwater/packages/headwater-standard`.

The digest in the release notes is the digest that this run printed. It is not a digest carried over from the last time a maintainer regenerated `.headwater/packages/`.

## Consequences

The tag and the artifact make the same claim: the artifact is what the source at this commit publishes. An adopter who runs `headwater taxonomy vendor --expect <digest>` checks the artifact in the same way that the release workflow produced it.

A vendored copy that is stale at the tagged commit does not reach an adopter through the release. It stays a defect of the loop inside this repository.

The release workflow needs a build of the engine, so a taxonomy release costs one release-profile build. The build uses `--locked` for the reason [HW-DR-0091](0091-the-runner-image-of-the-engine-release-build-is-written-in-the-workflow-because-it-sets-the-glibc-floor.md) states.
