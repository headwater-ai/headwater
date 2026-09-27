#!/bin/sh
# What holds `tools/site/fetch-apt.sh`.
#
# The script decides whether the site serves an APT repository, and a wrong
# decision either way costs adopters: served files of two releases fail
# `apt update` with "Hash Sum mismatch", and a repository dropped on a
# network error fails it with 404. Each case below serves a release from a
# local HTTP server, runs the script against it through
# `HEADWATER_APT_BASE`, and reads the exit status, the `apt/` it wrote and
# the line it printed. Two releases, A and B, are built with a throwaway gpg
# key, so `InRelease` is a real clearsigned file. The case "InRelease of A
# with the rest of B" is the one a re-verify of #764 found passing.
#
# Needs `gpg`, `python3`, `curl` and `sha256sum`. It writes only under a
# temporary directory.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
tool="$root/tools/site/fetch-apt.sh"
for need in gpg python3 curl sha256sum; do
    command -v "$need" >/dev/null || { echo "fetch-apt-fixtures.sh: needs \`$need\`" >&2; exit 1; }
done

scratch=$(mktemp -d) || exit 1
trap 'kill "$server" "$silent" 2>/dev/null; rm -rf "$scratch"' EXIT HUP INT TERM

passed=0
failed=0

# release NAME VERSION — a signed release as `release.yml` attaches it: the
# package and four metadata files, flat, in $scratch/rel/NAME.
export GNUPGHOME="$scratch/gpg"
mkdir -m 0700 "$GNUPGHOME"
gpg --batch --quiet --passphrase '' --quick-gen-key "fixture <fixture@invalid>" ed25519 sign never 2>/dev/null
release() {
    dir="$scratch/rel/$1"
    deb="headwater_$2_amd64.deb"
    mkdir -p "$dir"
    printf 'a package of %s\n' "$2" >"$dir/$deb"
    {
        printf 'Package: headwater\nVersion: %s\nArchitecture: amd64\n' "$2"
        printf 'Filename: pool/main/h/headwater/%s\n' "$deb"
        printf 'Size: %s\nSHA256: %s\n\n' "$(wc -c <"$dir/$deb" | tr -d ' ')" "$(sha256sum "$dir/$deb" | cut -d' ' -f1)"
    } >"$dir/Packages"
    {
        printf 'Suite: stable\nCodename: stable\nComponents: main\nArchitectures: amd64\n'
        # A line that opens with a dash, so the clearsign escapes it and the
        # script must undo the escape.
        printf -- '-Note: dash-escaped in InRelease\n'
        printf 'SHA256:\n %s %s main/binary-amd64/Packages\n' \
            "$(sha256sum "$dir/Packages" | cut -d' ' -f1)" "$(wc -c <"$dir/Packages" | tr -d ' ')"
    } >"$dir/Release"
    gpg --batch --yes --clearsign -o "$dir/InRelease" "$dir/Release"
    gpg --batch --yes --armor --detach-sign -o "$dir/Release.gpg" "$dir/Release"
}
release A 0.4.0
release B 0.5.0

# mix NAME FROM-A... — release B with the named files taken from release A.
mix() {
    name=$1
    shift
    mkdir -p "$scratch/rel/$name"
    cp "$scratch/rel/B/"* "$scratch/rel/$name/"
    for file in "$@"; do
        case "$file" in
            deb) rm "$scratch/rel/$name/headwater_0.5.0_amd64.deb"
                 cp "$scratch/rel/A/headwater_0.4.0_amd64.deb" "$scratch/rel/$name/headwater_0.5.0_amd64.deb" ;;
            *) cp "$scratch/rel/A/$file" "$scratch/rel/$name/$file" ;;
        esac
    done
}
mix inrelease-of-a InRelease
mix packages-of-a Packages
mix package-of-a deb
mkdir -p "$scratch/rel/unsigned"
cp "$scratch/rel/B/"* "$scratch/rel/unsigned/"
rm "$scratch/rel/unsigned/InRelease"
mkdir -p "$scratch/rel/no-release"
cp "$scratch/rel/B/"* "$scratch/rel/no-release/"
rm "$scratch/rel/no-release/Release"

