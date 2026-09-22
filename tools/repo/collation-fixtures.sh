#!/bin/sh
# What holds every `sort`-fed `comm` under `tools/`, `.githooks/` and
# `.claude/`: the comparison collates the same way its input was ordered.
#
# Run it from anywhere:
#     sh tools/repo/collation-fixtures.sh
#
# # THE DEFECT THIS SUITE EXISTS FOR
#
# `comm` requires both of its inputs to already be in `comm`'s own notion of
# sorted order, and that notion does not always agree with a locale-aware
# `sort` run under the ambient environment — verified by hand on more than
# one host: a `sort` that `sort -c` itself accepts as correctly ordered can
# still be one `comm` rejects, silently, on standard error, while its answer
# on standard output is either an empty diff over a population that is not
# really equal or a diff that names an entry wrongly (#828). Nine scripts
# under `tools/` paired a `sort` with a `comm` on 2026-09-22; one pinned
# `LC_ALL=C` on both throughout, seven pinned nothing, and one pinned it on
# only the newest of its call sites. All nine now pin `LC_ALL=C` on every
# `sort` that feeds a `comm` and on every `comm` itself, matching the
# convention `tools/site/assemble-site.sh` already carried.
#
# # WHAT THIS SUITE HOLDS, AND HOW IT IS ENUMERATED
#
# THE POPULATION is not a list of the nine files above. It is every regular
# file under `tools/`, `.githooks/` and `.claude/` whose text carries the
# word `comm` outside of a comment line, read fresh with `grep -rl` on every
# run. A tenth script added next month that pairs `sort` with `comm` enters
# this population the same way the first nine did, without this file being
# reopened to name it.
#
# THE CASE is that every invocation of `comm`, and every invocation of
# `sort` that is not provably unrelated to one — this suite does not trace
# which scratch file a `comm` call reads, because that would make it a second
# copy of the logic each of the nine fixture suites already states in its own
# comments; it holds the coarser and still mechanical bar that a file which
# compares populations with `comm` at all pins collation on every `sort` and
# `comm` it runs — opens with `LC_ALL=C`, immediately before the word,
# allowing for the pipe, the paren, the semicolon, `&&` or the line start
# that can precede it, and closes as a command word rather than a longer one
# (`sorted`, `command`), allowing for the whitespace, the closing paren of a
# `$(...)` it sits inside with no space before it (`| sort)` is the ordinary
# shape, and was the gap a verifier found before this file first shipped:
# `... | sort)` reads as unterminated to a check that only accepted
# whitespace or end-of-line on the right), the semicolon, the backtick or the
# line end that can follow it. A comment line (its first non-blank character
# is `#`) is never read as an invocation, so the prose above and inside each
# of the nine fixture files does not trip this suite on its own words.
#
# # WHAT IT NEEDS, AND WHAT IT WRITES
#
# `grep`, `awk` and `sed`. It builds no engine and runs none, and every
# scratch file is made under `mktemp -d`, which goes on an interrupt.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
self=$(basename "$0")

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

passed=0
failed=0

pass() {
    passed=$((passed + 1))
    echo "  ok    $1"
}

fail() {
    failed=$((failed + 1))
    echo "  FAIL  $1"
    [ -n "${2:-}" ] && printf '%s\n' "$2" | sed 's/^/          /'
}

# same NAME EXPECTED ACTUAL
same() {
    if [ "$2" = "$3" ]; then
        pass "$1"
    else
        fail "$1" "expected [$2], read [$3]"
    fi
}

# more_than NAME FLOOR ACTUAL — a population that came back empty is a judge
# that measured nothing. This is a floor and never an expected value.
more_than() {
    if [ "$3" -gt "$2" ] 2>/dev/null; then
        pass "$1 ($3)"
    else
        fail "$1" "expected more than $2, got \`$3\`"
    fi
}

# unpinned_lines FILE — one `path:line: text` per line of FILE that invokes
# `sort` or `comm` (as a command word, not as a substring of a longer word or
# a mention in prose) without `LC_ALL=C` immediately before it. A comment
# line is never read.
unpinned_lines() {
    awk '
        {
            line = $0
            trimmed = line
            sub(/^[ \t]*/, "", trimmed)
            if (substr(trimmed, 1, 1) == "#") next
            # Remove every already-pinned invocation so only a bare one, if
            # any remains, can trip the two checks below.
            work = line
            gsub(/LC_ALL=C[ \t]+sort/, "", work)
            gsub(/LC_ALL=C[ \t]+comm/, "", work)
            if (match(work, /(^|[|;(]|&&[ \t]*)[ \t]*sort([ \t);`]|&&|$)/)) {
                print FILENAME ":" FNR ": " line
                next
            }
            if (match(work, /(^|[|;(]|&&[ \t]*)[ \t]*comm([ \t);`]|&&|$)/)) {
                print FILENAME ":" FNR ": " line
            }
        }
    ' "$1"
}

