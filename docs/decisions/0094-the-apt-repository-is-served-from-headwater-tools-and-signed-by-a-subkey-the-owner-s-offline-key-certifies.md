---
id: HW-DR-0094
status: current
status_since: 2026-09-27
summary: "Headwater ships one amd64 Debian package, taxonomy-free, through an APT repository on headwater.tools whose metadata a CI-held signing subkey signs"
last_verified: 2026-09-30
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
      verified_revision: sha256:b0b36b7376133c211948572cc5a835c1d3e46f2e87c986869748c8d5f9170f47
    - to: tools/site/fetch-apt.sh
      verified_revision: sha256:141b9659d92f91b799b7f35ca8604c94394b27a200c47f2392bc5838be1eea6a
---

# The APT repository is served from headwater.tools and signed by a subkey the owner's offline key certifies

## Context

[#764](https://github.com/headwater-ai/headwater/issues/764) asks for an install through `apt` on Debian and Ubuntu. Before this record, an engine release carried three archives and their checksums, and no step built a `.deb`, wrote APT metadata or signed anything.

The owner made two rulings on the issue. On 2026-09-22 the owner chose "CLI package taxonomy-free" ([comment](https://github.com/headwater-ai/headwater/issues/764#issuecomment-5770899696)). On 2026-09-27 the owner accepted the recommendation for the host and the key. The repository is on headwater.tools, and a signing subkey is an Actions secret. The owner holds the primary key offline ([comment](https://github.com/headwater-ai/headwater/issues/764#issuecomment-5852855283)).

[HW-DR-0097](0097-a-figure-on-a-hand-built-page-is-measured-when-the-site-is-published-and-the-committed-page-carries-none.md) gives the site one deploy job, `.github/workflows/deploy-site.yml`. It constrains [HW-DR-0047](0047-how-the-two-halves-of-the-site-share-one-host.md), which ran the deploy in the build service of Cloudflare before that record. Two workflows call that job: `ci.yml` on a push to `main`, and `release.yml` after its `publish` job. A second deploy path would race the first.

## Decision

**One package.** `release.yml` builds `headwater_<version>_amd64.deb` from the static musl archive. The Debian version is the tag without its `v`. A pre-release hyphen becomes `~`, so apt puts `0.5.0~rc.1` before `0.5.0`. The package holds the paths that `surface.deb` in `.headwater/overlay.yml` lists, and a step compares the two in both directions. It holds the binary, the license, and the completion scripts for bash, zsh and fish. PowerShell has no completion path on Debian, so the package does not carry that script.

**No taxonomy in the package.** The package carries no taxonomy file and no maintainer script. An adopter runs `headwater taxonomy vendor` in a corpus, as on the tarball and cargo routes.

**The baseline.** The one supported architecture is `amd64`. The binary links no C library, so the package itself installs on any `amd64` Debian 11 or later and any Ubuntu 20.04 or later. The `smoke-apt` job holds that floor: it installs the package on `debian:bullseye` and `ubuntu:24.04` from a local repository. The install block in the README runs on Debian 12 or later and Ubuntu 20.04 or later. On 2026-09-30 the block installed `headwater` 0.5.0 in clean `debian:bookworm`, `ubuntu:20.04` and `ubuntu:24.04` containers. In a clean `debian:bullseye` container, the block stopped at `apt-get install -y ca-certificates curl` with exit 100. The Debian security archive returned 404 for the packages that step needs. That failure is in the Debian 11 archive and not in the package. The `.github/workflows/readme-apt.yml` workflow runs the README block in clean containers of the floor that the README names, against `https://headwater.tools/apt`. It runs each day and after each release deploys the site. On Debian 11, the README tells the reader to install the Debian package from its file, which needs no network. No release builds an `arm64` Linux binary, so there is no `arm64` package.

**The key.** A signing subkey signs the metadata. The secret `APT_SIGNING_KEY` holds the ASCII-armored private subkey, with no passphrase, because the secret store is its protection. The owner holds the primary key offline. The publish step imports the subkey into a directory that it makes and removes. The key does not go into a log, the tree or an artifact.

**The route to the site.** When the secret is set, the release carries `Packages`, `Release`, `InRelease` and `Release.gpg` beside the package, in the one `gh release create` call of [HW-PD-0009](../process/decisions/0009-a-release-gets-all-its-assets-in-one-create-call-and-a-person-deletes-it-to-run-again.md). `tools/site/fetch-apt.sh` runs in the `deploy-site.yml` job and copies those assets of the newest release into `apt/` of the served directory. `release.yml` calls that job after `publish`, so `apt/` serves the new release and not the release before it ([#1316](https://github.com/headwater-ai/headwater/issues/1316)). When the script cannot copy a whole repository, it stops the deploy job, so the deployed site keeps the repository that it serves. A newest release with no `InRelease` also stops the job. `tools/site/deploy-site.sh` also refuses a served directory with no `apt/dists/stable/Release` or `InRelease`, before it deploys. The script reads `releases/latest`, so that release must be an engine release. `release-taxonomy.yml` creates each taxonomy release with `--latest=false` for this reason. On 2026-09-30 a taxonomy release became the latest release, and each deploy after it published a site with no `apt/` ([#1449](https://github.com/headwater-ai/headwater/issues/1449)).

**The proof.** `smoke-apt` makes a local repository of the package, signed by a throwaway key, and installs from it through apt. It also makes a repository signed by a second key and one with no signature. It fails unless apt refuses both. Publishing waits on this job.

## Consequences

Each release from v0.4.1 is signed. The public keyring is committed as `site/apt/headwater-archive-keyring.asc`, and the README states the `apt` install. When `APT_SIGNING_KEY` is not set, a release carries the `.deb` and no metadata, and the workflow prints a warning that names the secret. In that state `fetch-apt.sh` stops each deploy of the site, and the site keeps the repository of the last signed release.

The APT key authenticates the publisher of the engine on the APT route only. It does not authenticate a taxonomy package, so [HW-OBL-0115](../obligations/0115-a-pinned-digest-authenticates-the-pin-and-never-the-publisher.md) stays open.

Cloudflare serves one asset of at most 25 MiB. The package built from the v0.4.0 archive is 8,132,796 bytes. The package job and `fetch-apt.sh` each refuse a larger package.

[Rotate or revoke the APT signing subkey](../how-to/rotate-or-revoke-the-apt-signing-subkey.md) gives the order of steps for a new subkey. The apt program checks a signature only against the keyring file that an adopter downloaded. So every adopter downloads the keyring again after a new subkey is added.

Reopen this record when a person asks for it on the tracker. Three requests reopen it: a second host, an `arm64` package, and a package in the Debian archive.
