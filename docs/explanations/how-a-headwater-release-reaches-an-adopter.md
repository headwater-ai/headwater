---
id: HW-EXP-how-a-headwater-release-reaches-an-adopter
status: current
status_since: 2026-09-28
summary: "An engine tag v* ships the binary and a taxonomy tag taxonomy/headwater-standard/v* ships the package zip. Each release states the digest an adopter pins. The APT repository exists only when CI holds the signing subkey, and that subkey signs it."
last_verified: 2026-09-30
title: "How a Headwater release reaches an adopter"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: reconstructed
relations:
  draws_on:
    - to: HW-DR-0088
    - to: HW-DR-0089
    - to: HW-DR-0090
    - to: HW-DR-0091
    - to: HW-DR-0094
    - to: HW-DR-0022
---

# How a Headwater release reaches an adopter

## Scope

This page states what an adopter gets from a Headwater release, and how the adopter can trust it. It is for a person who installs the engine, or who pins a version of the `headwater/standard` taxonomy in a corpus. It states no new rule. Each fact comes from one of the decisions that it draws on, or from the install section of [README.md](../../README.md). Where this page and a release workflow disagree, the workflow is correct and this page is stale.

How a maintainer cuts a release is out of scope. [Cut a release](../how-to/cut-a-release.md) and the process decisions that start at [HW-PD-0009](../process/decisions/0009-a-release-gets-all-its-assets-in-one-create-call-and-a-person-deletes-it-to-run-again.md) hold that account. This page also does not rule whether the kinds of this corpus follow the four modes of Diátaxis ([HW-OBL-0097](../obligations/0097-whether-the-four-modes-of-di-taxis-are-the-kind-set.md)).

## How it works

Headwater has two release lines. Each line has its own tags, and each line gives a different thing to an adopter.

1. **The engine line.** A tag that matches `v*` starts the engine release. It attaches the `headwater` binary in archives for Linux and macOS ([HW-DR-0088](../decisions/0088-an-engine-release-and-a-taxonomy-release-have-disjoint-tag-namespaces-and-are-cut-independently.md)).
2. **The taxonomy line.** A tag that matches `taxonomy/headwater-standard/v*` starts the taxonomy release. It attaches one zip of the `headwater/standard` package ([HW-DR-0088](../decisions/0088-an-engine-release-and-a-taxonomy-release-have-disjoint-tag-namespaces-and-are-cut-independently.md)).
3. **One field links the two lines.** A tag cannot match both patterns, so no release waits on the other. The package states the oldest engine that it needs in `requires_engine`. When a package raises that value, the engine release that satisfies it comes first ([HW-DR-0088](../decisions/0088-an-engine-release-and-a-taxonomy-release-have-disjoint-tag-namespaces-and-are-cut-independently.md)).
4. **The taxonomy zip comes from the source at the tag.** The taxonomy release builds the engine at the tagged commit, and it runs `headwater taxonomy publish` on `taxonomy-source/headwater-standard` at that commit. It never attaches the copy of the package that the repository keeps under `.headwater/packages/` ([HW-DR-0089](../decisions/0089-a-taxonomy-release-publishes-the-source-at-the-tagged-commit-and-never-ships-the-vendored-copy.md)).
5. **The release notes state the digest to pin.** Each release of either line prints the `release.digest` of `headwater/standard` in its notes. The taxonomy release prints the digest that its own `publish` run printed. The engine release prints the digest in the release record of the tree at its tag ([HW-DR-0090](../decisions/0090-each-release-states-in-its-notes-the-release-digest-that-a-consumer-pins.md)).
6. **The adopter pins that digest.** The adopter writes the digest into `.headwater/taxonomy.yml`, or gives it to `headwater taxonomy vendor --expect`. `vendor` computes the digest again from the files, and it refuses a package that does not match the pin ([HW-DR-0022](../decisions/0022-q22-the-integrity-posture-of-a-published-package.md)).
7. **The runner image sets the C library floor.** Each build row of the engine release names its runner image in the workflow file, and no variable changes it. The glibc archive needs the glibc version of that image. The build uses `--locked`, so the binary uses the dependency versions in the committed `engine/Cargo.lock` ([HW-DR-0091](../decisions/0091-the-runner-image-of-the-engine-release-build-is-written-in-the-workflow-because-it-sets-the-glibc-floor.md)).
8. **The APT repository is signed only when the signing secret is set.** The engine release also builds one `amd64` Debian package. The package holds no taxonomy, so the adopter runs `headwater taxonomy vendor` after the install. When the `APT_SIGNING_KEY` secret is set, a subkey signs the metadata of the APT repository on `headwater.tools`. When the secret is not set, the release carries the Debian package and no metadata. Each deploy of the site then stops, so the site keeps the APT repository of the last signed release. CI holds that subkey, and the owner holds the primary key offline. [Rotate or revoke the APT signing subkey](../how-to/rotate-or-revoke-the-apt-signing-subkey.md) says what an adopter does when that key changes ([HW-DR-0094](../decisions/0094-the-apt-repository-is-served-from-headwater-tools-and-signed-by-a-subkey-the-owner-s-offline-key-certifies.md)).

The table shows what each route gives an adopter, and what the adopter checks.

| what the adopter gets | from | what the adopter checks |
|---|---|---|
| the `headwater` binary in an archive | an engine release, tag `v*` | the checksum file beside the archive, with `sha256sum -c` |
| the `headwater` binary in a Debian package | the APT repository on `headwater.tools` | apt checks the signature of the repository metadata, which the release signs only when the `APT_SIGNING_KEY` secret is set. Without the secret, this route does not exist for that release. |
| the `headwater/standard` package as a zip | a taxonomy release, tag `taxonomy/headwater-standard/v*` | `headwater taxonomy vendor --expect <digest>`, with the digest from the release notes |

[README.md](../../README.md) gives the commands for each route, and the current version of each line.

## Why it is this way

Each record below holds the reasons for one part of the design. This page does not repeat them.

- The two lines have tags that cannot collide, and `requires_engine` is the only link ([HW-DR-0088](../decisions/0088-an-engine-release-and-a-taxonomy-release-have-disjoint-tag-namespaces-and-are-cut-independently.md)).
- The zip and the tag make one claim, because the zip comes from the source at the tag ([HW-DR-0089](../decisions/0089-a-taxonomy-release-publishes-the-source-at-the-tagged-commit-and-never-ships-the-vendored-copy.md)).
- The adopter pins the digest from the release that the adopter downloaded ([HW-DR-0090](../decisions/0090-each-release-states-in-its-notes-the-release-digest-that-a-consumer-pins.md)).
- The runner image is in the workflow, because it sets the glibc floor that an adopter reads ([HW-DR-0091](../decisions/0091-the-runner-image-of-the-engine-release-build-is-written-in-the-workflow-because-it-sets-the-glibc-floor.md)).
- When the APT repository exists, a key that CI holds signs it, and the owner's offline key certifies that key ([HW-DR-0094](../decisions/0094-the-apt-repository-is-served-from-headwater-tools-and-signed-by-a-subkey-the-owner-s-offline-key-certifies.md)).
- A digest is a check against a pin that a person committed, and it is not a signature ([HW-DR-0022](../decisions/0022-q22-the-integrity-posture-of-a-published-package.md)). Whether a pin can also prove who published the package is an open question ([HW-OBL-0115](../obligations/0115-a-pinned-digest-authenticates-the-pin-and-never-the-publisher.md)).

When one of these records is superseded or withdrawn, `headwater check` reports this page through its `draws_on` edge, and this page must change with it.
