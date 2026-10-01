#!/bin/sh
# unpreview-site.sh NUMBER — take the site preview of pull request NUMBER off
# its URL. Production goes on serving its last deploy.
#
# WHO RUNS THIS
#
#   The `Remove the site preview` job of
#   `.github/workflows/unpreview-site.yml`, when a pull request that carried
#   the `site-preview` label closes, merged or not. A maintainer can also run
#   that workflow from the Actions tab for a preview whose pull request closed
#   before the job existed. It reads `CLOUDFLARE_API_TOKEN` and
#   `CLOUDFLARE_ACCOUNT_ID` from the environment.
#
# WHY IT UPLOADS RATHER THAN DELETES
#
#   Cloudflare has no call that deletes a preview alias. An alias is created
#   only by `wrangler versions upload --preview-alias`, and Cloudflare evicts
#   one only when a Worker passes 1,000 of them. So a merged pull request's
#   `pr-<number>` URL would serve its last build for as long as the account
#   lasts. This script uploads `tools/site/preview-gone/` under the same
#   alias, a version that answers 410 Gone on every path, and the alias moves
#   to it. Like each version `tools/site/preview-site.sh` uploads, it takes
#   no traffic.
#
# `tools/repo/release-guide-fixtures.sh` holds that this script reaches no
# deploy.
set -eu

number=${1:-}
case $number in
    '' | *[!0-9]* | 0*)
        echo "unpreview-site.sh: name a pull request by its number, such as 1545" >&2
        exit 2
        ;;
esac
alias="pr-$number"

root=$(cd "$(dirname "$0")/../.." && pwd)
cd "$root"

# One pin for the three scripts: the version the production deploy runs.
wrangler_version=$(sed -n 's/^WRANGLER_VERSION=\([0-9.]*\)$/\1/p' tools/site/deploy-site.sh)
if [ -z "$wrangler_version" ]; then
    echo "unpreview-site.sh: tools/site/deploy-site.sh pins no WRANGLER_VERSION" >&2
    exit 1
fi

mkdir -p .headwater
echo "unpreview-site.sh: uploading the 410 tombstone under $alias with wrangler $wrangler_version"
log=.headwater/site-unpreview.log
npx --yes "wrangler@$wrangler_version" versions upload \
    --config tools/site/preview-gone/wrangler.jsonc \
    --preview-alias "$alias" --var "PR_NUMBER:$number" \
    --message "site preview $alias removed" 2>&1 | tee "$log"
# `tee` hides wrangler's status, so read the URL it prints as the proof.
url=$(sed -n 's/.*Preview Alias URL: *\(https:[^[:space:]]*\).*/\1/p' "$log" | tail -n 1)
if [ -z "$url" ]; then
    echo "unpreview-site.sh: wrangler printed no preview alias URL, so $alias may still serve the preview" >&2
    exit 1
fi
printf '%s\n' "$url" > .headwater/site-preview-url
echo "unpreview-site.sh: $url now answers 410"
