---
id: HW-PD-0010
status: current
status_since: 2026-09-27
summary: "Each release workflow refuses a tag whose version differs from its artifact. It reads under pipefail and refuses an empty reading, because the default shell passed every tag."
last_verified: 2026-09-27
title: "Each release workflow compares the tag with the version its artifact states and refuses an empty reading"
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
    - to: .github/workflows/publish-crates.yml
      verified_revision: sha256:779cd2d5d7fdcfe330d8fa714fc6b753ca9a59a5c0b0af9ae710d21cc6687763
---

# Each release workflow compares the tag with the version its artifact states and refuses an empty reading

## Context

Before this record, the reason lived in three places. On 2026-09-27 they were these comments:

- `.github/workflows/release.yml`: the step "The binary prints the version the tag names" (lines 150 to 164).
- `.github/workflows/release-taxonomy.yml`: the header (lines 35 to 45) and the publish step (lines 102 to 107).
- `.github/workflows/publish-crates.yml`: the header (lines 17 to 19) and the publish step (lines 137 to 142).

A person can cut a tag before the version bump that it was to carry, or against the wrong commit. A checkout of the wrong ref has the same result. The workflow then builds an artifact whose version is not the version that its tag names.

The first version guard of `release.yml` passed for every tag. GitHub runs a `run:` block on Linux with `bash -e {0}`, and not with `bash -eo pipefail {0}`. So `$(binary --version | awk ...)` takes the exit status of `awk`. A binary that does not run leaves the reading empty at exit 0. An empty reading then makes the pattern `"$printed"*` into `*`, which matches every tag.

## Decision

Each release workflow compares the tag with the version that its own artifact states, and stops the run when the two differ.

| workflow | what it reads |
|---|---|
| `release.yml` | the version that the built binary prints with `--version`, on the host that built it, and again in each smoke job |
| `release-taxonomy.yml` | the `version` that `taxonomy publish` wrote into the release record it produced |
| `publish-crates.yml` | the version of `headwater-cli` that `cargo metadata` reads from the workspace |

Each comparison reads its value under `set -eo pipefail`. Where an empty reading could match any tag, the step refuses the empty reading by name. A check that cannot run is not a check that passes.

## Consequences

A tag that does not agree with the version in the tree stops each release before it creates a release or publishes a crate. [Cut a release](../../how-to/cut-a-release.md) tells a maintainer to bump the version first, for this reason.

A container job does not get `bash` by default. Inside a container, GitHub runs `sh -e {0}`, which is `dash` on Debian and refuses `set -o pipefail`. So the musl smoke job of `release.yml` sets `bash` as its shell.

No pull request runs these steps ([HW-PD-0012](0012-the-release-workflows-are-separate-from-ci-one-tag-value-serves-both-entry-points-and-fixtures-read-them-statically.md)). A guard that is wrong in a new way shows itself only at a release.
