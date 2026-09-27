---
id: HW-DR-0091
status: current
status_since: 2026-09-27
summary: "release.yml names each runner image and never reads CI_RUNNER, because the image sets the glibc floor that README.md states. The build is --locked."
last_verified: 2026-09-27
title: "The runner image of the engine release build is written in the workflow because it sets the glibc floor"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: reconstructed
relations:
  governs:
    - to: .github/workflows/release.yml
      verified_revision: sha256:e2004f70a412b80dbc2ba92887f1a694973f8e21905114badd18a548ce4b4296
    - to: .github/workflows/release-taxonomy.yml
      verified_revision: sha256:cc62f8db1425466922c618475983155ec51572a77c846a4634a730e18a804ae4
---

# The runner image of the engine release build is written in the workflow because it sets the glibc floor

## Context

Before this record, the reason lived only in comments of `.github/workflows/release.yml` on 2026-09-27. They were the header (lines 44 to 53), the build matrix (lines 104 to 109) and the build step (lines 145 to 147). The build step of `.github/workflows/release-taxonomy.yml` (lines 89 to 91) cited the same reason for `--locked`.

`.github/workflows/ci.yml` reads its runner from the `CI_RUNNER` repository variable. That is correct for a verdict about a commit, because a move between the self-hosted runner and a hosted image then costs no commit. A person can change that variable with no commit and no review.

The glibc archive of an engine release needs the glibc version of the image that built it. `README.md` states that floor to an adopter as a fact about the download. The archive is built on `ubuntu-24.04`, so it needs glibc 2.39 or later.

## Decision

Each row of the build matrix in `release.yml` names its runner image in the workflow file. No job of `release.yml` reads `CI_RUNNER` or any other variable for its runner. A change to an image is a diff in `release.yml` that a reviewer reads.

The release build runs `cargo build --release` with `--locked`. The binary that an adopter downloads resolves the committed `engine/Cargo.lock`, and not the dependency versions that resolve on the day of the build. The taxonomy release builds its engine with `--locked` for the same reason. `tools/engine/build-declaration-fixtures.sh` holds the flag on the cargo commands of `ci.yml`. On 2026-09-27 it did not read the release workflows, so the flag there is held by review alone.

## Consequences

The glibc floor of the archive moves only with a reviewed commit. A reviewer of that commit can then check the sentence in `README.md` that states the floor. No fixture compares the image with that sentence.

A move of CI between runners does not move the floor of the release archives.

The musl archive needs no C library, so its floor does not depend on the image. The smoke job for the musl archive runs it on `debian:bullseye`, which carries glibc 2.31, to show this.
