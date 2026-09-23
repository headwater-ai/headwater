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
# `comm` it runs — opens with `LC_ALL=C`, immediately before the word.
#
# THE RIGHT SIDE is not a list: it closes on the first character that cannot
# continue a shell word — anything that is not `[A-Za-z0-9_]` — or the end of
# the line, which is what a command word actually ends on. An enumerated
# list here went through two rounds of a verifier finding one more
# terminator a real call site used (a bare closing paren of the `$(...)` the
# call sits inside with no space before it, then a bare pipe, `||`, a
# redirection or a heredoc marker closing it the same way) before this suite
# settled on the boundary a shell word actually has, rather than another
# character added to a list that was never going to stop growing.
#
# THE LEFT SIDE opens on the pipe, the paren, the semicolon, `&&`, a
# backtick, a double quote or the line start — the characters after which a
# shell command name can legally begin, including the two shapes a third
# verifier found this list still missed once the right side was fixed:
# backtick command substitution (`` `sort file` ``) and a quoted command name
# (`"sort" file`). It is deliberately NOT the same bare word boundary the
# right side uses: "sort" preceded by nothing but a space, as in a message a
# script echoes ("no sort or comm..."), is prose rather than a command
# position, and a left side that matched any non-word character reads that
# prose as an invocation — confirmed by running the bare-boundary form
# against this tree before this comment was written, which turned ten
# lines of ordinary case names and messages red. A bare backtick is its own
# hazard the same way: it also opens a markdown code span inside an ordinary
# double-quoted message, which `tools/site/assemble-site.sh` already writes
# one of, so a backslash-escaped backtick (`\``) is stripped before either
# check runs, the same way an already-pinned invocation is, and only a bare,
# unescaped backtick opens a match.
#
# A comment line (its first non-blank character is `#`) is never read as an
# invocation, so the prose above and inside each of the nine fixture files
# does not trip this suite on its own words.
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
            # any remains, can trip the two checks below. Also remove every
            # backslash-escaped backtick: a shell backtick opens a command
            # substitution only when it is not escaped, and an escaped one
            # (an error message elsewhere in this tree carries it, quoting
            # sort/comm as a code span inside a double-quoted string) never
            # opens one.
            work = line
            gsub(/LC_ALL=C[ \t]+sort/, "", work)
            gsub(/LC_ALL=C[ \t]+comm/, "", work)
            gsub(/\\`/, "", work)
            # A POSIX reserved word that opens a new command (`do`, `then`,
            # a bare `!`, a case-arm pattern`s closing `)`, …) is a command
            # position exactly the way a pipe or a semicolon is, and this
            # tree writes several of them on the same line as the command
            # they open (`for f in *; do sort "$f"; done`, `pattern) comm
            # -12 a b ;;`). Marking the boundary right after each one with a
            # `|` — the character the check below already recognizes — lets
            # one regex hold both without hand-listing keyword-by-keyword
            # inside it. This is lexical, not a real shell parse: the run
            # below over the real tree is what proves this substitution
            # reaches no case name or message it should not.
            #
            # A fifth verifier (#1031) found that the gsub above matched a
            # reserved word wherever it appeared as a whole word at all,
            # with no check that the word itself sat in a command position:
            # "wait for sort to finish" put `for` immediately before `sort`
            # as ordinary English, and the unconditional match still opened
            # a new command there. The opener now requires the same
            # left-open context the sort/comm check below already requires
            # of `sort`/`comm` itself — pipe, semicolon, paren, backtick,
            # quote, `&&`, brace, line start — so a reserved word only
            # counts as a command opener when something that can precede a
            # shell command already precedes it, never a bare space.
            gsub(/(^|[|;(`"!{})]|&&[ \t]*)[ \t]*(if|then|elif|else|fi|do|done|for|while|until|case|esac)([^A-Za-z0-9_]|$)/, "&|", work)
            if (match(work, /(^|[|;(`"!{})]|&&[ \t]*)[ \t]*sort([^A-Za-z0-9_]|$)/)) {
                print FILENAME ":" FNR ": " line
                next
            }
            if (match(work, /(^|[|;(`"!{})]|&&[ \t]*)[ \t]*comm([^A-Za-z0-9_]|$)/)) {
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

# A second verifier found that even the widened list above (`)`, `;`,
# backtick, `&&`) was still a list, and a list is exactly what the first
# verifier's finding predicted would keep growing: a bare pipe, a bare `||`,
# either direction of redirection and a heredoc marker each close a shell
# word the same way and were each still invisible. The fix above stopped
# enumerating characters and closes on the boundary a shell word actually
# has — anything that is not `[A-Za-z0-9_]`, or the end of the line — so this
# arm plants five bare invocations, one per closing shape named above, none
# of them sharing a character with the arms before it, and asserts all five
# redden together.
printf '#!/bin/sh\nsort|head\ncomm||true\nsort>out.txt\nsort<in.txt\ncomm<<EOF2\nplaceholder\nEOF2\n' \
    >"$scratch/arms/boundary.sh"
same "a bare sort/comm closed by a pipe, ||, either redirection or a heredoc marker reddens, all five together" \
    "$scratch/arms/boundary.sh:2: sort|head|$scratch/arms/boundary.sh:3: comm||true|$scratch/arms/boundary.sh:4: sort>out.txt|$scratch/arms/boundary.sh:5: sort<in.txt|$scratch/arms/boundary.sh:6: comm<<EOF2|" \
    "$(unpinned_lines "$scratch/arms/boundary.sh" | tr '\n' '|')"

# A third verifier found the mirror gap on the LEFT: the opener was still a
# hand-enumerated list (`|`, `;`, `(`, `&&`, line start) and missed backtick
# command substitution and a quoted command name, neither of which is a
# character the right-hand fix touched. Widening the opener to backtick and
# `"` closes both, but a bare backtick is ambiguous in this tree: it also
# opens a markdown code span inside an ordinary double-quoted message, and an
# ESCAPED backtick (`\``) is how those messages write one without triggering
# a real command substitution — `tools/site/assemble-site.sh` already does
# this. So an escaped backtick is stripped before either check runs, the
# same way an already-pinned invocation is, and only a bare, unescaped one
# opens a match. This arm plants the two real gaps together on one line each,
# and a negative arm right after plants the escaped-backtick shape that must
# stay clean.
printf '#!/bin/sh\nx=`sort file`; y=`comm -12 a b`\n"sort" file | "comm" -12 - other\n' \
    >"$scratch/arms/leftopen.sh"
same "backtick command substitution and a quoted command name both reddens on the left" \
    "$scratch/arms/leftopen.sh:2: x=\`sort file\`; y=\`comm -12 a b\`|$scratch/arms/leftopen.sh:3: \"sort\" file | \"comm\" -12 - other|" \
    "$(unpinned_lines "$scratch/arms/leftopen.sh" | tr '\n' '|')"

printf '#!/bin/sh\necho "the \\`sort\\` step wrote to \\`comm\\` on stderr"\n' \
    >"$scratch/arms/escapedbacktick.sh"
same "an escaped backtick quoting sort/comm as a code span in a message stays clean" \
    "" "$(unpinned_lines "$scratch/arms/escapedbacktick.sh" | tr '\n' '|')"

# A fourth verifier found the left side still missed every POSIX reserved
# word that opens a new command — do, then, elif, a bare !, a brace group,
# and, most tellingly, a case arm's closing `)`, which looks exactly like
# the `$(...)` close the right-hand fix already treats as a terminator but
# is a left-hand opener instead. Confirmed by injecting these eight shapes
# into `tools/repo/developing-fixtures.sh` — one of the nine files this suite
# already reports clean — and finding them invisible before this arm's fix,
# caught after it, restored before committing. This arm plants one shape per
# line, matching that same injection, and asserts all eight redden.
printf '#!/bin/sh\nfor f in *; do sort "$f"; done\nwhile true; do comm -12 a b; done\nuntil false; do sort x; done\nif sort file; then :; fi\nif false; then :; elif sort file; then :; fi\n! sort file\n{ sort file; }\ncase $x in pattern) comm -12 a b ;; esac\n' \
    >"$scratch/arms/keywords.sh"
same "for/do, while/do, until/do, if, elif, a bare !, a brace group and a case arm all redden" \
    "$scratch/arms/keywords.sh:2: for f in *; do sort \"\$f\"; done|$scratch/arms/keywords.sh:3: while true; do comm -12 a b; done|$scratch/arms/keywords.sh:4: until false; do sort x; done|$scratch/arms/keywords.sh:5: if sort file; then :; fi|$scratch/arms/keywords.sh:6: if false; then :; elif sort file; then :; fi|$scratch/arms/keywords.sh:7: ! sort file|$scratch/arms/keywords.sh:8: { sort file; }|$scratch/arms/keywords.sh:9: case \$x in pattern) comm -12 a b ;; esac|" \
    "$(unpinned_lines "$scratch/arms/keywords.sh" | tr '\n' '|')"

# A fifth verifier (#1031) found that the fourth verifier's keyword-opener
# gsub above marks a reserved word as a command opener wherever it appears
# as a whole word at all, with no check that the word itself sits in a
# command position. English trips this the same way a real invocation
# does: "wait for sort to finish" and "wait until comm settles down" both
# put a keyword immediately before sort/comm as ordinary prose, and the
# unconditional gsub inserted a `|` right after "for"/"until" regardless,
# which then satisfied the left-side sort/comm check the same way a real
# pipe would. This arm plants both sentences and asserts neither reddens;
# it depends on the keywords arm just above staying green unmodified, which
# is the other half of the issue's own Done-when — a genuine invocation
# must still fire.
printf '#!/bin/sh\necho "wait for sort to finish before continuing"\necho "wait until comm settles down"\n' \
    >"$scratch/arms/prose.sh"
same "an opener keyword immediately before sort/comm in ordinary prose does not redden" \
    "" "$(unpinned_lines "$scratch/arms/prose.sh" | tr '\n' '|')"

# The verifier of that fix (#1031) found two genuine shapes it missed. A
# reserved word that opens a command right after another one (`do if sort`,
# `then if comm`, `else if sort`) was missed before the fix and after it,
# because the one-pass gsub consumed the space after the first keyword and
# never read the `|` it had inserted. And a single `&`, which ends a
# background command and opens the next, was in neither opener class, so
# `false & if sort` went from caught to missed. This arm plants one shape per
# line, each one resting on a different member of the opener class — the
# nested keyword, `&&`, `&`, `|` and a case arm's `)` — and every line must
# redden. The expected value is every line of the file, so it names no
# output of the function it judges.
printf '#!/bin/sh\nfor f in a b; do if sort "$f"; then :; fi; done\nif true; then if comm -12 a b; then :; fi; fi\nwhile read x; do while sort y; do :; done; done\nif x; then :; else if sort z; then :; fi; fi\ntrue && if sort a; then :; fi\nfalse & if sort bg; then :; fi\ncat f | while sort; do :; done\ncase $x in a) if comm -12 a b; then :; fi ;; esac\ntrue && sort x\nfalse & comm -12 a b\n' \
    >"$scratch/arms/nested.sh"
same "a keyword after a keyword, after && or &, after a pipe or a case arm, and a bare & before sort/comm all redden" \
    "$(awk 'NR > 1 { print FILENAME ":" NR ": " $0 }' "$scratch/arms/nested.sh" | tr '\n' '|')" \
    "$(unpinned_lines "$scratch/arms/nested.sh" | tr '\n' '|')"

echo
echo "$passed passed, $failed failed"
[ "$failed" -eq 0 ]
