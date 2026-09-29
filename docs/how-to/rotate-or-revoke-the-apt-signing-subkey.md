---
id: HW-HOW-rotate-or-revoke-the-apt-signing-subkey
status: current
status_since: 2026-09-27
summary: "How a maintainer replaces or withdraws the subkey that signs the Headwater APT metadata, and what each adopter must download again"
last_verified: 2026-09-30
title: "Rotate or revoke the APT signing subkey"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: direct
---

# Rotate or revoke the APT signing subkey

**Audience:** the owner of the Headwater primary key. An adopter does one step after a rotation, and step 4 tells them. [HW-DR-0094](../decisions/0094-the-apt-repository-is-served-from-headwater-tools-and-signed-by-a-subkey-the-owner-s-offline-key-certifies.md) is the decision that this guide follows.

## Before you start

You need the primary key, which you keep offline, and admin access to the repository settings. A signing subkey is in the `APT_SIGNING_KEY` Actions secret. The public keyring at `site/apt/headwater-archive-keyring.asc` is what adopters trust.

Rotate when a subkey comes near its expiry date. Revoke when a subkey leaves your control, for example when a person who could read the secret leaves.

apt does not fetch keys. It checks a signature only against the keyring file that the adopter downloaded and named in `signed-by`. A machine whose keyring does not hold a new subkey refuses metadata that the subkey signs, with `NO_PUBKEY`. This is true when the same primary key certifies the subkey. So a rotation puts the new subkey in the public keyring one release before it signs anything. It also tells adopters to download the keyring again.

## Steps

1. On the offline machine, add a new signing subkey to the primary key: `gpg --quick-add-key <primary-fingerprint> ed25519 sign 2y`.
2. To revoke, also revoke the old subkey now: `gpg --edit-key <primary-fingerprint>`, then `key <n>`, `revkey` and `save`. To rotate, keep the old subkey. It continues to sign until step 6.
3. Export the public keyring, which now holds both subkeys: `gpg --armor --export <primary-fingerprint> > site/apt/headwater-archive-keyring.asc`. Commit it in a pull request and merge it. The merge starts the `deploy-site.yml` job, and that job serves the keyring at `https://headwater.tools/apt/headwater-archive-keyring.asc`.
4. Tell adopters to download the keyring again to the path that their `signed-by` names. Put the line in the notes of the next release and on the tracker. To rotate, cut at least one release that the old subkey signs after this step. Adopters then have one release cycle to fetch the keyring. To revoke, there is no such cycle, because the old subkey must not sign again. An adopter who has not fetched the new keyring gets `NO_PUBKEY` at the next release.
5. Export the new subkey alone, with no passphrase: `gpg --armor --export-secret-subkeys <new-subkey-fingerprint>! > subkey.asc`. The `!` exports that one subkey and no other.
6. Put the contents of `subkey.asc` into the `APT_SIGNING_KEY` Actions secret, then delete `subkey.asc` with `shred -u subkey.asc`.
7. Sign again. A release is immutable, so cut the next release per [Cut a release](cut-a-release.md). Its metadata carries the new signature. The `deploy-site` job of the release runs after `publish` and serves it. An adopter whose keyring is older than step 3 gets `NO_PUBKEY` from this release until they do step 4.
8. To rotate, let the old subkey expire after step 7. It signs nothing after step 6.

## How to know it worked

On a Debian or Ubuntu machine with the keyring from step 3, run `apt-get update` against the repository. It must fetch `InRelease` with no `NO_PUBKEY` and no `EXPKEYSIG` line.

Run `gpg --verify InRelease` on a copy of the file with the keyring from step 3 imported. The line `Good signature` must name the new subkey.

The `publish` job of the release prints no warning about `APT_SIGNING_KEY`.