# One server for every release, and one that accepts and never answers.
port=$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1",0)); print(s.getsockname()[1])')
python3 -m http.server "$port" --bind 127.0.0.1 --directory "$scratch/rel" >/dev/null 2>&1 &
server=$!
silent_port=$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1",0)); print(s.getsockname()[1])')
python3 -c '
import socket, sys, time
s = socket.socket(); s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
s.bind(("127.0.0.1", int(sys.argv[1]))); s.listen(8)
held = []
while True:
    held.append(s.accept()[0])
' "$silent_port" &
silent=$!
for _ in 1 2 3 4 5 6 7 8 9 10; do
    curl -s -o /dev/null "http://127.0.0.1:$port/" && break
    sleep 0.3
done

# case NAME BASE WANT-STATUS WANT-FILES REQUIRED-TEXT
case_() {
    out="$scratch/out-$1"
    mkdir -p "$out"
    (cd "$root" && HEADWATER_APT_BASE="$2" HEADWATER_APT_MAX_TIME=3 sh "$tool" "$out" >"$out.log" 2>&1)
    got=$?
    files=none
    [ -d "$out/apt" ] && files=$(find "$out/apt" -type f | wc -l | tr -d ' ')
    if [ "$got" = "$3" ] && [ "$files" = "$4" ] && grep -qF -- "$5" "$out.log"; then
        passed=$((passed + 1))
        echo "  ok    $1"
    else
        failed=$((failed + 1))
        echo "  FAIL  $1: exit $got (want $3), apt files $files (want $4)"
        sed 's/^/          /' "$out.log"
    fi
}

base="http://127.0.0.1:$port"
echo "fetch-apt.sh"
case_ "one signed release is served whole" "$base/B" 0 5 "serving the APT repository"
case_ "and apt finds each file where Packages and Release say" "$base/B" 0 5 "headwater_0.5.0_amd64.deb"
[ -f "$scratch/out-and apt finds each file where Packages and Release say/apt/pool/main/h/headwater/headwater_0.5.0_amd64.deb" ] &&
    [ -f "$scratch/out-and apt finds each file where Packages and Release say/apt/dists/stable/main/binary-amd64/Packages" ] &&
    cmp -s "$scratch/out-and apt finds each file where Packages and Release say/apt/dists/stable/InRelease" "$scratch/rel/B/InRelease" &&
    { passed=$((passed + 1)); echo "  ok      at the pool and dists paths, InRelease unchanged"; } ||
    { failed=$((failed + 1)); echo "  FAIL    at the pool and dists paths, InRelease unchanged"; }
case_ "a release with no InRelease serves nothing and passes" "$base/unsigned" 0 none "carries no InRelease"
case_ "InRelease of one release with the rest of another stops the build" "$base/inrelease-of-a" 1 none "the signed text of InRelease is not Release"
case_ "Packages of one release with the rest of another stops the build" "$base/packages-of-a" 1 none "Release does not carry the SHA-256 of Packages"
case_ "a package of one release with the rest of another stops the build" "$base/package-of-a" 1 none "Packages does not carry the SHA-256"
case_ "a missing Release stops the build rather than serving nothing" "$base/no-release" 1 none "Release answered HTTP 404"
case_ "a server nobody can reach stops the build" "http://127.0.0.1:1" 1 none "InRelease could not be downloaded"
start=$(date +%s)
case_ "a server that never answers stops the build" "http://127.0.0.1:$silent_port" 1 none "InRelease could not be downloaded"
took=$(( $(date +%s) - start ))
# Three seconds a try, four tries, and curl's own back-off between them.
if [ "$took" -le 40 ]; then
    passed=$((passed + 1)); echo "  ok      and gives up in ${took}s, not at the platform timeout"
else
    failed=$((failed + 1)); echo "  FAIL    and gives up in ${took}s"
fi

echo
echo "$passed passed, $failed failed"
[ "$failed" -eq 0 ]
