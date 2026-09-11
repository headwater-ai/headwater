#!/bin/sh
# Refuse a served site that carries MkDocs's stock footer.
#
# An absence check over no pages is not a pass. The caller names a minimum so
# its log states both the forbidden-string numerator and the population it read.

set -u

if [ $# -ne 1 ]; then
    echo "usage: tools/site/check-site-footer.sh SITE-ROOT" >&2
    exit 2
fi

site_root=$1
minimum_pages=${HEADWATER_SITE_FOOTER_MINIMUM_PAGES:-300}
forbidden='Documentation built with MkDocs'

case "$minimum_pages" in
    ''|*[!0-9]*)
        echo "the minimum served-page count must be a nonnegative integer." >&2
        exit 2
        ;;
esac

if [ ! -d "$site_root" ]; then
    echo "no served site at \`$site_root\`. Assemble it before checking its footer." >&2
    exit 2
fi

pages=$(find "$site_root" -type f -name '*.html' | wc -l | tr -d ' ')
matches=$(grep -rlF --include='*.html' -- "$forbidden" "$site_root" || true)
if [ -n "$matches" ]; then
    forbidden_pages=$(printf '%s\n' "$matches" | wc -l | tr -d ' ')
else
    forbidden_pages=0
fi

echo "$forbidden_pages forbidden footer strings out of $pages served HTML pages (minimum $minimum_pages)."

if [ "$pages" -lt "$minimum_pages" ]; then
    echo "the served-page denominator is $pages, below the stated floor of $minimum_pages." >&2
    exit 2
fi

if [ "$forbidden_pages" -ne 0 ]; then
    echo "pages carrying \`$forbidden\`:" >&2
    printf '%s\n' "$matches" >&2
    exit 1
fi
