---
id: HW-DR-0094
status: current
status_since: 2026-09-27
summary: "Headwater ships one amd64 Debian package, taxonomy-free, through an APT repository on headwater.tools whose metadata a CI-held signing subkey signs"
last_verified: 2026-09-28
title: "The APT repository is served from headwater.tools and signed by a subkey the owner's offline key certifies"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: direct
relations:
  governs:
    - to: .github/workflows/release.yml
      verified_revision: sha256:1f39541b866aafcb049d493b3d9f4904b552d6f6d5d5bb545264a8f3dd530c47
    - to: tools/site/fetch-apt.sh
      verified_revision: sha256:c90a48b65a9eadc95661a619a9d2254e4271df87c4b2e2b8d3a54fa0a287f3d3
---

# The APT repository is served from headwater.tools and signed by a subkey the owner's offline key certifies

## Context

[#764](https://github.com/headwater-ai/headwater/issues/764) asks for an install through `apt` on Debian and Ubuntu. Before this record, an engine release carried three archives and their checksums, and no step built a `.deb`, wrote APT metadata or signed anything.

The owner made two rulings on the issue. On 2026-09-22 the owner chose "CLI package taxonomy-free" ([comment](https://github.com/headwater-ai/headwater/issues/764#issuecomment-5770899696)). On 2026-09-27 the owner accepted the recommendation for the host and the key. The repository is on headwater.tools, and a signing subkey is an Actions secret. The owner holds the primary key offline ([comment](https://github.com/headwater-ai/headwater/issues/764#issuecomment-5852855283)).

[HW-DR-0047](0047-how-the-two-halves-of-the-site-share-one-host.md) gives the site one deploy script and concurrency group. The main-push and post-publish jobs share both.

## Decision

**One package.** `release.yml` builds `headwater_<version>_amd64.deb` from the static musl archive. The Debian version is the tag without its `v`. A pre-release hyphen becomes `~`, so apt puts `0.5.0~rc.1` before `0.5.0`. The package holds the paths that `surface.deb` in `.headwater/overlay.yml` lists, and a step compares the two in both directions. It holds the binary, the license, and the completion scripts for bash, zsh and fish. PowerShell has no completion path on Debian, so the package does not carry that script.

**No taxonomy in the package.** The package carries no taxonomy file and no maintainer script. An adopter runs `headwater taxonomy vendor` in a corpus, as on the tarball and cargo routes.

**The baseline.** The one supported architecture is `amd64`. The binary links no C library, so the package runs on any `amd64` Debian 11 or later and any Ubuntu 20.04 or later. The `smoke-apt` job installs it on `debian:bullseye` and `ubuntu:24.04`. No release builds an `arm64` Linux binary, so there is no `arm64` package.

**The key.** A signing subkey signs the metadata. The secret `APT_SIGNING_KEY` holds the ASCII-armored private subkey, with no passphrase, because the secret store is its protection. The owner holds the primary key offline. The publish step imports the subkey into a directory that it makes and removes. The key does not go into a log, the tree or an artifact.

**The route to the site.** When the secret is set, the release carries `Packages`, `Release`, `InRelease` and `Release.gpg` beside the package, in the one `gh release create` call of [HW-PD-0009](../process/decisions/0009-a-release-gets-all-its-assets-in-one-create-call-and-a-person-deletes-it-to-run-again.md). `tools/site/fetch-apt.sh` copies those assets from the newest release into `apt/` of the served directory. The main-push deploy can run before a new release exists, so `release.yml` runs the same site deploy after `publish` succeeds. When the newest release has no `InRelease`, the script prints one line and the site has no `apt/` directory. When a download fails for another reason, the script stops the site build, so the deployed site keeps the repository that it serves.

**The proof.** `smoke-apt` makes a local repository of the package, signed by a throwaway key, and installs from it through apt. It also makes a repository signed by a second key and one with no signature. It fails unless apt refuses both. Publishing waits on this job.

## Consequences

Until the owner adds `APT_SIGNING_KEY`, a release carries the `.deb` and no metadata, and the workflow prints a warning that names the secret. The site serves no APT repository in that state. So no page for an adopter states an `apt` install yet. The install text lands with the first signed release, and the public keyring is committed as `site/apt/headwater-archive-keyring.asc` at that time.

The APT key authenticates the publisher of the engine on the APT route only. It does not authenticate a taxonomy package, so [HW-OBL-0115](../obligations/0115-a-pinned-digest-authenticates-the-pin-and-never-the-publisher.md) stays open.

Cloudflare serves one asset of at most 25 MiB. The package built from the v0.4.0 archive is 8,132,796 bytes. The package job and `fetch-apt.sh` each refuse a larger package.

[Rotate or revoke the APT signing subkey](../how-to/rotate-or-revoke-the-apt-signing-subkey.md) gives the order of steps for a new subkey. The apt program checks a signature only against the keyring file that an adopter downloaded. So every adopter downloads the keyring again after a new subkey is added.

Reopen this record when a person asks for it on the tracker. Three requests reopen it: a second host, an `arm64` package, and a package in the Debian archive.
