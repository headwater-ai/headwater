#!/bin/sh
# cloudflare-build.sh — what Cloudflare Workers Builds runs before it deploys
# https://headwater.tools/.
#
# WHY THIS IS A FILE AND NOT A FIELD
#
#   The build command lives in the Cloudflare dashboard, where no commit
#   reaches it. It was first set on 2026-09-06 as a literal string that
#   installed `mkdocs` alone, and it went stale the same day: #431 changed
#   `mkdocs.yml` to derive every heading anchor through
#   `pymdownx.slugs.slugify`, CI gained `pymdown-extensions` in the same pull
#   request, and the dashboard copy could not. The next push to the default
#   branch built, failed, and deployed nothing, while the previous deployment
#   went on serving and said nothing was wrong.
#
#   That is a claim in one place falsified by a change in another with nothing
#   connecting them, which is the defect this repository exists to report. So
#   the dashboard field holds one line that names this file:
#
#       sh tools/cloudflare-build.sh
#
#   and every dependency below moves in a reviewed commit instead.
#
#   TWO DASHBOARD FIELDS NAME THIS FILE, NOT ONE. Workers Builds keeps a
#   build configuration for the default branch and a second one for every
#   other branch, and each carries its own build command. The second is what
#   runs on a pull request, and it deploys with `wrangler versions upload`
#   rather than `wrangler deploy`. Both read `wrangler.jsonc`, so both need
#   the asset directory that this script writes. A second field left empty
#   costs the live site nothing and fails every pull request's build.
#
# THE PINS, AND WHY THEY ARE THE SAME PINS CI USES
#
#   `.github/workflows/ci.yml` installs this same set before it runs
#   `mkdocs build --strict`, and the reasons are written there: `mkdocs` is
#   pinned because a site build is a projection and a projection that moves
#   without a commit is drift; `pymdown-extensions` is pinned because
#   `mkdocs.yml` names `pymdownx.slugs.slugify` as the rule behind every anchor,
#   so a release that moved that function would move thousands of anchors and
#   links with no commit here; `PyYAML` is named rather than left implicit
#   because a step reads `.headwater/taxonomy.lock` with it.
#
#   Keep this list and CI's identical. They install the same things for the
#   same reasons, and a divergence between them is the failure above repeating
#   itself in the other direction.
#
# WHAT THIS DOES NOT DO
#
#   It does not deploy. Cloudflare runs `npx wrangler deploy` afterwards, and
#   `wrangler.jsonc` names the directory that deploy serves. While that file
#   names `./site`, this script's output is built and not served — which is
#   deliberate, and HW-DR-0047 states the order in which that changes.
#
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"

echo "cloudflare-build.sh: installing the pinned site toolchain"
python3 -m pip install --break-system-packages \
    mkdocs==1.6.1 pymdown-extensions==10.11.2 PyYAML

echo "cloudflare-build.sh: rendering the generated half"
python3 -m mkdocs build --strict

echo "cloudflare-build.sh: composing the served directory"
sh tools/assemble-site.sh

echo "cloudflare-build.sh: done"
