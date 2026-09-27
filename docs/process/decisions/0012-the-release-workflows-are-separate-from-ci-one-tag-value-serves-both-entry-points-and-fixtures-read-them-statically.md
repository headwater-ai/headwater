---
id: HW-PD-0012
status: current
status_since: 2026-09-27
summary: "Release workflows are files apart from ci.yml and read the tag from one step output. No pull request runs them, so fixtures read their text."
last_verified: 2026-09-27
title: "The release workflows are separate from CI, one tag value serves both entry points, and fixtures read them statically"
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
      verified_revision: sha256:ead2238fa32a3fc4d600d5238d40a072cc5371aee1409a9417fa3fa2d62fea26
---

# The release workflows are separate from CI, one tag value serves both entry points, and fixtures read them statically

## Context

Before this record, the reasons lived in comments of the three release workflows on 2026-09-27:

- `.github/workflows/release.yml`: the header (lines 3 to 35), the tag step (lines 83 to 85) and the build matrix (lines 98 to 102). Also the archive step (lines 178 to 182), the publish job (lines 330 to 340) and the step that attaches the archives (lines 396 to 399).
- `.github/workflows/release-taxonomy.yml`: the header (lines 15 to 19), the tag step (lines 69 to 70), and the archive step (lines 127 to 130).
- `.github/workflows/publish-crates.yml`: the header (lines 8 to 16), and the tag step (lines 87 to 90).

`ci.yml` runs on a push to `main` and on a pull request. Neither event starts a run for a tag. A tag trigger in `ci.yml` would run each corpus gate and each fixture suite again, on a commit that passed all of them on `main`.

## Decision

**Separate files.** Each release workflow is a file of its own and is not a job in `ci.yml`. The taxonomy release is also not a job in `release.yml`.

**Two entry points, one value.** A pushed tag starts each release workflow. `workflow_dispatch` also starts each one, with a required `tag` input that has no default. The first step resolves the tag from the input, or from the ref when there is no input, and writes it as a step output. Every later step reads that output. No step reads `github.ref_name`, so the two entry points cannot diverge.

**A dry run.** A hand run of `release.yml` with `publish: false` builds every archive, runs both smoke jobs, and creates no release. It is the only run that a change to `release.yml` gets before a tag. The run can use a branch and a tag that already exists. The job that creates the release asks for `contents: write` itself. The workflow default is `contents: read`, so a job added later must ask for the permission.

**Text that a fixture can read.** No pull request runs these files. So fixtures that run on every pull request read their text:

- Each asset name is written in full, on its matrix row and again as an operand of `gh release create`. Those operands use variable names that no other step assigns. No name is assembled from `matrix.target` or by shell concatenation. Group 7 of `tools/repo/readme-fixtures.sh` compares these names with the names in `README.md`.
- The step of `release.yml` that writes the release notes has a fixed name. `tools/repo/readme-fixtures.sh` finds the step by that name and runs it.
- `tools/repo/release-guide-fixtures.sh` reads the `on:` block of each workflow against [Cut a release](../../how-to/cut-a-release.md). It also holds each workflow header against the records that govern the workflow.

## Consequences

A typo in a trigger, a missing permission, or an asset name that nobody offers a reader shows itself only at a tag. A fixture that reads the text is the exception. So the fixtures above are the only run these files get between two releases.

A hand run repairs a tag that has no release yet, with no new tag. It cannot repair a tag that has a release ([HW-PD-0009](0009-a-release-gets-all-its-assets-in-one-create-call-and-a-person-deletes-it-to-run-again.md)).

The fixtures are red on a new name for a step that a fixture finds by name. They are also red on an asset name built from parts.
