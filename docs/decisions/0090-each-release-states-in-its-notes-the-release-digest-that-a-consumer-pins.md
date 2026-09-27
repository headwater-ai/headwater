---
id: HW-DR-0090
status: current
status_since: 2026-09-27
summary: "An engine release and a taxonomy release each print the headwater/standard release.digest in their notes, so an adopter pins it from the tag they downloaded."
last_verified: 2026-09-27
title: "Each release states in its notes the release digest that a consumer pins"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: evidenced
relations:
  governs:
    - to: .github/workflows/release.yml
      verified_revision: sha256:66aeccf8d5dd9a9a73f5a2e93de8062823828f404e2f46253368cbcc296cb15f
    - to: .github/workflows/release-taxonomy.yml
      verified_revision: sha256:cc62f8db1425466922c618475983155ec51572a77c846a4634a730e18a804ae4
---

# Each release states in its notes the release digest that a consumer pins

## Context

Before this record, the reason lived only in two step comments. One was on the step "The digest the package in this tree publishes" of `.github/workflows/release.yml` (lines 350 to 365 on 2026-09-27). The other was on the step that attaches the archive in `.github/workflows/release-taxonomy.yml` (lines 139 to 145).

An adopter pins `headwater/standard` by its `release.digest` and passes the value to `headwater taxonomy vendor --expect`. `gh release create --generate-notes` writes a list of merged pull requests and a compare link, and nothing else. So an engine release that used only that flag stated no digest. `v0.1.2` is such a release. `README.md` then sent a reader to the notes of a different tag for the value, which is [#775](https://github.com/headwater-ai/headwater/issues/775).

## Decision

Each release states the `release.digest` of `headwater/standard` in its release notes.

- The engine release builds the text from the release record in the tree at the tag, `.headwater/packages/headwater-standard/release.yml`, in a step of its own. It passes the text with `--notes-file` together with `--generate-notes`. `gh` puts the text of the file first and adds the generated list after it.
- The taxonomy release passes `--notes` with the digest that its own `taxonomy publish` run printed ([HW-DR-0089](0089-a-taxonomy-release-publishes-the-source-at-the-tagged-commit-and-never-ships-the-vendored-copy.md)).

The step of the engine release that writes the notes reaches no network, and it refuses a record that states no digest.

## Consequences

An adopter reads the digest from the notes of the tag they downloaded, and never from the notes of a different tag. The digest is the value of the field in the release record. It is not what `sha256sum` prints for any one file.

The notes step of `release.yml` runs only on a pushed tag or a hand run. So `tools/repo/readme-fixtures.sh` finds that step by its name and runs it against the checkout. It compares what the step writes with the digest in the record of that checkout. A new name for the step is a red fixture until the fixture follows it ([HW-PD-0012](../process/decisions/0012-the-release-workflows-are-separate-from-ci-one-tag-value-serves-both-entry-points-and-fixtures-read-them-statically.md)).
