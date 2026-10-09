#!/bin/sh
# Refuse a served site that holds a file Cloudflare Workers will not accept.
#
# Workers serves an asset of up to 25 MiB and `wrangler deploy` refuses the whole
# upload when one file is above that ("Asset too large"). On 2026-10-09 the
# search index reached 43.9 MiB after the campaign transcripts landed, and the
# failure surfaced on `main`, after the merge, as a red "Deploy the site".
# This runs on the assembled directory in the pull-request job, so the same
# file fails the pull request instead.
#
# The caller may lower the limit to keep a margin: a file at 24 MiB passes now
# and fails after the next shelf lands, and the log states the headroom.

set -u

if [ $# -ne 1 ]; then
    echo "usage: tools/site/check-site-asset-size.sh SITE-ROOT" >&2
    exit 2
fi

site_root=$1
limit_bytes=${HEADWATER_SITE_ASSET_LIMIT_BYTES:-26214400}

case "$limit_bytes" in
    ''|*[!0-9]*)
        echo "the asset limit must be a nonnegative integer count of bytes." >&2
        exit 2
        ;;
esac

if [ ! -d "$site_root" ]; then
    echo "no served site at \`$site_root\`. Assemble it before checking its asset sizes." >&2
    exit 2
fi

files=$(find "$site_root" -type f | wc -l | tr -d ' ')
if [ "$files" -eq 0 ]; then
    echo "no file under \`$site_root\`, so no asset size was checked." >&2
    exit 2
fi

over=$(find "$site_root" -type f -size +"${limit_bytes}c" -exec ls -l {} + | awk '{print $5 "\t" $NF}' | sort -rn)
largest=$(find "$site_root" -type f -exec ls -l {} + | awk '{print $5 "\t" $NF}' | sort -rn | head -n 1)

echo "largest served file: $largest (limit $limit_bytes bytes, $files files read)."

if [ -n "$over" ]; then
    echo "these served files are above the $limit_bytes-byte limit, and \`wrangler deploy\` refuses the upload:" >&2
    printf '%s\n' "$over" >&2
    echo "withhold the file's source from the build (for the search index, add the shelf to \`WITHHELD\` in \`mkdocs/hooks/search_scope.py\`)." >&2
    exit 1
fi
