---
id: HW-DR-0088
status: current
status_since: 2026-09-27
summary: "An engine tag matches v* and a taxonomy tag matches taxonomy/headwater-standard/v*, so neither release waits on the other and requires_engine is the only link."
last_verified: 2026-09-27
title: "An engine release and a taxonomy release have disjoint tag namespaces and are cut independently"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: reconstructed
relations:
  governs:
    - to: .github/workflows/release.yml
      verified_revision: sha256:66aeccf8d5dd9a9a73f5a2e93de8062823828f404e2f46253368cbcc296cb15f
    - to: .github/workflows/release-taxonomy.yml
      verified_revision: sha256:cc62f8db1425466922c618475983155ec51572a77c846a4634a730e18a804ae4
    - to: .github/workflows/publish-crates.yml
      verified_revision: sha256:ead2238fa32a3fc4d600d5238d40a072cc5371aee1409a9417fa3fa2d62fea26
---

# An engine release and a taxonomy release have disjoint tag namespaces and are cut independently

## Context

Before this record, the reason for two tag namespaces lived only in the header comment of `.github/workflows/release-taxonomy.yml` (lines 7 to 19 on 2026-09-27) and in one paragraph of `README.md`. [#757](https://github.com/headwater-ai/headwater/issues/757) found the gap, and [#760](https://github.com/headwater-ai/headwater/issues/760) named the shape of the fix.

An engine tag is cut when the engine changes. Before #760, the only route for a `headwater/standard` fix to reach an adopter was an engine release. So the package version that an engine tag carried stayed behind the source for as long as nobody cut the next engine release. The comment that this record replaces counted ten fixes to the package that waited between two engine releases.

## Decision

An engine release and a taxonomy release have two tag namespaces that no tag can match at the same time.

- `v*` starts `.github/workflows/release.yml` and `.github/workflows/publish-crates.yml`.
- `taxonomy/headwater-standard/v*` starts `.github/workflows/release-taxonomy.yml`.

A tag matches one pattern at most, because the prefixes are different. A maintainer cuts each kind of release independently. A taxonomy release needs no engine version bump, and an engine release needs no taxonomy release. The one link between them is `requires_engine` in `taxonomy-source/headwater-standard/package.yml`. When a package version raises that floor, the engine release that satisfies it comes first.

The taxonomy release has its own workflow file and is not a job in `release.yml`. The reason is the same as the reason `release.yml` is not a job in `ci.yml` ([HW-PD-0012](../process/decisions/0012-the-release-workflows-are-separate-from-ci-one-tag-value-serves-both-entry-points-and-fixtures-read-them-statically.md)). A shared file would also make an edit to the engine release job run the taxonomy steps.

## Consequences

An adopter can pin a fixed `headwater/standard` version as soon as its maintainer publishes it, with no engine release. `README.md` tells the adopter that an engine tag carries the package version that shipped with it. That version can be older than the newest one.

A tag in a third namespace starts no release workflow. `tools/repo/release-guide-fixtures.sh` holds the tag patterns of each release workflow against the table in [Cut a release](../how-to/cut-a-release.md). It reads in both directions.

A change to the tag pattern of any of the three workflows makes this record suspect, because this record governs each of them.
