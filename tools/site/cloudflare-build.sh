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
#       sh tools/site/cloudflare-build.sh
#
#   and every dependency below moves in a reviewed commit instead.
#
#   TWO CONFIGURATIONS NAME THIS FILE, AND THE DASHBOARD SHOWS ONE OF THEM.
#   Workers Builds keeps a build configuration for the default branch and a
#   second one for every other branch, and each carries its own build command.
#   The second is what runs on a pull request, and it deploys with
#   `wrangler versions upload` rather than `wrangler deploy`. Both read
#   `wrangler.jsonc`, so both need the asset directory that this script
#   writes, and a second one left empty costs the live site nothing and fails
#   every pull request's build.
#
#   The Builds settings page carries one `Build command` field, and that field
#   writes the first configuration only. There is no second field and no
#   preview settings page. The second configuration is reachable through the
#   API that page itself calls:
#
#       GET   /accounts/<account>/builds/workers/<script_tag>
#       PATCH /accounts/<account>/builds/workers/<script_tag>
#             {"previews_base_config": {…, "build_command": "sh tools/site/cloudflare-build.sh"}}
#
#   Send `previews_base_config` back as the GET returned it with that one key
#   changed, and omit `production_settings` so the live path is untouched.
#   Read it back afterwards, because nothing else reports what it holds:
#   a build record names its own build command, so a green preview build on a
#   branch cut after the asset directory changed is the only proof that lands.
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
# THIS IS NO LONGER THE DEPLOY PATH (#1273)
#
#   The site is deployed by the `Deploy the site` job of
#   `.github/workflows/deploy-site.yml`, which runs `tools/site/deploy-site.sh`.
#   `ci.yml` calls it on a push to `main` after every gating job is green, and
#   `release.yml` calls it after it creates a release. That script measures the
#   figures into the assembled pages, and nothing here can: a Workers Build has
#   no built engine, and it starts on the push rather than after CI.
#
#   So the committed pages carry every figure blank, and this script refuses
#   the assembled directory with `tools/site/check-site-figures.sh` before
#   Cloudflare deploys it. While the Workers Builds git integration stays
#   connected, every build it runs goes red and publishes nothing, which is the
#   intended outcome: a red build in the Cloudflare dashboard rather than a
#   live page with holes where the figures belong. Disconnect the repository
#   in the Worker's Builds settings, and this file has no caller.
#
set -eu

root=$(cd "$(dirname "$0")/../.." && pwd)
cd "$root"

echo "cloudflare-build.sh: installing the pinned site toolchain"
python3 -m pip install --break-system-packages \
    mkdocs==1.6.1 pymdown-extensions==10.11.2 PyYAML

echo "cloudflare-build.sh: rendering the generated half"
python3 -m mkdocs build --strict

echo "cloudflare-build.sh: composing the served directory"
sh tools/site/assemble-site.sh

echo "cloudflare-build.sh: refusing a blank figure, which only the CI deploy fills"
sh tools/site/check-site-figures.sh .headwater/site-deploy

echo "cloudflare-build.sh: adding the signed APT repository of the newest release"
sh tools/site/fetch-apt.sh .headwater/site-deploy

echo "cloudflare-build.sh: done"
