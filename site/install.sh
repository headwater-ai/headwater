#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
#
# Install the headwater binary:
#
#     curl -fsSL https://headwater.tools/install.sh | sh
#
# It downloads the release archive for this machine from GitHub, checks it
# against the checksum published beside it, and unpacks the one binary into
# ~/.local/bin. It needs `sh`, `curl`, `tar` and `uname`, and it uses
# `sha256sum` or `shasum` to check the archive when either is present. It
# never asks for root and writes nothing outside the install directory and
# one temporary directory that it removes.
#
# Two settings, both optional:
#
#     HEADWATER_VERSION       the release tag to install (default below)
#     HEADWATER_INSTALL_DIR   where the binary goes (default ~/.local/bin)
#
#     curl -fsSL https://headwater.tools/install.sh | HEADWATER_VERSION=v0.6.1 sh
#
# Run inside a repository whose corpus states the engine it asks for, as a
# range in the `engine:` line of `.headwater/taxonomy.yml`, it installs a
# release that range admits. It looks for that file in the current directory
# and in each directory above it, and the first one it finds decides. When the
# default release is in the range, it installs the default. When it is not, it
# reads the release list from the GitHub API and installs the newest release
# in the range that has an archive for this machine, and it refuses when there
# is none. It says which file chose the version. A release named in
# HEADWATER_VERSION is installed as named, with one line when it is outside the
# range.
#
# Every other route, apt and cargo among them, is at
# https://headwater.tools/install/.
#
# The body is one function called on the last line, so a download cut off
# part way runs nothing.
#
# For this repository: `site/install.sh` is served byte for byte at
# https://headwater.tools/install.sh. Step 9 of `docs/process/how-to/cut-a-release.md`
# moves the default tag below, and `tools/repo/readme-fixtures.sh` holds it to
# the release the README installs. The range is read here with `awk`, because
# the script is served alone and no engine exists yet. `release::satisfies` in
# the engine is the definition, and `tools/repo/readme-fixtures.sh` holds the
# `satisfies` function below to every row of
# `engine/crates/resolve/tests/engine-ranges.tsv`.

set -eu

main() {
    version=${HEADWATER_VERSION:-v0.6.1}
    dir=${HEADWATER_INSTALL_DIR:-$HOME/.local/bin}

    os=$(uname -s)
    arch=$(uname -m)
    case "$os $arch" in
        "Linux x86_64" | "Linux amd64") target=x86_64-unknown-linux-musl ;;
        "Darwin arm64" | "Darwin aarch64") target=aarch64-apple-darwin ;;
        *)
            say "no release archive is built for $os $arch."
            say "Every other route is at https://headwater.tools/install/."
            exit 1
            ;;
    esac

    # `mkdir` with no -p fails on a path that already exists, so nothing
    # placed there in advance is written through.
    work="${TMPDIR:-/tmp}/headwater-install.$$"
    mkdir "$work"
    trap 'rm -rf "$work"' EXIT
    trap 'exit 1' HUP INT TERM

    declaration=$(find_declaration)
    range=""
    if [ -n "$declaration" ]; then
        range=$(engine_range_of "$declaration")
    fi

    if [ -n "$range" ] && [ -n "${HEADWATER_VERSION:-}" ]; then
        if [ "$(readable "$range")" = error ]; then
            say "the engine range \"$range\" in $declaration cannot be read. $version is installed because HEADWATER_VERSION names it."
        elif [ "$(satisfies "$range" "${version#v}")" != yes ]; then
            say "$version is outside the engine range \"$range\" that $declaration asks for. It is installed because HEADWATER_VERSION names it."
        fi
        fetch "$version" || refuse_download
    elif [ -n "$range" ]; then
        if [ "$(readable "$range")" = error ]; then
            say "the engine range \"$range\" in $declaration cannot be read, so no release is chosen and nothing was installed."
            say "Write it as comparators, such as engine: \">=0.5 <0.6\", or name a release in HEADWATER_VERSION."
            exit 1
        fi
        if [ "$(satisfies "$range" "${version#v}")" = yes ]; then
            fetch "$version" || refuse_download
        else
            install_from_range
        fi
        say "$declaration asks for engine \"$range\", so this installs $version."
    else
        fetch "$version" || refuse_download
    fi
    curl -fsSL -o "$work/$archive.sha256" "$url.sha256"

    if command -v sha256sum >/dev/null 2>&1; then
        (cd "$work" && sha256sum -c "$archive.sha256" >/dev/null) || refuse_checksum
    elif command -v shasum >/dev/null 2>&1; then
        (cd "$work" && shasum -a 256 -c "$archive.sha256" >/dev/null) || refuse_checksum
    else
        say "neither sha256sum nor shasum is installed, so the archive is not checked."
    fi

    mkdir -p "$dir"
    tar -xzf "$work/$archive" -C "$dir" headwater
    say "installed $("$dir/headwater" --version) at $dir/headwater"

    case ":$PATH:" in
        *":$dir:"*) ;;
        *) say "$dir is not on your PATH. Add it, or run $dir/headwater by its full path." ;;
    esac
    say "Next: the tutorial at https://headwater.tools/tutorial/"
}

say() {
    printf 'headwater install: %s\n' "$1"
}

# fetch TAG: download the archive of release TAG for $target into $work, and
# set $version, $archive and $url to it. It fails when that archive does not
# download.
fetch() {
    version=$1
    archive="headwater-$version-$target.tar.gz"
    url="https://github.com/headwater-ai/headwater/releases/download/$version/$archive"
    say "downloading headwater $version for $target"
    curl -fsSL -o "$work/$archive" "$url"
}

refuse_download() {
    say "could not download $url"
    say "Check the version: the releases are at https://github.com/headwater-ai/headwater/releases."
    exit 1
}

# The first .headwater/taxonomy.yml in the current directory or above it.
find_declaration() {
    fd_dir=$PWD
    while :; do
        if [ -f "$fd_dir/.headwater/taxonomy.yml" ]; then
            printf '%s\n' "$fd_dir/.headwater/taxonomy.yml"
            return 0
        fi
        case "$fd_dir" in
            / | "") return 0 ;;
        esac
        fd_dir=$(dirname "$fd_dir")
    done
}

