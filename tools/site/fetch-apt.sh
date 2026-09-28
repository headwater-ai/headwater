#!/bin/sh
# fetch-apt.sh — copy the signed APT repository of the newest engine release
# into the served directory, so that https://headwater.tools/apt/ serves it.
#
# WHY HERE
#
#   The metadata is signed in `.github/workflows/release.yml`, because the
#   signing subkey is a secret there, and the release carries it as four
#   assets beside the package. `tools/site/deploy-site.sh` calls this script
#   after it assembles the served directory. The main-push job of `ci.yml` and
#   the post-publish job of `release.yml` both use that script and the
#   `deploy-site` concurrency group (HW-DR-0047). HW-DR-0094 is the decision.
#
# TWO KINDS OF "NOTHING TO SERVE", AND WHY THEY END DIFFERENTLY
#
#   A newest release with no `InRelease` is an answer: no release is signed
#   yet. The script prints one line, exits 0, and writes no `apt/` directory.
#   That is the state until the owner sets `APT_SIGNING_KEY` and cuts a
#   release.
#
#   A download that fails for any other reason is no answer at all: GitHub
#   unreachable, a 5xx, a rate limit, a server that never answers. A deploy
#   built then would drop a repository adopters already use, and every
#   `apt update` would get 404. So the script exits 1 and the site build
#   fails. A failed build deploys nothing, and the previous deployment, with
#   its `apt/`, goes on serving.
#
#   The files come from `releases/latest/download/<name>`, which is a
#   redirect to the asset and not the REST API, so the API's rate limit for
#   a shared builder address does not apply.
#
# ONE RELEASE, NOT TWO
#
#   Each file is a separate download, so a release published between two of
#   them can hand this script files of two releases, and apt then refuses
#   the repository with "Hash Sum mismatch". Three checks chain the files to
#   one release: the signed text of `InRelease` is byte for byte `Release`,
#   `Release` carries the SHA-256 of `Packages`, and `Packages` carries the
#   SHA-256 of the package. `Release.gpg` is read only by an apt that
#   ignores `InRelease`, and this script has no key to check it with, so it
#   is the one file not chained. `tools/site/fetch-apt-fixtures.sh` holds
#   every case.
#
# TIME
#
#   The whole script has one deadline, `HEADWATER_APT_DEADLINE` seconds
#   from its start (600 by default). Each try of a download may take at most
#   `HEADWATER_APT_MAX_TIME` seconds (120 by default), and never more than
#   what is left of the deadline. A download is tried at most four times: a
#   transport failure and HTTP 408, 429, 500, 502, 503 and 504 are tried
#   again after one second, and every other answer is final. When the
#   deadline has passed, the next try is not started and the build stops.
#
#   So the worst case is the deadline plus the one-second pauses and the
#   local work: under 11 minutes with the defaults, whatever number of
#   files never answer. That is inside the 20 minutes Cloudflare Workers
#   Builds gives a build, so a silent server fails this build rather than
#   the platform's timeout. `tools/site/fetch-apt-fixtures.sh` holds the
#   bound with two silent files and a ten-second deadline.
#
#   The retries are this script's own loop and not `curl --retry`, because
#   curl's retries each get a full `--max-time` and so multiply it.
#
# Usage: sh tools/site/fetch-apt.sh <served directory>
set -eu

