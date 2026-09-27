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
# TWO KINDS OF "NOTHING TO SERVE", AND WHY THEY END DIFFERENTLY
#
#   A newest release with no `InRelease` is an answer: no release is signed
#   yet. The script prints one line, exits 0, and writes no `apt/` directory.
#   That is the state until the owner sets `APT_SIGNING_KEY` and cuts a
#   release.
#
#   A download that fails for any other reason is no answer at all: GitHub
#   unreachable, a 5xx, a rate limit. A deploy built then would drop a
#   repository adopters already use, and every `apt update` would get 404.
#   So the script exits 1 and the site build fails. A failed build deploys
#   nothing, and the previous deployment, with its `apt/`, goes on serving.
#
#   The files come from `releases/latest/download/<name>`, which is a
#   redirect to the asset and not the REST API, so the API's rate limit for
#   a shared builder address does not apply. `Packages` names the package,
#   and `Release` must carry the SHA-256 of that `Packages`, so the files
#   cannot come from two different releases.
#
# Usage: sh tools/site/fetch-apt.sh <served directory>
set -eu

out=${1:?usage: fetch-apt.sh <served directory>}
# `HEADWATER_APT_BASE` points the script at another server, for a test.
base=${HEADWATER_APT_BASE:-https://github.com/headwater-ai/headwater/releases/latest/download}

stage=$(mktemp -d)

# get NAME DEST — download one asset of the newest release. Prints the HTTP
# status. A transport failure after the retries is a failed build.
get() {
    if ! code=$(curl -sSL --retry 3 --retry-all-errors -o "$2" -w '%{http_code}' "$base/$1"); then
        echo "fetch-apt.sh: $1 could not be downloaded, so this build stops and the deployed site keeps its APT repository" >&2
        rm -rf "$stage"
        exit 1
    fi
    echo "$code"
}

code=$(get InRelease "$stage/InRelease")
if [ "$code" = 404 ]; then
    echo "fetch-apt.sh: the newest release carries no InRelease, so the site serves no APT repository"
    rm -rf "$stage"
    exit 0
fi
for name in InRelease Release Release.gpg Packages; do
    [ "$name" = InRelease ] || code=$(get "$name" "$stage/$name")
    if [ "$code" != 200 ]; then
        echo "fetch-apt.sh: $name answered HTTP $code, so this build stops and the deployed site keeps its APT repository" >&2
        rm -rf "$stage"
        exit 1
    fi
done

sum=$(sha256sum "$stage/Packages" | cut -d' ' -f1)
if ! grep -q " $sum .* main/binary-amd64/Packages\$" "$stage/Release"; then
    echo "fetch-apt.sh: Release does not carry the SHA-256 of Packages, so the two came from different releases; this build stops" >&2
    rm -rf "$stage"
    exit 1
fi
path=$(sed -n 's/^Filename: //p' "$stage/Packages" | head -1)
deb=${path##*/}
case "$path" in
    pool/main/h/headwater/headwater_*_amd64.deb) ;;
    *)
        echo "fetch-apt.sh: Packages names \`$path\`, which is not the pool path release.yml writes; this build stops" >&2
        rm -rf "$stage"
        exit 1
        ;;
esac

mkdir -p "$stage/apt/dists/stable/main/binary-amd64" "$stage/apt/pool/main/h/headwater"
mv "$stage/InRelease" "$stage/Release" "$stage/Release.gpg" "$stage/apt/dists/stable/"
mv "$stage/Packages" "$stage/apt/dists/stable/main/binary-amd64/Packages"
code=$(get "$deb" "$stage/apt/$path")
if [ "$code" != 200 ]; then
    echo "fetch-apt.sh: $deb answered HTTP $code, so this build stops and the deployed site keeps its APT repository" >&2
    rm -rf "$stage"
    exit 1
fi
# Cloudflare serves one asset of at most 25 MiB. `release.yml` refuses a
# larger package, and this line refuses one again, because a larger file
# fails the deploy rather than this build.
bytes=$(wc -c <"$stage/apt/$path")
if [ "$bytes" -gt 26214400 ]; then
    echo "fetch-apt.sh: $deb is $bytes bytes, over the 25 MiB a site asset may be; this build stops" >&2
    rm -rf "$stage"
    exit 1
fi
# The public keyring, when the owner has committed it (HW-DR-0094).
if [ -f site/apt/headwater-archive-keyring.asc ]; then
    cp site/apt/headwater-archive-keyring.asc "$stage/apt/"
fi
rm -rf "$out/apt"
cp -R "$stage/apt" "$out/apt"
rm -rf "$stage"
echo "fetch-apt.sh: serving the APT repository of the newest release with $deb"
