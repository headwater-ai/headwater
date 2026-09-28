---
id: HW-DR-0097
status: current
status_since: 2026-09-28
summary: "The pages under site/ commit every data-figure element empty. CI deploys on main pushes, and the release workflow deploys after publish. Both use wrangler."
last_verified: 2026-09-28
title: "A figure on a hand-built page is measured when the site is published, and the committed page carries none"
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  supersedes:
    - HW-DR-0039
  constrains:
    - HW-DR-0047
  traces_to:
    - HW-DR-0037
  governs:
    - to: tools/site/refresh-figures.sh
      verified_revision: sha256:1b3005cd7aa99a8c72b48ed5edd2db826240470f6e125ce1202918e175ab046f
    - to: tools/site/deploy-site.sh
      verified_revision: sha256:93498827528df8182c99854088c378e5c6a911cf3a1c0deb030be5996bafe4ac
    - to: tools/site/check-site-figures.sh
      verified_revision: sha256:79e7c61aebe2d199337536022b5046c5968e11e43e4f6889b783bf5f5a2afdde
    - to: tools/site/figures-fixtures.sh
      verified_revision: sha256:4819ee634449e64a510ebddcffe1cf39707b8005a885ea0f9e64c8cd2b19146e
---

# A figure on a hand-built page is measured when the site is published, and the committed page carries none

## Context

[HW-DR-0039](0039-q39-how-a-figure-reaches-a-hand-built-page.md) put the interpolation of a figure before the commit. `tools/site/refresh-figures.sh` wrote each measured number into the committed page, because nothing ran between the commit and the served bytes. That record names its own reopening condition: a build command in the deploy path. [HW-DR-0047](0047-how-the-two-halves-of-the-site-share-one-host.md) put one there, so the condition is met.

**The committed figures made each page a fold over the corpus.** On `origin/main` at `c0caabd9`, three of the eight pages under `site/` carried 106 figure elements over 34 keys. The landing page carried 57, the proof page 44 and the how-it-works page 5. Any pull request that adds a document moves those figures. `.gitattributes` declared the three pages `-merge`, so every such pair of pull requests conflicted on the pages.

**The cost is serial merging.** [#1273](https://github.com/headwater-ai/headwater/issues/1273) measured 11 pull requests that landed one at a time on 2026-09-27, between 17:12Z and 22:26Z, about 29 minutes each. The merge queue ([HW-PD-0020](../process/decisions/0020-merges-go-through-the-github-merge-queue-one-squash-commit-per-pull-request.md)) takes a group of 5. It used a group of 1, because each entry conflicted with the one before it.

**Three shapes could measure at publish time.**

- A. A CI job on the push to `main` measures the figures, then deploys the assembled directory with `wrangler`. HW-DR-0047 weighed this mechanism and chose Workers Builds, because this one needs two repository secrets and the retirement of the git integration.
- B. CI measures the figures and hands Cloudflare an artifact. Workers Builds starts on the push and finishes in minutes, and CI takes about 15 minutes. A GitHub Actions artifact also needs a token to download, on a public repository too. Nothing on the Cloudflare side can wait for CI.
- C. Workers Builds builds the Rust engine and measures. That costs build minutes on every push and runs near the build time limit. It cannot use a released binary, because `refresh-figures.sh` refuses a binary older than the engine sources (exit 3). That guard exists because a stale binary once measured 12 figures wrong. Nobody measured this option.

## Decision

**The committed pages carry every `data-figure` element empty.** `sh tools/site/refresh-figures.sh --blank` writes that form, and it is the one command that resolves a conflict on a page: take either side's prose, then blank. It measures nothing and needs no engine.

**Shape A is the deploy path.** The `Deploy the site` job of `.github/workflows/ci.yml` runs `tools/site/deploy-site.sh` on a push to `main`. The `Deploy the site` job of `.github/workflows/release.yml` runs the same script only after `publish` succeeds. Both jobs share the `deploy-site` concurrency group. The CI job waits for the two gating jobs of its push, `Engine tests` and `headwater check (advisory)`. The release job waits for the tag and publish jobs. Neither deploy is a required check. The script renders the generated half, assembles the directory, and fills the figures with `refresh-figures.sh --into .headwater/site-deploy`. Then it adds the APT repository with `tools/site/fetch-apt.sh` and deploys with `wrangler` at a pinned version. Shapes B and C are rejected for the reasons in the context above.

**A deploy that cannot measure publishes nothing.** The script runs under `set -eu`. A failing measurement, an unknown key, a measured figure that reaches no page, or a marker the pattern cannot read stops it before `wrangler` runs. `tools/site/check-site-figures.sh` then refuses an assembled directory that still holds an empty element. `tools/site/figures-fixtures.sh` holds each of these with `wrangler` stubbed.

**Every CI event measures.** `refresh-figures.sh --check` fails on a value in a committed marker. It also fails on each measurement failure above. The step after the assembly runs `--into` on the assembled directory. So a merge group finds a key with no measurement before the merge, not at the deploy. A push to `main` deploys through `ci.yml`, and a published engine release deploys through `release.yml`.

**The clock partition is removed, not reduced.** The eight figures that read the clock were compared against a committed value. A second engine run at the page's own date excused a difference that the clock alone caused. Nothing committed is compared against a run now, so that comparison has no input.

**The pages are not derived artifacts.** `headwater derived` knows three producers, and none of them claims a page under `site/`. `.gitattributes` declares no page under `site/`. The figures clause of `.githooks/pre-commit` is removed. The site arm of `.githooks/merge-regenerate` is removed, and so is the site-review marker that `.githooks/pre-push` and `.githooks/post-rewrite` read.

## Consequences

**Two things wait on the owner, and the merge of this record waits on the first.** Somebody must set the repository secrets `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID`, and disconnect the Workers Builds git integration of the `headwater` Worker. Until the secrets exist, the deploy job fails alone with one error line that names both. While the integration stays connected, `tools/site/cloudflare-build.sh` refuses each build with a blank figure. That build publishes nothing, which is better than a live page with blanks.

**The merge queue can take pull requests in groups again.** No shared derived file moves between two pull requests that each add a document. HW-PD-0020 states the rule.

**A pull request that was open before this record lands conflicts once on each page it touched.** Its own `.gitattributes` still declares the page `-merge`. Keep the branch's prose, run `sh tools/site/refresh-figures.sh --blank`, and commit. After that merge the pages move only with the prose of the branch.

**The live site shows the figures of the newest deploy, not of the newest commit.** A push to `main` that fails a gate does not deploy, and the live site keeps the last good deploy.

**One known gap survives unchanged.** `used` in `refresh-figures.sh` is a union across pages. A key lost on one page is invisible while another page carries it. The script header records the measurement.

**This record reopens on either of two events.** The deploy job fails on 3 consecutive pushes to `main` for a reason other than the secrets. Or Cloudflare offers a build step that can wait on CI and read its artifact, which would make shape B available.
