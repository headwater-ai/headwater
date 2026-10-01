#!/bin/sh
# preview-site.sh ALIAS — build https://headwater.tools/ from this commit and
# upload it as a preview version of the Worker, served at a URL named ALIAS.
# Production goes on serving its last deploy.
#
# WHO RUNS THIS
#
#   The `Preview the site` job of `.github/workflows/preview-site.yml`, on a
#   pull request from a branch of this repository that carries the
#   `site-preview` label. It reads `CLOUDFLARE_API_TOKEN` and
#   `CLOUDFLARE_ACCOUNT_ID` from the environment.
#
# WHAT IT DOES, IN ORDER
#
#   1. `tools/site/build-site.sh`, the build `tools/site/deploy-site.sh` runs,
#      so the preview carries the bytes a deploy of this commit would serve.
#   2. `npx wrangler versions upload --preview-alias ALIAS`, at the version
#      `deploy-site.sh` pins. An uploaded version takes no traffic until a
#      deploy names it, and this script names none. Cloudflare serves it at
#      `https://ALIAS-<worker>.<subdomain>.workers.dev`, and a second upload
#      under the same alias moves that URL to the newer version.
#   3. The alias URL wrangler printed, written to `.headwater/site-preview-url`
#      for the workflow to post on the pull request.
#
# `tools/repo/release-guide-fixtures.sh` holds that this script never runs
# `wrangler deploy` and that only a labelled pull request reaches it.
set -eu

alias=${1:-}
case $alias in
    '' | *[!a-z0-9-]* | -* | *-)
        echo "preview-site.sh: name an alias of lower-case letters, digits and inner hyphens, such as pr-1543" >&2
        exit 2
        ;;
esac

root=$(cd "$(dirname "$0")/../.." && pwd)
cd "$root"

# One pin for both scripts: the version the production deploy runs.
wrangler_version=$(sed -n 's/^WRANGLER_VERSION=\([0-9.]*\)$/\1/p' tools/site/deploy-site.sh)
if [ -z "$wrangler_version" ]; then
    echo "preview-site.sh: tools/site/deploy-site.sh pins no WRANGLER_VERSION" >&2
    exit 1
fi

sh tools/site/build-site.sh

echo "preview-site.sh: uploading a preview version under $alias with wrangler $wrangler_version"
log=.headwater/site-preview.log
npx --yes "wrangler@$wrangler_version" versions upload --preview-alias "$alias" \
    --message "site preview $alias at $(git rev-parse --short HEAD)" 2>&1 | tee "$log"
# `tee` hides wrangler's status, so read the URL it prints as the proof.
url=$(sed -n 's/.*Preview Alias URL: *\(https:[^[:space:]]*\).*/\1/p' "$log" | tail -n 1)
if [ -z "$url" ]; then
    echo "preview-site.sh: wrangler printed no preview alias URL. Preview URLs need the Worker's workers.dev route and preview_urls enabled" >&2
    exit 1
fi
printf '%s\n' "$url" > .headwater/site-preview-url
echo "preview-site.sh: $url"
