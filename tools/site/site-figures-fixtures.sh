#!/bin/sh
# What holds the figures a person typed on a hand-built page.
#
# `tools/site/refresh-figures.sh` fills every `data-figure` element when the
# site is published, and the tutorial page is rendered from its document. The
# review of #1489 read every hand-built page for a number outside those two,
# and found three that move when the repository moves and that nothing held:
#
#   1. The engine release, as `vX.Y.Z` or `version X.Y.Z`, on `site/index.html`,
#      `site/install/index.html` and `site/proof/index.html`. It must be the
#      release that `README.md` checks out (`git checkout vX.Y.Z`), which is
#      the line a release moves first. Step 9 of the release guide finds these
#      lines, and `tools/repo/release-guide-fixtures.sh` holds that its search
#      finds them, but nothing failed when a page still named the previous
#      release. `site/proof/index.html` was outside that search entirely.
#   2. The Rust floor that `site/install/index.html` states for a build from
#      source. It must be `rust-version` in `engine/Cargo.toml`.
#   3. The two platforms the pages name, "Linux x86_64" and "macOS on Apple
#      silicon". Each must be a target of the matrix in
#      `.github/workflows/release.yml`.
#
# The review record under `docs/reviews/` names this file for each of them,
# and names what holds every other figure on the nine pages.
#
# Run it from anywhere:
#     sh tools/site/site-figures-fixtures.sh
#
# It needs no browser. It writes only under `mktemp -d`.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
pages="site/index.html site/install/index.html site/proof/index.html"

# audit ROOT — prints one finding per figure that disagrees with its source,
# and one when a source or the population is missing. Prints nothing when
# every figure agrees.
audit() {
    tag=$(sed -n 's/^git checkout v\([0-9][0-9.]*\)[[:space:]]*$/\1/p' "$1/README.md" 2>/dev/null | head -n 1)
    if [ -z "$tag" ]; then
        echo "README.md has no \`git checkout v<version>\` line, so no release can be held"
    else
        named=0
        for page in $pages; do
            if [ ! -f "$1/$page" ]; then
                echo "$page is not there"
                continue
            fi
            for found in $(grep -noE 'v[0-9]+\.[0-9]+\.[0-9]+|[Vv]ersion `?[0-9]+\.[0-9]+\.[0-9]+' "$1/$page" |
                sed 's/[Vv]ersion `\{0,1\}/v/' | tr -d ' '); do
                named=$((named + 1))
                line=${found%%:*}
                version=${found#*:v}
                if [ "$version" != "$tag" ]; then
                    echo "$page:$line names release $version, and README.md checks out v$tag"
                fi
            done
        done
        # A count of zero is not agreement. A page that lost every release
        # line, or a pattern that matches nothing, would print no finding.
        if [ "$named" -eq 0 ]; then
            echo "no page of $pages names a release, so nothing was held"
        fi
    fi

    floor=$(sed -n 's/^rust-version = "\([0-9.]*\)".*/\1/p' "$1/engine/Cargo.toml" 2>/dev/null | head -n 1)
    stated=$(grep -oE 'toolchain at [0-9]+\.[0-9]+' "$1/site/install/index.html" 2>/dev/null | head -n 1 | sed 's/toolchain at //')
    if [ -z "$floor" ] || [ -z "$stated" ]; then
        echo "the Rust floor is not stated in both engine/Cargo.toml ($floor) and site/install/index.html ($stated)"
    elif [ "$floor" != "$stated" ]; then
        echo "site/install/index.html states Rust $stated, and engine/Cargo.toml declares rust-version $floor"
    fi

    for pair in "Linux x86_64:x86_64-unknown-linux-musl" "macOS on Apple silicon:aarch64-apple-darwin"; do
        words=${pair%%:*}
        target=${pair#*:}
        if grep -qF "$words" "$1/site/index.html" "$1/site/install/index.html" 2>/dev/null &&
            ! grep -qE "target: $target\$" "$1/.github/workflows/release.yml" 2>/dev/null; then
            echo "the pages name \"$words\", and .github/workflows/release.yml builds no $target"
        fi
    done
}

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

passed=0
failed=0

# expect NAME WANT GOT — WANT is a substring of GOT, or empty for "no finding".
expect() {
    if [ -z "$2" ]; then
        if [ -z "$3" ]; then ok=yes; else ok=no; fi
    else
        case "$3" in *"$2"*) ok=yes ;; *) ok=no ;; esac
    fi
    if [ "$ok" = yes ]; then
        passed=$((passed + 1))
        echo "  ok    $1"
    else
        failed=$((failed + 1))
        echo "  FAIL  $1"
        echo "          wanted: ${2:-no finding}"
        echo "          got:    ${3:-no finding}"
    fi
}

copy() {
    rm -rf "$1"
    for f in README.md engine/Cargo.toml .github/workflows/release.yml $pages; do
        mkdir -p "$1/$(dirname "$f")"
        cp "$root/$f" "$1/$f"
    done
}

echo "the real tree"

# 1. Every figure agrees with its source, over a population that is not empty.
expect "every release, the Rust floor and both platforms agree with their source" "" \
    "$(audit "$root")"

echo "a page that lags the release"

# 2. The proof page names the previous release. It was outside the release
#    guide's search, so this is the case that search never covered.
copy "$scratch/lag"
sed -i 's/v\([0-9]*\)\.\([0-9]*\)\.\([0-9]*\) carries/v0.0.1 carries/' "$scratch/lag/site/proof/index.html"
expect "a proof page naming another release is found" \
    "site/proof/index.html:" "$(audit "$scratch/lag")"
expect "  and the finding names both releases" "names release 0.0.1, and README.md checks out v" \
    "$(audit "$scratch/lag")"

# 3. A release moves README.md first. Every page line that still names the
#    old one is a finding, on the install page as well.
copy "$scratch/moved"
sed -i 's/^git checkout v[0-9.]*$/git checkout v9.9.9/' "$scratch/moved/README.md"
expect "a release that moved and an install page that did not is found" \
    "site/install/index.html:" "$(audit "$scratch/moved")"

echo "the Rust floor"

# 4. The workspace raises its floor and the install page does not.
copy "$scratch/floor"
sed -i 's/^rust-version = "[0-9.]*"/rust-version = "1.99"/' "$scratch/floor/engine/Cargo.toml"
expect "a raised floor the install page does not state is found" \
    "states Rust" "$(audit "$scratch/floor")"

echo "the platforms"

# 5. The release stops building for Apple silicon and the pages still name it.
copy "$scratch/arm"
sed -i '/target: aarch64-apple-darwin$/d' "$scratch/arm/.github/workflows/release.yml"
expect "a platform the release no longer builds is found" \
    "builds no aarch64-apple-darwin" "$(audit "$scratch/arm")"

echo "the assumptions this states rather than reporting zero over"

# 6. No release named anywhere is not agreement.
copy "$scratch/none"
for page in $pages; do
    sed -i 's/v[0-9]*\.[0-9]*\.[0-9]*//g; s/[Vv]ersion [0-9.]*//g' "$scratch/none/$page"
done
expect "pages that name no release are refused, not passed" \
    "nothing was held" "$(audit "$scratch/none")"

# 7. No README.md tag is not agreement either.
copy "$scratch/notag"
sed -i '/^git checkout v/d' "$scratch/notag/README.md"
expect "a README.md with no release line is refused" \
    "no release can be held" "$(audit "$scratch/notag")"

echo
echo "$passed passed, $failed failed"
[ "$failed" -eq 0 ]
