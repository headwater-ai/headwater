---
id: HW-HOW-cut-a-release
status: current
status_since: 2026-09-27
summary: "Bump the version, push a v* or taxonomy tag, confirm the workflow each tag starts, and move the install text last."
last_verified: 2026-09-27
title: "Cut a release"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: reconstructed
---

# Cut a release

**Audience:** a contributor to this repository who did not cut the last release. An adopter of Headwater needs no step of this guide, and the consumer surface in `.headwater/overlay.yml` does not list it. An adopter gets the result: each release has a changelog entry, a binary, the crates on crates.io and an install line that works.

This repository cuts two kinds of release. An engine release ships the `headwater` binary and the crates. A taxonomy release ships the `headwater/standard` package. Each kind has its own tag namespace and its own workflows, and you can cut one without the other. The steps below come from the v0.3.0 release of 2026-09-26 and from the workflow files that they name. Where this page and a workflow file disagree, the workflow file is correct and this page is stale.

## Before you start

Four workflows under `.github/workflows/` take part in a release. A pushed tag starts three of them, and a person starts the fourth by hand.

| workflow | what starts it | what it does |
|---|---|---|
| `.github/workflows/release.yml` | `v*` | Builds three archives, runs two smoke jobs on hosts that did not build them, and creates the GitHub release. |
| `.github/workflows/publish-crates.yml` | `v*` | Publishes every workspace crate to crates.io, leaves first, in the order that its `order` variable states. |
| `.github/workflows/release-taxonomy.yml` | `taxonomy/headwater-standard/v*` | Publishes `taxonomy-source/headwater-standard` at the tagged commit and attaches the zip to the release. |
| `.github/workflows/yank-crates.yml` | `workflow_dispatch` | Yanks the crate versions that a person names. No push and no tag starts it. |

`sh tools/repo/release-guide-fixtures.sh` holds this table against the `on:` block of each workflow, in both directions. A new release workflow without a row here fails that suite.

A tag matches one of the two patterns at most. So an engine tag never starts the taxonomy workflow, and a taxonomy tag never starts an engine workflow. Both engine workflows take `workflow_dispatch` with a `tag` input too. `release-taxonomy.yml` does the same. Use that input to run a workflow again for a tag that exists already.

Before an engine release, make sure that these conditions are true:

- The version milestone for the release is closed, and the owner said yes to the release. The `RELEASE READY` line in `.claude/agents/headwater-product-owner.md` is where that question starts.
- CI on `main` is green.
- You have push access to tags on `headwater-ai/headwater`. The crates.io token is the `CARGO_REGISTRY_TOKEN` secret, and only the workflows use it.

## Steps

### The engine release

1. **Bump the version on a branch.** Change `version` under `[workspace.package]` in `engine/Cargo.toml`. Change the `version` of each internal crate under `[workspace.dependencies]` in the same file to the same value.
2. **Update the lock.** Run a build with the engine manifest, so that `engine/Cargo.lock` records the new versions. Commit the lock.
3. **Re-record the fixtures that print the version.** Run the workspace suite with `HEADWATER_BLESS=1`, as [DEVELOPING.md](../../DEVELOPING.md) says. Read the diff. In v0.3.0 this moved six recorded fixtures, and each change was the version string only.
4. **Write the changelog entry.** Add an `<h2>` section for the new version at the top of `site/changelog/index.html`. Link the release page for the tag, give the date, and name what an adopter meets. The v0.3.0 entry is the model.
5. **Merge the release pull request.** It does not change the install text. Pull requests #1124 (v0.2.1) and #1169 (v0.3.0) show the shape: nine files, and none of them is an install page.
6. **Choose the commit to tag.** Use a commit on `main` that carries the version bump and that has a green CI run. For v0.3.0 the owner tagged the head of `main`, and not the merge commit of the release pull request. Both are correct if both conditions are true.
7. **Tag and push.** Use an annotated tag:

        git tag -a v<version> -m "Release v<version>"
        git push origin v<version>

    The tag starts `release.yml` and `publish-crates.yml`. Each one refuses a tag that does not agree with `[workspace.package] version`. `release.yml` compares the tag with what the binary prints. `publish-crates.yml` compares the tag with what `cargo metadata` reads.
