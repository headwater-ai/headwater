#!/bin/sh
# fetch-apt.sh — copy the signed APT repository of the newest engine release
# into the served directory, so that https://headwater.tools/apt/ serves it.
#
# WHY HERE
#
#   The metadata is signed in `.github/workflows/release.yml`, because the
#   signing subkey is a secret there, and the release carries it as four
#   assets beside the package. The site reaches Cloudflare by one path only,
#   the Cloudflare build (HW-DR-0047), so this step reads those assets at
#   build time rather than a second job deploying them. HW-DR-0094 is the
#   decision.
#
# WHAT IT DOES WHEN THERE IS NOTHING TO SERVE
#
#   It fails open: one line, exit 0, and no `apt/` directory. That is the
#   state until the owner sets `APT_SIGNING_KEY` and a release is cut, and
#   it is also the state when GitHub cannot be reached. A site that builds
#   with no APT repository serves every other page, and a site build that
#   fails deploys nothing at all.
#
# The layout is the one `release.yml` signs: `Packages` names
# `pool/main/h/headwater/<deb>` and `Release` hashes
# `main/binary-amd64/Packages` under `dists/stable/`.
#
# Usage: sh tools/site/fetch-apt.sh <served directory>
set -eu

out=${1:?usage: fetch-apt.sh <served directory>}
repo=headwater-ai/headwater
api="https://api.github.com/repos/$repo/releases/latest"

stage=$(mktemp -d)
cleanup() { rm -rf "$stage"; }
trap cleanup EXIT

if ! curl -fsSL "$api" -o "$stage/release.json"; then
    echo "fetch-apt.sh: the newest release could not be read, so the site serves no APT repository"
    exit 0
fi
tag=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["tag_name"])' "$stage/release.json")
names=$(python3 -c 'import json,sys; print("\n".join(a["name"] for a in json.load(open(sys.argv[1]))["assets"]))' "$stage/release.json")
deb=$(printf '%s\n' "$names" | grep -E '^headwater_[^/]+_amd64\.deb$' | head -1 || true)
for name in InRelease Release Release.gpg Packages; do
    if ! printf '%s\n' "$names" | grep -qx "$name"; then
        echo "fetch-apt.sh: $tag carries no $name, so the site serves no APT repository"
        exit 0
    fi
done
if [ -z "$deb" ]; then
    echo "fetch-apt.sh: $tag carries no Debian package, so the site serves no APT repository"
    exit 0
fi

base="https://github.com/$repo/releases/download/$tag"
mkdir -p "$stage/apt/dists/stable/main/binary-amd64" "$stage/apt/pool/main/h/headwater"
for name in InRelease Release Release.gpg; do
    curl -fsSL "$base/$name" -o "$stage/apt/dists/stable/$name"
done
curl -fsSL "$base/Packages" -o "$stage/apt/dists/stable/main/binary-amd64/Packages"
# Cloudflare serves one asset of at most 25 MiB. `release.yml` refuses a
# larger package, and this line refuses one again, because a larger file
# fails the deploy rather than this build.
curl -fsSL "$base/$deb" -o "$stage/apt/pool/main/h/headwater/$deb"
bytes=$(wc -c <"$stage/apt/pool/main/h/headwater/$deb")
if [ "$bytes" -gt 26214400 ]; then
    echo "fetch-apt.sh: $deb is $bytes bytes, over the 25 MiB a site asset may be, so the site serves no APT repository"
    exit 0
fi
# The public keyring, when the owner has committed it (HW-DR-0094).
if [ -f site/apt/headwater-archive-keyring.asc ]; then
    cp site/apt/headwater-archive-keyring.asc "$stage/apt/"
fi
rm -rf "$out/apt"
cp -R "$stage/apt" "$out/apt"
echo "fetch-apt.sh: serving the APT repository of $tag with $deb"