# comm_using_files — every regular file under tools/, .githooks/ and
# .claude/ whose text names `comm` as a whole word outside a comment, one
# path per line, this suite's own file excluded. `grep -rl` reads the tree
# fresh on every run; nothing here is a list of file names.
comm_using_files() {
    for cu_dir in tools .githooks .claude; do
        [ -d "$root/$cu_dir" ] || continue
        grep -rlE '(^|[^A-Za-z0-9_])comm([^A-Za-z0-9_]|$)' "$root/$cu_dir" 2>/dev/null
    done | while read -r cu_f; do
        [ -f "$cu_f" ] || continue
        [ "$(basename "$cu_f")" = "$self" ] && continue
        case "$cu_f" in
            */.git/*) continue ;;
        esac
        printf '%s\n' "$cu_f"
    done | sort -u
}

# ---------------------------------------------------------------------------

comm_using_files >"$scratch/population"

echo "every sort/comm pairing under tools/, .githooks/ and .claude/ (#828)"

more_than "at least one script under the three directories calls \`comm\`" 0 \
    "$(wc -l <"$scratch/population" | tr -d ' ')"

: >"$scratch/violations"
while read -r cf; do
    [ -n "$cf" ] || continue
    unpinned_lines "$cf" >>"$scratch/violations"
done <"$scratch/population"

same "every \`sort\` and \`comm\` in a script that compares populations with \`comm\` is pinned to \`LC_ALL=C\`" \
    "" "$(tr '\n' '|' <"$scratch/violations")"

echo
echo "the judge, provoked over scratch files"

mkdir -p "$scratch/arms"
printf '#!/bin/sh\n# comm and sort are mentioned here in prose, never run.\nLC_ALL=C sort file | LC_ALL=C comm -23 - other\n' \
    >"$scratch/arms/clean.sh"
same "a script that pins both, and only mentions the words in a comment, is clean" \
    "" "$(unpinned_lines "$scratch/arms/clean.sh" | tr '\n' '|')"

printf '#!/bin/sh\nsort file | comm -23 - other\n' >"$scratch/arms/bare.sh"
same "a bare \`sort\` piped into a bare \`comm\` reddens on both words" \
    "$scratch/arms/bare.sh:2: sort file | comm -23 - other|" \
    "$(unpinned_lines "$scratch/arms/bare.sh" | tr '\n' '|')"

printf '#!/bin/sh\nLC_ALL=C sort file | comm -23 - other\n' >"$scratch/arms/half.sh"
same "a pinned \`sort\` into a bare \`comm\` still reddens, on the \`comm\` alone" \
    "$scratch/arms/half.sh:2: LC_ALL=C sort file | comm -23 - other|" \
    "$(unpinned_lines "$scratch/arms/half.sh" | tr '\n' '|')"

# A verifier found that the two checks above missed a bare invocation closed
# by the `)` of the `$(...)` it sits inside with no space before it —
# `x=$(... | sort)` — because the right-hand boundary only ever accepted
# whitespace or end of line. `tools/repo/library-index-fixtures.sh` and
# `tools/repo/diataxis-facet-fixtures.sh` both carried exactly this shape on
# an unrelated `sort` the day it was found. This arm is that shape, verbatim,
# for both words, so a future narrowing of the boundary reddens here first.
printf '#!/bin/sh\nx=$(find . -type f | sort)\ny=$(comm -12 a b)\n' \
    >"$scratch/arms/paren.sh"
same "a bare \`sort\` or \`comm\` closed by the ) of its own \$(...) reddens too" \
    "$scratch/arms/paren.sh:2: x=\$(find . -type f | sort)|$scratch/arms/paren.sh:3: y=\$(comm -12 a b)|" \
    "$(unpinned_lines "$scratch/arms/paren.sh" | tr '\n' '|')"

echo
echo "$passed passed, $failed failed"
[ "$failed" -eq 0 ]