8. **Move the install text after the tag exists.** Search the tree for the previous tag, and change each install line to the new tag. Pull request #1194 is the model, and it changed seven files. Do this step last. Group 4 of `tools/repo/readme-fixtures.sh` requires that the tag which `README.md` pins resolves on the remote, or that the page says the tag is not cut. So a change that pins the new tag cannot merge before the tag exists. [DEVELOPING.md](../../DEVELOPING.md) states that exclusive or.

The asset names follow the pattern `headwater-<tag>-<target>.tar.gz`, and each archive has a `.sha256` file beside it. The matrix in `release.yml` is the list of targets. Group 7 of `tools/repo/readme-fixtures.sh` holds that list against `README.md`, so this page does not copy it.

### The taxonomy release

The four-step process in the header of `.headwater/packages/headwater-standard/package.yml` is the maintenance loop. A taxonomy release adds a tag to its result.

1. **Change the version in the source.** Set `version` in `taxonomy-source/headwater-standard/package.yml`.
2. **Publish, pin, vendor and resolve.** Do the four steps in the order that the header of `.headwater/packages/headwater-standard/package.yml` gives. Commit the result on a branch and merge it.
3. **Tag the merged commit and push.** Use the tag `taxonomy/headwater-standard/v<version>`. The workflow publishes again at the tagged commit. It refuses a tag whose version is not the version that `publish` wrote.
4. **Move the pinned taxonomy tag and digest.** `README.md` and `docs/tutorials/your-first-governed-corpus.md` pin one taxonomy tag and its digest. Change both after the tag exists. Group 6 of `tools/repo/readme-fixtures.sh` reads the digest out of the tree of the pinned tag, so a digest from a different tag fails.

### How the two releases depend on each other

The workflows do not depend on each other, and a taxonomy release needs no engine version bump. The one link is `requires_engine` in `taxonomy-source/headwater-standard/package.yml`. When a package version raises that floor, cut the engine release that satisfies it first. An engine release depends on no taxonomy release.

## When a step fails

**A crates.io publish stops partway.** Run the failed job again. Do not yank. The loop in `publish-crates.yml` skips each crate that crates.io already has at this version. So a second run starts where the first run stopped. For v0.3.0, the first attempt stopped at `headwater-compat`, because the crates.io index did not yet list `headwater-scaffold` after the fixed 30-second wait. Four crates were not published. A second run of the failed job published them. The loop waits out a 429 from crates.io and tries again, and [HW-OBL-0182](../obligations/0182-the-publish-crates-retry-loop-has-no-retry-ceiling.md) records that this wait has no ceiling.

**A release has no archive, or an archive is wrong.** Run `release.yml` by hand with the `tag` input. It attaches the archives to a tag that exists, and it cuts no new tag. With `publish` set to `false`, it builds and runs the smoke jobs and creates no release. The header of `release.yml` gives the command.

**A crate version can never become complete.** Only then use `yank-crates.yml`. Version 0.1.0 is the example: it was published before `headwater-compat` could be, and 0.1.1 replaced it. A yank hides a version from a new `cargo add`. It deletes nothing, and a lockfile that names the version continues to work. A crate that is complete but wrong gets a new patch version and no yank.

## How to know it worked

- `gh release view v<version> --json url,assets` lists three archives and three `.sha256` files, one pair for each row of the matrix in `release.yml`.
- For each crate in the `order` variable of `publish-crates.yml`, `https://crates.io/api/v1/crates/<name>` reports the new version as `max_version`. The crates.io API refuses a request that has no `User-Agent` header, so send one.
- `site/changelog/index.html` has an entry for the new version.
- CI is green on the pull request that moves the install text, which includes `tools/repo/readme-fixtures.sh`.
- For a taxonomy release, `gh release view taxonomy/headwater-standard/v<version>` lists `headwater-standard-<version>.zip`, and the release notes state the digest that `README.md` now pins.