out=${1:?usage: fetch-apt.sh <served directory>}
# `HEADWATER_APT_BASE` points the script at another server, for a test.
base=${HEADWATER_APT_BASE:-https://github.com/headwater-ai/headwater/releases/latest/download}
max_time=${HEADWATER_APT_MAX_TIME:-120}
deadline=$(( $(date +%s) + ${HEADWATER_APT_DEADLINE:-600} ))

stage=$(mktemp -d)

# stop WHY — end this build with the deployed site left as it is.
stop() {
    echo "fetch-apt.sh: $1, so this build stops and the deployed site keeps its APT repository" >&2
    rm -rf "$stage"
    exit 1
}

# get NAME DEST — download one asset of the newest release, and print the
# HTTP status. Four tries at most, each inside the deadline. A transport
# failure on the last try, or a deadline that has passed, stops the build.
get() {
    try=1
    while :; do
        left=$(( deadline - $(date +%s) ))
        [ "$left" -gt 0 ] || stop "the deadline passed before $1 was downloaded"
        limit=$max_time
        [ "$left" -ge "$limit" ] || limit=$left
        if code=$(curl -sSL --connect-timeout 20 --max-time "$limit" -o "$2" -w '%{http_code}' "$base/$1"); then
            case "$code" in
                408 | 429 | 500 | 502 | 503 | 504) ;;
                *) echo "$code"; return 0 ;;
            esac
        fi
        [ "$try" -lt 4 ] || stop "$1 could not be downloaded"
        try=$((try + 1))
        sleep 1
    done
}

code=$(get InRelease "$stage/InRelease")
if [ "$code" = 404 ]; then
    echo "fetch-apt.sh: the newest release carries no InRelease, so the site serves no APT repository"
    rm -rf "$stage"
    exit 0
fi
for name in InRelease Release Release.gpg Packages; do
    [ "$name" = InRelease ] || code=$(get "$name" "$stage/$name")
    [ "$code" = 200 ] || stop "$name answered HTTP $code"
done

# The signed text of a clearsigned file: the lines after the blank line that
# ends its armor header, up to the signature, with dash-escaping undone.
awk '
    /^-----BEGIN PGP SIGNATURE-----$/ { exit }
    body { sub(/^- /, ""); print; next }
    armor && /^$/ { body = 1; next }
    /^-----BEGIN PGP SIGNED MESSAGE-----$/ { armor = 1 }
' "$stage/InRelease" >"$stage/InRelease.text"
cmp -s "$stage/InRelease.text" "$stage/Release" ||
    stop "the signed text of InRelease is not Release, so the two came from different releases"
sum=$(sha256sum "$stage/Packages" | cut -d' ' -f1)
grep -q " $sum .* main/binary-amd64/Packages\$" "$stage/Release" ||
    stop "Release does not carry the SHA-256 of Packages, so the two came from different releases"

path=$(sed -n 's/^Filename: //p' "$stage/Packages" | head -1)
deb=${path##*/}
case "$path" in
    pool/main/h/headwater/headwater_*_amd64.deb) ;;
    *) stop "Packages names \`$path\`, which is not the pool path release.yml writes" ;;
esac
debsum=$(sed -n 's/^SHA256: //p' "$stage/Packages" | head -1)

mkdir -p "$stage/apt/dists/stable/main/binary-amd64" "$stage/apt/pool/main/h/headwater"
mv "$stage/InRelease" "$stage/Release" "$stage/Release.gpg" "$stage/apt/dists/stable/"
mv "$stage/Packages" "$stage/apt/dists/stable/main/binary-amd64/Packages"
code=$(get "$deb" "$stage/apt/$path")
[ "$code" = 200 ] || stop "$deb answered HTTP $code"
[ "$(sha256sum "$stage/apt/$path" | cut -d' ' -f1)" = "$debsum" ] ||
    stop "Packages does not carry the SHA-256 of $deb, so the two came from different releases"
# Cloudflare serves one asset of at most 25 MiB. `release.yml` refuses a
# larger package, and this line refuses one again, because a larger file
# fails the deploy rather than this build.
bytes=$(wc -c <"$stage/apt/$path")
[ "$bytes" -le 26214400 ] || stop "$deb is $bytes bytes, over the 25 MiB a site asset may be"
# The public keyring, when the owner has committed it (HW-DR-0094).
if [ -f site/apt/headwater-archive-keyring.asc ]; then
    cp site/apt/headwater-archive-keyring.asc "$stage/apt/"
fi
rm -rf "$out/apt"
cp -R "$stage/apt" "$out/apt"
rm -rf "$stage"
echo "fetch-apt.sh: serving the APT repository of the newest release with $deb"
