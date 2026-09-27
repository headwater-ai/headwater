---
id: HW-HOW-rotate-or-revoke-the-apt-signing-subkey
status: current
status_since: 2026-09-27
summary: "The order in which a maintainer replaces or withdraws the subkey that signs the Headwater APT metadata, from a new subkey to the old one's expiry"
last_verified: 2026-09-27
title: "Rotate or revoke the APT signing subkey"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: direct
---

# Rotate or revoke the APT signing subkey

**Audience:** the owner of the Headwater primary key. An adopter needs no step of this guide. [HW-DR-0094](../decisions/0094-the-apt-repository-is-served-from-headwater-tools-and-signed-by-a-subkey-the-owner-s-offline-key-certifies.md) is the decision that this guide follows.

## Before you start

You need the primary key, which you keep offline, and admin access to the repository settings. A signing subkey is in the `APT_SIGNING_KEY` Actions secret. The public keyring at `site/apt/headwater-archive-keyring.asc` is what adopters trust.

Rotate when a subkey comes near its expiry date. Revoke when a subkey leaves your control, for example when a person who could read the secret leaves.

Adopters trust the primary key through the keyring. So a new subkey that the same primary key certifies needs no step from an adopter, when the keyring on the site already holds it.

## Steps

1. On the offline machine, add a new signing subkey to the primary key: `gpg --quick-add-key <primary-fingerprint> ed25519 sign 2y`.
2. To revoke, also revoke the old subkey now: `gpg --edit-key <primary-fingerprint>`, then `key <n>`, `revkey` and `save`. To rotate, keep the old subkey until step 7.
3. Export the public keyring: `gpg --armor --export <primary-fingerprint> > site/apt/headwater-archive-keyring.asc`. Commit it in a pull request and merge it. The Cloudflare build serves it at `https://headwater.tools/apt/headwater-archive-keyring.asc`.
4. Export the new subkey alone, with no passphrase: `gpg --armor --export-secret-subkeys <new-subkey-fingerprint>! > subkey.asc`. The `!` exports that one subkey and no other.
5. Put the contents of `subkey.asc` into the `APT_SIGNING_KEY` Actions secret, then delete `subkey.asc` with `shred -u subkey.asc`.
6. Sign again. A release is immutable, so cut the next release per [Cut a release](cut-a-release.md). Its metadata carries the new signature, and the Cloudflare build serves it.
7. To rotate, let the old subkey expire. Do not revoke it earlier, because a machine that has not fetched the keyring from step 3 still needs it.

## How to know it worked

On a Debian or Ubuntu machine, run `apt-get update` against the repository. It must fetch `InRelease` with no `NO_PUBKEY` and no `EXPKEYSIG` line.

Run `gpg --verify InRelease` on a copy of the file with the keyring from step 3 imported. The line `Good signature` must name the new subkey.

The `publish` job of the release prints no warning about `APT_SIGNING_KEY`.
