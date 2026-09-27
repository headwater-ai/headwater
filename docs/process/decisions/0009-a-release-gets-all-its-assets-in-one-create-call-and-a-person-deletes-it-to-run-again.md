---
id: HW-PD-0009
status: current
status_since: 2026-09-27
summary: "A release is immutable once gh release create returns, so every asset goes into that one call. A second run refuses, and a person deletes the release to retry."
last_verified: 2026-09-27
title: "A release gets all its assets in one create call, and a person deletes it to run again"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: evidenced
relations:
  governs:
    - to: .github/workflows/release.yml
      verified_revision: sha256:e2004f70a412b80dbc2ba92887f1a694973f8e21905114badd18a548ce4b4296
    - to: .github/workflows/release-taxonomy.yml
      verified_revision: sha256:cc62f8db1425466922c618475983155ec51572a77c846a4634a730e18a804ae4
---

# A release gets all its assets in one create call, and a person deletes it to run again

## Context

Before this record, the reason lived only in two step comments on 2026-09-27. One was on the step that attaches every archive in `.github/workflows/release.yml` (lines 372 to 389). The other was on the step that attaches the archive in `.github/workflows/release-taxonomy.yml` (lines 139 to 143).

A pushed tag has no release object until something creates one, and `gh release upload` refuses a tag that has none. The engine release workflow once made two calls: `gh release create`, and then `gh release upload` on the next line. At `v0.1.1`, GitHub made the release immutable when `create` returned. So the upload failed with "Cannot upload assets to an immutable release". This was not a race. It failed each time, and `README.md` records that `v0.1.1` carries no archive.

## Decision

Each release workflow passes every asset to the one `gh release create` call that makes the release. The assets land before the release becomes immutable. No step calls `gh release upload`, and no step passes `--clobber`.

Before that call, the step asks whether the tag already has a release. If it has one, the step stops with an error and names the recovery. It does not try again. The recovery is to delete the release, keep the tag, and run the workflow again. A person makes that decision, and no workflow step does.

The title of each release is its tag: both workflows pass `--title "$TAG"`. No file in this repository recorded why, and this record does not supply a reason. It states the rule as the workflows apply it, and the reason is not recorded.

## Consequences

A hand run of a release workflow against a tag that already has a release cannot repair that release. [Cut a release](../../how-to/cut-a-release.md) sends a maintainer to a hand run only for a tag that has no release.

A release that shipped a wrong asset stays wrong until a person deletes it. The tag stays when the release goes.

A future change that splits the assets across two calls fails at the second call, on the day of a release. No pull request runs these workflows ([HW-PD-0012](0012-the-release-workflows-are-separate-from-ci-one-tag-value-serves-both-entry-points-and-fixtures-read-them-statically.md)), so review is the guard.
