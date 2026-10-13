#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
#
# readme-install-block.sh README ROUTE — print the commands of one install
# route of the page's `## Obtaining a named version` section, as a reader runs
# them (#1490).
#
# `.github/workflows/install-routes.yml` runs what this script prints, on a
# schedule, and it carries no copy of a command. Group 14 of
# `tools/repo/readme-fixtures.sh` holds that every route the page documents is
# a route here and a job there.
#
# The routes, each the first fence after the paragraph whose opening words are
# given, unless the line says otherwise:
#
#   archive        `**Download the binary`
#   archive-macos  the archive fence, with the one substitution the archive
#                  paragraph states: "put `A` where the block says `B`"
#   apt            `**Install it with apt`, through `readme-apt-block.sh`,
#                  which `.github/workflows/readme-apt.yml` runs
#   deb-file       the Debian 11 command of the APT paragraph,
#                  `sudo apt-get install ./headwater_<version>_amd64.deb`, less
#                  `sudo`, with `<version>` the version of the tag the archive
#                  fence downloads
#   cargo          `**The alternative, if you already have a Rust toolchain`
#   source         `**A fixed version built from source`
#   release        not a route: the release download directory the archive
#                  fence names, `https://github.com/<owner>/<repo>/releases/download/<tag>`
#
# `readme-install-block.sh --routes` prints each route that reads a fence, as
# `<route> <opening words>`, so that group 14 can match the page's fenced
# paragraphs against this table rather than a second copy of it.
#
# It exits 1 and prints nothing on standard output when the paragraph, its
# fence or the sentence a route reads is missing, or the fence is empty. The
# workflow depends on that refusal, because `sh -e` of an empty script exits 0,
# so a paragraph that moved would turn the job green while it ran nothing.

set -u

here=$(cd "$(dirname "$0")" && pwd)

routes() {
    printf '%s\n' \
        'archive **Download the binary' \
        'apt **Install it with apt' \
        'cargo **The alternative, if you already have a Rust toolchain' \
        'source **A fixed version built from source'
}

if [ "${1:-}" = "--routes" ]; then
    routes
    exit 0
fi

if [ $# -ne 2 ]; then
    echo "usage: readme-install-block.sh README ROUTE | --routes" >&2
    exit 2
fi

readme=$1
route=$2

refuse() {
    echo "readme-install-block: $readme: $1" >&2
    exit 1
}

[ -f "$readme" ] || refuse "no such file"

# fence OPENER — the lines of the fence that follows the paragraph that opens
# with OPENER, inside `## Obtaining a named version`, with only blank lines
# between the two. The fence test is the one `readme-apt-block.sh` uses. A
# paragraph whose fence is gone prints nothing, rather than the fence of the
# next paragraph.
fence() {
    awk -v opener="$1" '
        /^## / { section = ($0 == "## Obtaining a named version"); next }
        !section { next }
        !seen && index($0, opener) == 1 { seen = 1; next }
        !seen { next }
        /^[ \t]*```/ { if (fence) exit; fence = 1; next }
        fence { print; next }
        /[^ \t]/ { exit }
    ' "$readme"
}

# paragraph OPENER — the one line of the paragraph that opens with OPENER.
paragraph() {
    awk -v opener="$1" '
        /^## / { section = ($0 == "## Obtaining a named version"); next }
        section && index($0, opener) == 1 { print; exit }
    ' "$readme"
}

opener_of() {
    routes | awk -v r="$1" '$1 == r { sub(/^[^ ]+ /, ""); print; exit }'
}

nonempty() {
    [ -n "$(printf '%s' "$1" | tr -d ' \t\n')" ]
}

# release — the download directory the archive fence names.
release() {
    fence "$(opener_of archive)" |
        sed -n 's|.*\(https://github\.com/[^/ ]*/[^/ ]*/releases/download/[^/ ]*\)/.*|\1|p' |
        head -n 1
}

case $route in
    archive | cargo | source)
        block=$(fence "$(opener_of "$route")")
        nonempty "$block" || refuse "no fence with commands after the paragraph that opens \`$(opener_of "$route")\`"
        printf '%s\n' "$block"
        ;;
    archive-macos)
        block=$(fence "$(opener_of archive)")
        nonempty "$block" || refuse "no fence with commands after the paragraph that opens \`$(opener_of archive)\`"
        line=$(paragraph "$(opener_of archive)")
        to=$(printf '%s\n' "$line" | sed -n 's/.*put `\([^`]*\)` where the block says `\([^`]*\)`.*/\1/p')
        from=$(printf '%s\n' "$line" | sed -n 's/.*put `\([^`]*\)` where the block says `\([^`]*\)`.*/\2/p')
        [ -n "$to" ] && [ -n "$from" ] ||
            refuse "the archive paragraph states no substitution of the form \"put \`A\` where the block says \`B\`\""
        out=$(printf '%s\n' "$block" | awk -v from="$from" -v to="$to" '
            {
                rest = $0; line = ""
                while ((i = index(rest, from)) > 0) {
                    line = line substr(rest, 1, i - 1) to
                    rest = substr(rest, i + length(from))
                }
                print line rest
            }')
        [ "$out" != "$block" ] || refuse "the archive fence does not contain \`$from\`, so the substitution changes nothing"
        printf '%s\n' "$out"
        ;;
    apt)
        exec sh "$here/readme-apt-block.sh" "$readme"
        ;;
    deb-file)
        cmd=$(paragraph "$(opener_of apt)" |
            sed -n 's/.*`\(sudo apt-get install \.\/headwater_<version>_[a-z0-9]*\.deb\)`.*/\1/p' |
            sed 's/sudo //')
        [ -n "$cmd" ] || refuse "the APT paragraph gives no \`sudo apt-get install ./headwater_<version>_<arch>.deb\` command"
        dir=$(release)
        [ -n "$dir" ] || refuse "the archive fence names no release download directory"
        # The version the release workflow writes into the package's name:
        # the tag less its `v`, with a pre-release `-` as `~`.
        version=$(basename "$dir" | sed 's/^v//; s/-/~/')
        printf '%s\n' "$cmd" | sed "s/<version>/$version/"
        ;;
    release)
        dir=$(release)
        [ -n "$dir" ] || refuse "the archive fence names no release download directory"
        printf '%s\n' "$dir"
        ;;
    *)
        echo "readme-install-block: no route \`$route\`; the routes are archive, archive-macos, apt, deb-file, cargo, source and release" >&2
        exit 2
        ;;
esac
