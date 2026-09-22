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
set -eu

case "$REQUESTED" in
    latest)
        api="https://api.github.com/repos/headwater-ai/headwater/releases?per_page=30"
        auth=""
        if [ -n "${GH_TOKEN:-}" ]; then
            auth="Authorization: Bearer $GH_TOKEN"
        fi
        if [ -n "$auth" ]; then
            body=$(curl -fsSL -H "Accept: application/vnd.github+json" -H "$auth" "$api")
        else
            body=$(curl -fsSL -H "Accept: application/vnd.github+json" "$api")
        fi
        tag=$(printf '%s' "$body" | python3 "$(dirname "$0")/resolve-latest.py")
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