# The range in the first top-level `engine:` line of FILE, in double quotes,
# single quotes or none, or nothing when FILE states none. One byte order
# mark at the start of line 1 is dropped first, as the engine's loader drops
# it, so a range behind one is read (#1755). The mark is built with printf and
# matched byte for byte in the C locale, because macOS sed and dash read no
# `\xEF` escape.
engine_range_of() {
    er_bom=$(printf '\357\273\277')
    er_value=$(LC_ALL=C sed -n "1s/^$er_bom//; s/^engine:[[:space:]]*//p" "$1" | head -n 1 | tr -d '\r')
    case "$er_value" in
        \"*\"*)
            er_value=${er_value#\"}
            er_value=${er_value%%\"*}
            ;;
        \'*\'*)
            er_value=${er_value#\'}
            er_value=${er_value%%\'*}
            ;;
        *) er_value=$(printf '%s' "$er_value" | sed 's/[[:space:]][[:space:]]*#.*$//') ;;
    esac
    printf '%s' "$er_value" | sed 's/^[[:space:]]*//; s/[[:space:]]*$//'
}

# satisfies RANGE VERSION: print yes, no or error, as the engine's
# `release::satisfies` answers. A version is one to three dot-separated whole
# numbers below 2^64, and an absent part is zero. A range is comparators
# separated by blanks, each `>=`, `<=`, `>`, `<` or `=` and a version, and a
# bare version is `=`. Every comparator must hold, and the first that does not
# ends the reading. Numbers are compared as digit strings, so no number is
# rounded. With HW_EVERY=1 every comparator is read, so a comparator that
# cannot be read is an error wherever it stands.
satisfies() {
    HW_RANGE=$1 HW_VERSION=$2 LC_ALL=C awk '
        # The engine splits and trims on every Unicode white space character,
        # so each one, as UTF-8 bytes, becomes a space before awk splits.
        function blank(s,    i) {
            for (i = 1; i <= nblanks; i++) gsub(blanks[i], " ", s)
            return s
        }
        function trim(s) {
            s = blank(s)
            sub(/^ +/, "", s)
            sub(/ +$/, "", s)
            return s
        }
        function part(s) {
            s = "" s
            if (s !~ /^[+]?[0-9]+$/) return ""
            sub(/^[+]/, "", s)
            sub(/^0+/, "", s)
            if (s == "") s = "0"
            if (length(s) > 20 || (length(s) == 20 && s > "18446744073709551615")) return ""
            return s
        }
        function parse(v, out,    n, p, i) {
            v = trim(v)
            if (v == "") return 0
            n = split(v, p, ".")
            if (n > 3) return 0
            out[1] = "0"; out[2] = "0"; out[3] = "0"
            for (i = 1; i <= n; i++) {
                out[i] = part(p[i])
                if (out[i] == "") return 0
            }
            return 1
        }
        function order(a, b) {
            if (length(a) != length(b)) return length(a) < length(b) ? -1 : 1
            if (a == b) return 0
            return a < b ? -1 : 1
        }
        function compare(x, y,    i, c) {
            for (i = 1; i <= 3; i++) {
                c = order(x[i], y[i])
                if (c != 0) return c
            }
            return 0
        }
        BEGIN {
            nblanks = split("\011,\012,\013,\014,\015,\302\205,\302\240,\341\232\200,\342\200\200,\342\200\201,\342\200\202,\342\200\203,\342\200\204,\342\200\205,\342\200\206,\342\200\207,\342\200\210,\342\200\211,\342\200\212,\342\200\250,\342\200\251,\342\200\257,\342\201\237,\343\200\200", blanks, ",")
            every = ENVIRON["HW_EVERY"] == "1"
            if (!parse(ENVIRON["HW_VERSION"], target)) { print "error"; exit }
            n = split(blank(ENVIRON["HW_RANGE"]), terms, " ")
            answer = "yes"
            for (i = 1; i <= n; i++) {
                term = terms[i]
                op = "="; rest = term
                if (substr(term, 1, 2) == ">=" || substr(term, 1, 2) == "<=") {
                    op = substr(term, 1, 2); rest = substr(term, 3)
                } else if (substr(term, 1, 1) == ">" || substr(term, 1, 1) == "<" || substr(term, 1, 1) == "=") {
                    op = substr(term, 1, 1); rest = substr(term, 2)
                }
                if (!parse(rest, bound)) { print "error"; exit }
                c = compare(target, bound)
                held = (op == ">=" && c >= 0) || (op == ">" && c > 0) || (op == "<=" && c <= 0) || (op == "<" && c < 0) || (op == "=" && c == 0)
                if (!held) {
                    answer = "no"
                    if (!every) break
                }
            }
            if (n == 0) answer = "error"
            print answer
        }'
}

# readable RANGE: print error when some comparator of RANGE cannot be read.
readable() {
    HW_EVERY=1 satisfies "$1" 0
}

# Install the release with the highest version that the range admits and that
# has an archive for this machine. The GitHub API lists releases by publish
# date, so a patch release of an older line can come before a newer line, and
# the tags are sorted by version before the first fetch (#1756).
install_from_range() {
    api="https://api.github.com/repos/headwater-ai/headwater/releases?per_page=100"
    if ! curl -fsSL -H "Accept: application/vnd.github+json" -o "$work/releases.json" "$api"; then
        say "$declaration asks for engine \"$range\", which the default $version is outside, and the release list at $api could not be read."
        say "Name a release in the range with HEADWATER_VERSION."
        exit 1
    fi
    # One JSON member a line, so a tag is found however the body is laid out.
    # The engine's own stream alone: a `taxonomy/...` tag never opens with v.
    tr ',{}' '\n\n\n' <"$work/releases.json" |
        sed -n 's/^[[:space:]]*"tag_name"[[:space:]]*:[[:space:]]*"\(v[0-9][^"]*\)"[[:space:]]*$/\1/p' >"$work/tags"
    # Keep the tags the range admits, highest version first. `satisfies`
    # refuses a tag that is no version, so the sort sees numbers alone.
    : >"$work/admitted"
    while IFS= read -r ir_tag; do
        if [ "$(satisfies "$range" "${ir_tag#v}")" = yes ]; then
            printf '%s\n' "${ir_tag#v}" >>"$work/admitted"
        fi
    done <"$work/tags"
    sort -t. -k1,1nr -k2,2nr -k3,3nr "$work/admitted" >"$work/ordered"
    while IFS= read -r ir_version; do
        if fetch "v$ir_version"; then
            return 0
        fi
    done <"$work/ordered"
    say "no release has an archive for $target and satisfies the engine range \"$range\" that $declaration asks for, so nothing was installed."
    say "Widen the range, or name a release in HEADWATER_VERSION. The releases are at https://github.com/headwater-ai/headwater/releases."
    exit 1
}

refuse_checksum() {
    say "the archive does not match its published checksum, so nothing was installed."
    exit 1
}

main "$@"
