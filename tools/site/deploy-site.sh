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
#   1. `tools/site/build-site.sh`, which renders the generated half, composes
#      the two halves into `.headwater/site-deploy`, measures the figures into
#      the copies, adds the signed APT repository, and refuses a tree with no
#      APT repository or a blank figure. That script lists its steps and the
#      incident behind each refusal.
#   2. `npx wrangler deploy`, at the pinned version below.
#
#   Each step runs under `set -eu`. A step that fails ends the script before
#   step 2, so a failing measurement, an unknown key, a figure on no page or
#   a blank marker publishes nothing, and the live site goes on serving the
#   last good deploy. `tools/site/figures-fixtures.sh` holds that with
#   `wrangler` stubbed. `tools/site/preview-site.sh` runs the same build and
#   uploads a preview version instead, which production never serves.
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

sh tools/site/build-site.sh

echo "deploy-site.sh: deploying with wrangler $WRANGLER_VERSION"
npx --yes "wrangler@$WRANGLER_VERSION" deploy

echo "deploy-site.sh: done"
