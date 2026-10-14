#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
#
# Resolve `$REQUESTED` (the `version` input) to a release tag, and write it to
# `$GITHUB_OUTPUT` as `tag`.
#
# `latest` does not mean "the newest tag this repository has cut" — this
# repository also cuts `taxonomy/headwater-standard/v<version>` tags, on a
# release stream of their own, and GitHub's own `/releases/latest` endpoint
# answers with whichever release published most recently across every tag
# namespace, taxonomy releases included. It has already answered with the
# wrong stream once in this repository's own history: the taxonomy release
# `taxonomy/headwater-standard/v4.2.0` published two hours before `v0.1.2`,
# and would be "latest" by publish time alone on a day nobody has cut a newer
# engine tag since. A second, narrower failure sits beside it: `v0.1.1` is a
# real tag with no release object at all (a collision with an immutable
# release, per `.github/workflows/release.yml`'s own account of it), so even
# a query scoped to the `v*` stream cannot stop at the newest tag by name; it
# has to confirm the asset this action needs is actually attached.
#
# So this walks the release list, newest first, filters to the engine's own
# stream (a tag matching `v<digit>…`, which a `taxonomy/…` tag never does),
# and stops at the first one that carries both the archive and its checksum.
#
# A corpus can state the engine it asks for, as a range in the `engine:` line
# of `$ROOT/.headwater/taxonomy.yml` (#1600). When it does, `latest` stops at
# the first such release whose version the range admits, so a corpus that
# pins an older release is checked by that release and not by the newest. A
# range no listed release satisfies is a refusal that names the range and the
# file, never a fall back to the newest. The line is read before any engine
# exists, so `resolve-latest.py` reads it and evaluates the range itself;
# `engine/crates/resolve/tests/engine-ranges.tsv` holds its reading to the
# engine's. A tag named in `version` is installed as named, and the engine
# reports a range it is outside when it runs.
#
# The list is asked for 100 releases a page, the most GitHub returns. Each
# engine release and each taxonomy release takes one place in it, and a corpus
# that pins an older engine must still find that release as the list grows.
set -eu

declaration="${ROOT:-.}/.headwater/taxonomy.yml"
range=""
if [ -f "$declaration" ]; then
    range=$(python3 "$(dirname "$0")/resolve-latest.py" --engine-line "$declaration")
fi

case "$REQUESTED" in
    latest)
        api="https://api.github.com/repos/headwater-ai/headwater/releases?per_page=100"
        auth=""
        if [ -n "${GH_TOKEN:-}" ]; then
            auth="Authorization: Bearer $GH_TOKEN"
        fi
        if [ -n "$auth" ]; then
            body=$(curl -fsSL -H "Accept: application/vnd.github+json" -H "$auth" "$api")
        else
            body=$(curl -fsSL -H "Accept: application/vnd.github+json" "$api")
        fi
        if [ -n "$range" ]; then
            if tag=$(printf '%s' "$body" | python3 "$(dirname "$0")/resolve-latest.py" --range "$range"); then
                :
            else
                echo "::error::the engine range \"$range\" in $declaration cannot be read, so 'latest' resolves to nothing. Write it as comparators such as \">=0.5 <0.6\"." >&2
                exit 1
            fi
            if [ -z "$tag" ]; then
                echo "::error::no release of headwater-ai/headwater satisfies the engine range \"$range\" that $declaration asks for and carries a headwater-<tag>-x86_64-unknown-linux-gnu.tar.gz asset, so 'latest' resolves to nothing. Widen the range, or pin 'version' to a tag." >&2
                exit 1
            fi
            echo "the engine range \"$range\" in $declaration chose $tag"
        else
            tag=$(printf '%s' "$body" | python3 "$(dirname "$0")/resolve-latest.py")
        fi
        if [ -z "$tag" ]; then
            echo "::error::no release of headwater-ai/headwater in the v* stream carries a headwater-<tag>-x86_64-unknown-linux-gnu.tar.gz asset, so 'latest' resolves to nothing. Pin 'version' to a known-good tag instead." >&2
            exit 1
        fi
        ;;
    v*)
        tag="$REQUESTED"
        ;;
    *)
        echo "::error::version '$REQUESTED' is not 'latest' and not a tag of the form v<version>" >&2
        exit 1
        ;;
esac

echo "tag=$tag" >>"$GITHUB_OUTPUT"
echo "resolved $REQUESTED to $tag"
