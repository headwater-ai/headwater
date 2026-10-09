#!/bin/sh
# build-site.sh — assemble https://headwater.tools/ from this commit into
# `.headwater/site-deploy`, with every figure measured, and serve nothing.
#
# WHO RUNS THIS
#
#   `tools/site/deploy-site.sh`, which then deploys the directory to
#   production, and `tools/site/preview-site.sh`, which uploads it as a
#   preview version that production never serves. One build, so a preview
#   shows the bytes a deploy of the same commit would serve.
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
#      directory: a tree without them refuses the build, whatever step 4
#      returned. A taxonomy release that became GitHub's "latest" release
#      once made step 4 exit 0 with no `apt/`, and every deploy then took
#      the APT repository off the live site (#1449).
#   6. A last look at the assembled directory: an empty `data-figure` element
#      there refuses the build.
#
#   Each step runs under `set -eu`, so a step that fails exits with its own
#   status and the caller never reaches `wrangler`. `tools/site/figures-fixtures.sh`
#   holds that through `deploy-site.sh`, with `wrangler` stubbed.
#
# The toolchain is the caller's to install: the Rust engine at
# `engine/target/{release,dev-release}/headwater`, and the pinned `mkdocs`,
# `pymdown-extensions` and `PyYAML` that `.github/workflows/ci.yml` names.
set -eu

root=$(cd "$(dirname "$0")/../.." && pwd)
cd "$root"
out=.headwater/site-deploy

echo "build-site.sh: rendering the generated half"
python3 -m mkdocs build --strict

echo "build-site.sh: composing the served directory"
sh tools/site/assemble-site.sh

echo "build-site.sh: measuring this tree into the assembled pages"
sh tools/site/refresh-figures.sh --into "$out"

echo "build-site.sh: adding the signed APT repository of the newest release"
sh tools/site/fetch-apt.sh "$out"

echo "build-site.sh: refusing an assembled directory with no APT repository"
if [ ! -f "$out/apt/dists/stable/Release" ] || [ ! -f "$out/apt/dists/stable/InRelease" ]; then
    echo "build-site.sh: the served tree has no apt/dists/stable/Release or InRelease, so this build stops and the site keeps its APT repository" >&2
    exit 1
fi

echo "build-site.sh: refusing a blank figure in the assembled directory"
sh tools/site/check-site-figures.sh "$out"

echo "build-site.sh: refusing a file Cloudflare Workers will not serve"
sh tools/site/check-site-asset-size.sh "$out"
