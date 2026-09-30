#!/bin/sh
# deploy-site.sh — build https://headwater.tools/ from this commit, fill its
# figures from a run of the engine, and deploy it with `wrangler`.
#
# WHO RUNS THIS
#
#   The `Deploy the site` job of `.github/workflows/deploy-site.yml`, which
#   two workflows call: `ci.yml` on a push to `main`, after every gating job
#   of that push is green, and `release.yml` with `ref: main` after it
#   creates a release, so `apt/` serves that release (#1316). Nobody runs it
#   by hand. It reads `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` from
#   the environment, which only that job holds.
#
#   Until #1273 Cloudflare Workers Builds deployed the site from the committed
#   pages, and the committed pages carried the figures. Workers Builds starts
#   on the push and cannot wait for CI, and it has no built engine to measure
#   with, so the figures had to be committed, and every pull request that
#   added a document conflicted with every other one on the pages. The
#   decision that supersedes HW-DR-0039 records the move.
#
# WHAT IT DOES, IN ORDER
#
#   1. `mkdocs build --strict`, the generated half.
#   2. `tools/site/assemble-site.sh`, the two halves into `.headwater/site-deploy`.
#   3. `tools/site/refresh-figures.sh --into .headwater/site-deploy`, which
#      measures this tree and fills every `data-figure` element of the copies.
#      The pages under `site/` are not touched.
#   4. `tools/site/fetch-apt.sh .headwater/site-deploy`, the signed APT
#      repository of the newest release. `tools/site/cloudflare-build.sh` ran
#      this step before, and a deploy that dropped it would take the APT
#      repository off the live site.
#   5. A look for `apt/dists/stable/Release` and `InRelease` in the assembled
#      directory: a tree without them refuses the deploy, whatever step 4
#      returned. A taxonomy release that became GitHub's "latest" release
#      once made step 4 exit 0 with no `apt/`, and every deploy then took
#      the APT repository off the live site (#1449).
#   6. A last look at the assembled directory: an empty `data-figure` element
#      there refuses the deploy.
#   7. `npx wrangler deploy`, at the pinned version below.
#
#   Each step runs under `set -eu`. A step that fails ends the script before
#   step 7, so a failing measurement, an unknown key, a figure on no page or
#   a blank marker publishes nothing, and the live site goes on serving the
#   last good deploy. `tools/site/figures-fixtures.sh` holds that with
#   `wrangler` stubbed.
#
# The toolchain is the caller's to install: the Rust engine at
# `engine/target/{release,dev-release}/headwater`, and the pinned `mkdocs`,
# `pymdown-extensions` and `PyYAML` that `.github/workflows/ci.yml` names.
set -eu

# The wrangler release this deploy runs. Pinned for the reason `mkdocs` is: a
# deploy tool that moved without a commit is drift in the one step nobody reads.
WRANGLER_VERSION=4.142.0

root=$(cd "$(dirname "$0")/../.." && pwd)
cd "$root"
out=.headwater/site-deploy

echo "deploy-site.sh: rendering the generated half"
python3 -m mkdocs build --strict

echo "deploy-site.sh: composing the served directory"
sh tools/site/assemble-site.sh

echo "deploy-site.sh: measuring this tree into the assembled pages"
sh tools/site/refresh-figures.sh --into "$out"

echo "deploy-site.sh: adding the signed APT repository of the newest release"
sh tools/site/fetch-apt.sh "$out"

echo "deploy-site.sh: refusing an assembled directory with no APT repository"
if [ ! -f "$out/apt/dists/stable/Release" ] || [ ! -f "$out/apt/dists/stable/InRelease" ]; then
    echo "deploy-site.sh: the served tree has no apt/dists/stable/Release or InRelease, so this deploy stops and the site keeps its APT repository" >&2
    exit 1
fi

echo "deploy-site.sh: refusing a blank figure in the assembled directory"
sh tools/site/check-site-figures.sh "$out"

echo "deploy-site.sh: deploying with wrangler $WRANGLER_VERSION"
npx --yes "wrangler@$WRANGLER_VERSION" deploy

echo "deploy-site.sh: done"
