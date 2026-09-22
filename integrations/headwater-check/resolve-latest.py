#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
#
# Read the JSON body of `GET /repos/headwater-ai/headwater/releases` (newest
# first, which is the order that endpoint states) from standard input, and
# print the first tag in the engine's own release stream that carries both
# the archive and its checksum. Print nothing, and exit 0, when none does:
# the caller (`resolve-version.sh`) decides that a "latest" no release
# answers is a refusal, and this script's job is finding, not judging.
#
# Pulled out of `resolve-version.sh` as its own file so that
# `tools/repo/integrations-fixtures.sh` can hold this logic against the three
# shapes a real release list actually carries, with no network reachable and
# no corpus needed: an engine release with the asset, a `taxonomy/…` release
# (a different stream, never a match), and an engine release with no asset at
# all — `v0.1.1`, a real, not hypothetical, collision with an immutable
# release object that this repository's own release history carries.
import json
import sys


def resolve_latest(releases):
    for release in releases:
        tag = release.get("tag_name", "")
        # The engine stream alone: "v<digit>...", never "taxonomy/...".
        if "/" in tag or not tag.startswith("v"):
            continue
        if len(tag) < 2 or not tag[1].isdigit():
            continue
        names = {asset.get("name", "") for asset in release.get("assets", [])}
        want = f"headwater-{tag}-x86_64-unknown-linux-gnu.tar.gz"
        if want in names and f"{want}.sha256" in names:
            return tag
    return None


if __name__ == "__main__":
    releases = json.load(sys.stdin)
    tag = resolve_latest(releases)
    if tag:
        print(tag)
