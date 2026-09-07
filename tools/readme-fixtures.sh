#!/bin/sh
# What holds the first screen: the root `README.md`.
#
# Nothing else in this repository reads that file. `docs/` is the corpus root,
# so the engine never sees it; `mkdocs build --strict` never sees it, because
# `docs_dir: docs`; and every `README` named in a workflow, a hook or another
# script under `tools/` is `engine/README.md` or `docs/interfaces/README.md`.
# The measurement that settled this planted a British spelling, a contraction,
# a hard-wrapped block, four retired terms, a dead link and a dead fragment
# into the root `README.md` and got exit 0 with a byte-identical check report.
# So the most-read prose in this repository was the only prose in it with no
# gate at all.
#
# This suite closes TWO of those eight — the dead link and the dead fragment —
# and it holds four further claims the page makes about itself. It closes none
# of the other six. The section below says so with the measurement, rather than
# leaving a reader to assume from a passing step that the page is covered.
#
# Run it from anywhere:
#     sh tools/readme-fixtures.sh
#
# # THE CASE THIS SUITE EXISTS FOR, WHICH IS NOT A BROKEN LINK
#
# The paste block on the first screen pins `git checkout v0.1.0`, and that tag
# is not cut. The paragraph above the fence says so, and tells the reader to
# omit the line. That sentence is true today and it is one merge away from
# being false: the day the tag is cut, the page tells a stranger to skip a line
# that now works, and every other gate in this repository stays green.
#
# So case group 4 asserts an EXCLUSIVE OR. Either the tag resolves against the
# remote, or the hedge sits in the paragraph directly above the fence. Never
# both, and never neither. That turns a coupling that lived in a comment thread
# into a mechanical one: the change that cuts the tag cannot merge until it
# deletes the sentence, and a change that deletes the sentence early cannot
# merge until the tag exists.
#
# The tag the judge asks about is read out of the fence rather than written
# here, so a page that pins a different version is judged against that version.
# A fence carrying no pinned checkout at all fails loudly, because a judge whose
# population went empty is a judge that reports green for the wrong reason.
#
# # WHAT EACH JUDGE READS, AND THE TWO EASY MISTAKES
#
# Links are read per OCCURRENCE and not per line. The Done-when this page was
# written against measured its own link count with `grep -c '](' `, which counts
# LINES CONTAINING a link: the page printed 16 before the rewrite and 8 after,
# while the occurrence counts were 45 and 18. This repository forbids
# hard-wrapped Markdown, so every paragraph here is one long line and a
# line-shaped count is off by a factor of three on this file. Case group 1
# counts occurrences.
#
# The second mistake is a heading slug written by hand. GitHub lowercases a
# heading, drops every character that is not a letter, a digit, a space, an
# underscore or a hyphen, and turns each remaining space into a hyphen — so
# `## Q11 — License and distribution posture` is `#q11--license-and-distribution-posture`,
# with two hyphens where the em dash stood between two spaces. The slug function
# below implements that rule and case 1e provokes it against a heading carrying
# an em dash, a backtick and a comma, because a slug rule nobody has seen
# refuse anything is a slug rule nobody has seen work.
#
# # WHAT THIS SUITE DOES NOT HOLD, WHICH IS SAID HERE RATHER THAN INFERRED
#
# Three claims on that page have no authority inside this tree, and no case
# below pretends otherwise.
#
#   The Status blockquote NAMES the open milestones. Case group 5 refuses a
#   COUNT of them, because a count is what went stale once and a count has no
#   in-tree source to be checked against. A milestone that closes with no edit
#   to this page leaves the names wrong and this suite silent. The authority
#   for that is the GitHub API, and a gate here does not open a socket — the
#   same posture `headwater probe` takes.
#
#   An absolute URL is not fetched. A dead external link stays dead and green.
#   Case group 3 judges only the ORG AND REPOSITORY a GitHub URL names, which
#   is the part a rename breaks silently and the part that is knowable offline.
#
#   The prose itself answers to no rule, and that is measured rather than
#   assumed: a British spelling, a contraction, a hard-wrapped paragraph, a
#   retired term and `headwater` written lower case in running prose were
#   planted on the page together, and this suite reported every case passing.
#   The language regime binds the kinds declared under the corpus root and this
#   file is not on a shelf, so this suite reimplements none of those rules — a
#   second copy of a rule living in a script is what this repository refuses
#   everywhere else. Putting the file on a shelf instead buys the census move
#   that #600 priced and refused, for a file GitHub renders and MkDocs never
#   sees, so whether the regime should reach outside `docs/` is a question
#   about the taxonomy and not a gap here.
#
# # WHAT IT NEEDS, AND WHAT IT WRITES
#
# `git` and `awk`. It builds no engine and runs none. Case group 4 asks the
# configured remote whether the tag resolves, which is one `ls-remote` and the
# only network this suite performs; every scratch arm stubs that with a local
# bare repository instead. Every scratch file is made under `mktemp -d`, the
# directory goes on an interrupt, and nothing inside this checkout is written.

set -u

root=$(cd "$(dirname "$0")/.." && pwd)
readme="$root/README.md"

if ! command -v git >/dev/null 2>&1; then
    echo "no \`git\` on the path, and the remote below is what names the org" >&2
    echo "  this page's badges must point at. This suite stops rather than" >&2
    echo "  reporting a row of passes over a set it could not read." >&2
    exit 1
fi

if [ ! -f "$readme" ]; then
    echo "no \`README.md\` at the root of this repository, so every case here" >&2
    echo "  has nothing to judge. This suite stops rather than reporting a row" >&2
    echo "  of passes over a file that is not there." >&2
    exit 1
fi

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
    echo "          $2"
}

# same NAME EXPECTED ACTUAL
same() {
    if [ "$2" = "$3" ]; then
        pass "$1"
    else
        fail "$1" "expected \`$2\`, got \`$3\`"
    fi
}

# more_than NAME FLOOR ACTUAL — a population that came back empty is a judge
# that measured nothing, and every group below carries one of these.
more_than() {
    if [ "$3" -gt "$2" ] 2>/dev/null; then
        pass "$1 ($3)"
    else
        fail "$1" "expected more than $2, got \`$3\`"
    fi
}

# The two awk helpers every link reader below shares, defined once and pasted
# into each program because awk has no include. Both exist because the first
# cut of this suite reddened on correct Markdown, and a required check on the
# most-read page in the project that refuses legitimate syntax is a check the
# first person it annoys turns off — after which it guards nothing. A false
# positive here costs more than a missed defect.
#
#   `strip_code` removes every inline code span, so a link PRINTED as an
#   example inside backticks is not read as a link. Fenced blocks were exempt
#   from the start and inline spans were not, which is the same rule applied
#   inconsistently. A run of n backticks closes on the next run of n, and an
#   unterminated run is treated as ordinary text, which is what CommonMark
#   does with it.
#
#   `dest` takes the link destination out of what stands between `](` and `)`.
#   CommonMark allows a title after the destination — `[a](p "t")`, `'t'` or
#   `(t)` — and the first cut carried the title into the path and reported the
#   file as missing. The destination is `<…>` if it is angle-bracketed and
#   everything up to the first space otherwise, which is the rule itself rather
#   than a list of the title shapes.
awk_helpers='
function strip_code(s,   out, i, run, open, rest, j) {
    out = ""
    while ((i = index(s, "`")) > 0) {
        out = out substr(s, 1, i - 1)
        s = substr(s, i)
        run = 0
        while (substr(s, run + 1, 1) == "`") run++
        open = substr(s, 1, run)
        rest = substr(s, run + 1)
        j = index(rest, open)
        if (j == 0) return out rest
        s = substr(rest, j + run)
    }
    return out s
}
function dest(d,   i) {
    sub(/^[ \t]+/, "", d)
    if (substr(d, 1, 1) == "<") {
        d = substr(d, 2)
        i = index(d, ">")
        return (i > 0) ? substr(d, 1, i - 1) : d
    }
    i = index(d, " ")
    if (i > 0) d = substr(d, 1, i - 1)
    i = index(d, "\t")
    if (i > 0) d = substr(d, 1, i - 1)
    return d
}
'

# links_of FILE — every `](target)` occurrence outside a fence and outside an
# inline code span, as `line<TAB>target`. Per occurrence, so two links on one
# line are two rows. A nested image link, `[![alt](badge)](href)`, yields both.
links_of() {
    awk "$awk_helpers"'
        /^[ \t]*```/ { fence = 1 - fence; next }
        fence { next }
        {
            rest = strip_code($0)
            while ((i = index(rest, "](")) > 0) {
                rest = substr(rest, i + 2)
                j = index(rest, ")")
                if (j == 0) { print NR "\t<unterminated>"; break }
                print NR "\t" dest(substr(rest, 1, j - 1))
                rest = substr(rest, j + 1)
            }
        }
    ' "$1"
}

# images_of FILE — every `![alt](target)` occurrence outside a fence and outside
# an inline code span.
images_of() {
    awk "$awk_helpers"'
        /^[ \t]*```/ { fence = 1 - fence; next }
        fence { next }
        {
            rest = strip_code($0)
            while (match(rest, /!\[[^]]*\]\([^)]*\)/)) {
                m = substr(rest, RSTART, RLENGTH)
                sub(/^!\[[^]]*\]\(/, "", m)
                sub(/\)$/, "", m)
                print NR "\t" dest(m)
                rest = substr(rest, RSTART + RLENGTH)
            }
        }
    ' "$1"
}

# urls_of FILE — every absolute URL, fences AND inline code spans included,
# because the clone command inside the fence names the repository too and a
# rename breaks it just as quietly as it breaks a badge. This is deliberately
# the opposite of the two readers above, and the difference is the reason: a
# link is markup, so where it is printed decides whether it is markup at all,
# while a URL naming this repository is wrong wherever it is printed.
urls_of() {
    awk '
        {
            rest = $0
            while (match(rest, /https?:\/\/[^ )>"`]+/)) {
                print NR "\t" substr(rest, RSTART, RLENGTH)
                rest = substr(rest, RSTART + RLENGTH)
            }
        }
    ' "$1"
}

# slugs_of FILE — the GitHub anchor of every ATX heading outside a fence.
slugs_of() {
    awk '
        /^[ \t]*```/ { fence = 1 - fence; next }
        fence { next }
        /^#{1,6}[ \t]/ {
            line = $0
            sub(/^#+[ \t]+/, "", line)
            sub(/[ \t]+#+[ \t]*$/, "", line)
            s = tolower(line)
            gsub(/[^a-z0-9 _-]/, "", s)
            gsub(/ /, "-", s)
            if (s != "") print s
        }
    ' "$1"
}

# link_judge ROOT FILE — one line per broken relative link, then a tail line
# holding `<occurrences> <relative> <fragment-bearing>`.
link_judge() {
    lj_root=$1
    lj_file=$2
    lj_dir=$(dirname "$lj_file")
    lj_all=0
    lj_rel=0
    lj_frag=0
    links_of "$lj_file" | while IFS='	' read -r line target; do
        printf '%s\t%s\n' "$line" "$target"
    done >"$scratch/links"
    while IFS='	' read -r line target; do
        lj_all=$((lj_all + 1))
        case $target in
            http://*|https://*|mailto:*|'') continue ;;
        esac
        lj_rel=$((lj_rel + 1))
        lj_path=${target%%#*}
        case $target in
            *'#'*) lj_anchor=${target#*#} ;;
            *) lj_anchor= ;;
        esac
        [ -n "$lj_anchor" ] && lj_frag=$((lj_frag + 1))
        if [ -z "$lj_path" ]; then
            lj_target=$lj_file
        else
            lj_target=$lj_dir/$lj_path
            if [ ! -e "$lj_target" ]; then
                echo "$line: $target  no such file"
                continue
            fi
        fi
        [ -z "$lj_anchor" ] && continue
        case $lj_target in
            *.md) ;;
            *) continue ;;
        esac
        if ! slugs_of "$lj_target" | grep -qxF "$lj_anchor"; then
            echo "$line: $target  no such heading"
        fi
    done <"$scratch/links"
    lj_all=$(wc -l <"$scratch/links" | tr -d ' ')
    lj_rel=$(awk -F'\t' '$2 !~ /^(https?:\/\/|mailto:)/ && $2 != ""' "$scratch/links" | wc -l | tr -d ' ')
    lj_frag=$(awk -F'\t' '$2 !~ /^(https?:\/\/|mailto:)/ && $2 ~ /#/' "$scratch/links" | wc -l | tr -d ' ')
    echo "$lj_all $lj_rel $lj_frag"
}

# png_size FILE — `<width> <height>` from the IHDR chunk, or `0 0`.
png_size() {
    od -An -tu1 -j16 -N8 "$1" 2>/dev/null | awk '
        NF == 8 {
            printf "%d %d\n", $1*16777216+$2*65536+$3*256+$4, $5*16777216+$6*65536+$7*256+$8
            found = 1
        }
        END { if (!found) print "0 0" }
    '
}

# image_judge ROOT FILE — offenders, then a tail line holding the image count.
# The social preview GitHub renders on a link card is 1280 by 640 and under
# 1 MB; the first image on the page is that file, and the rest are judged for
# existence and for weight only.
image_judge() {
    ij_file=$2
    ij_dir=$(dirname "$ij_file")
    images_of "$ij_file" >"$scratch/images"
    ij_n=0
    while IFS='	' read -r line target; do
        ij_n=$((ij_n + 1))
        case $target in
            http://*|https://*) continue ;;
        esac
        ij_path=$ij_dir/${target%%#*}
        if [ ! -f "$ij_path" ]; then
            echo "$line: $target  no such file"
            continue
        fi
        ij_bytes=$(wc -c <"$ij_path" | tr -d ' ')
        if [ "$ij_bytes" -ge 1048576 ]; then
            echo "$line: $target  $ij_bytes bytes, and GitHub refuses a social preview at 1 MB or more"
        fi
        [ "$ij_n" -eq 1 ] || continue
        set -- $(png_size "$ij_path")
        if [ "$1 $2" != "1280 640" ]; then
            echo "$line: $target  $1 by $2, and the social preview is 1280 by 640"
        fi
    done <"$scratch/images"
    wc -l <"$scratch/images" | tr -d ' '
}

# repo_slug REMOTE-URL — `owner/repository`, from either transport.
repo_slug() {
    rs=$1
    rs=${rs%.git}
    rs=${rs#git@github.com:}
    rs=${rs#ssh://git@github.com/}
    rs=${rs#https://github.com/}
    rs=${rs#http://github.com/}
    echo "$rs"
}

# badge_judge FILE SLUG — every URL naming a GitHub repository must name this
# one. Two families are read, and no other host is in the population:
#   github.com/<owner>/<repo>/…            — the CI badge, and the clone command
#   img.shields.io/github/…/<owner>/<repo> — a shields badge, whose metric path
#                                            varies in length, so the owner and
#                                            the repository are its last two
#                                            segments once a query string and a
#                                            `.svg` suffix are removed
badge_judge() {
    bj_file=$1
    bj_slug=$2
    urls_of "$bj_file" >"$scratch/urls"
    bj_n=0
    while IFS='	' read -r line url; do
        bj_path=${url#http://}
        bj_path=${bj_path#https://}
        bj_path=${bj_path%%\?*}
        case $bj_path in
            github.com/*)
                bj_rest=${bj_path#github.com/}
                bj_owner=${bj_rest%%/*}
                bj_rest=${bj_rest#*/}
                bj_name=${bj_rest%%/*}
                bj_name=${bj_name%.git}
                bj_named=$bj_owner/$bj_name
                ;;
            img.shields.io/github/*)
                bj_rest=${bj_path%.svg}
                bj_name=${bj_rest##*/}
                bj_rest=${bj_rest%/*}
                bj_owner=${bj_rest##*/}
                bj_named=$bj_owner/$bj_name
                ;;
            *) continue ;;
        esac
        bj_n=$((bj_n + 1))
        if [ "$bj_named" != "$bj_slug" ]; then
            echo "$line: $url  names $bj_named, and this repository is $bj_slug"
        fi
    done <"$scratch/urls"
    grep -cE '(github\.com|img\.shields\.io/github)/' "$scratch/urls" | tr -d ' '
}

# pinned_tag_of FILE — the tag the first fence pins with `git checkout`, or the
# empty string. The judge below asks the remote about THIS value rather than
# about a version written into this file.
pinned_tag_of() {
    awk '
        /^[ \t]*```/ { fence = 1 - fence; next }
        fence && /^[ \t]*git checkout[ \t]+/ {
            t = $3
            if (t != "") { print t; exit }
        }
    ' "$1"
}

# hedge_of FILE MARKER — where the hedge stands: `above-the-fence`, `elsewhere`
# or `absent`. The paragraph directly above the fence is the only position that
# reaches a reader before the paste does.
hedge_of() {
    awk -v marker="$2" '
        /^[ \t]*```/ && !fence { fence = 1; print (prev ~ marker) ? "above-the-fence" : "elsewhere-or-absent"; exit }
        /^[ \t]*$/ { next }
        { prev = $0 }
    ' "$1" >"$scratch/hedge.where"
    if grep -qF "$2" "$1"; then
        if [ "$(cat "$scratch/hedge.where")" = "above-the-fence" ]; then
            echo above-the-fence
        else
            echo elsewhere
        fi
    else
        echo absent
    fi
}

# tag_judge FILE MARKER TAG-RESOLVES — the exclusive or, as one word.
tag_judge() {
    tj_where=$(hedge_of "$1" "$2")
    case "$3:$tj_where" in
        no:above-the-fence) echo ok ;;
        yes:absent) echo ok ;;
        no:absent) echo "the tag does not resolve and the page does not say so" ;;
        no:elsewhere) echo "the tag does not resolve and the hedge is not above the fence" ;;
        yes:above-the-fence) echo "the tag resolves and the page still tells the reader to omit the line" ;;
        yes:elsewhere) echo "the tag resolves and the hedge is still on the page" ;;
        *) echo "unreadable" ;;
    esac
}

# tag_resolves REMOTE TAG — `yes`, `no`, or `unreadable`. Unreadable is not a
# pass: a remote this cannot reach leaves the exclusive or undecided, and the
# case that consumes it goes red rather than quiet.
tag_resolves() {
    if GIT_TERMINAL_PROMPT=0 git ls-remote --tags "$1" "refs/tags/$2" >"$scratch/lsremote" 2>"$scratch/lsremote.err"; then
        if [ -s "$scratch/lsremote" ]; then echo yes; else echo no; fi
    else
        echo unreadable
    fi
}

# milestone_judge FILE — a sentence stating how MANY milestones are open or
# shipped. The names are checked by nobody; the count is what went stale.
milestone_judge() {
    awk '
        {
            line = tolower($0)
            if (line ~ /(^|[^a-z])(one|two|three|four|five|six|seven|eight|nine|ten|eleven|twelve|[0-9]+) milestones/)
                print NR ": a count of milestones, and nothing in this tree can check one"
            if (line ~ /m[0-9]+ (to|through|-) ?m?[0-9]+/)
                print NR ": a range of milestones, and nothing in this tree can check one"
        }
    ' "$1"
}

slug=$(repo_slug "$(git -C "$root" remote get-url origin 2>/dev/null)")
marker='The tag is not cut yet'

echo "every relative link on the first screen"

# 1a-1c. The real page. All three numbers are reported rather than asserted
#        against a constant, because a count written down here is a count that
#        drifts; what is asserted is that the population is not empty and that
#        no member of it is broken.
link_judge "$root" "$readme" >"$scratch/links.out"
broken=$(sed '$d' "$scratch/links.out")
set -- $(tail -1 "$scratch/links.out")
same "no relative link on the page is dead" "" "$broken"
more_than "  link occurrences read" 0 "$1"
more_than "  of them relative" 0 "$2"
more_than "  of them carrying a fragment" 0 "$3"

# 1d. And the occurrence count is not the line count. The Done-when this page
#     answers to measured itself with `grep -c '](' `, which counts lines, and
#     the two numbers differ by a factor of three on a page whose paragraphs are
#     each one line. This case states the gap rather than describing it.
lines_with=$(grep -c '](' "$readme" | tr -d ' ')
if [ "$1" -gt "$lines_with" ]; then
    pass "  and occurrences ($1) outnumber lines carrying one ($lines_with)"
else
    fail "  and occurrences outnumber lines carrying one" \
        "occurrences $1, lines $lines_with — a line-shaped count would not be wrong here, so the case above it proves less than it claims"
fi

# 1e. The slug rule, provoked. An em dash between two spaces leaves two
#     hyphens, a backtick and a comma vanish, and a closing hash run is not part
#     of the heading. A slug function nobody has seen refuse anything is a slug
#     function nobody has seen work.
mkdir -p "$scratch/slugs"
printf '%s\n' \
    '# Q11 — License and distribution posture' \
    '' \
    '## The `check` verb, and what it reads ##' \
    '' \
    '```' \
    '### not a heading at all' \
    '```' \
    '' \
    '#### Under_score and hyphen-ated' >"$scratch/slugs/page.md"
got=$(slugs_of "$scratch/slugs/page.md" | tr '\n' ' ')
same "the slug rule matches GitHub on an em dash, a backtick and a fence" \
    "q11--license-and-distribution-posture the-check-verb-and-what-it-reads under_score-and-hyphen-ated " \
    "$got"

# 1f. The link judge, provoked. A scratch page carrying one live link, one dead
#     path and one live path with a dead fragment.
mkdir -p "$scratch/links.d/sub"
printf '%s\n' '## A real heading' >"$scratch/links.d/sub/there.md"
printf '%s\n' \
    'Live: [there](sub/there.md), and its [heading](sub/there.md#a-real-heading).' \
    '' \
    'Dead: [gone](sub/missing.md) and [wrong](sub/there.md#no-such-heading).' \
    '' \
    'Absolute, and not fetched: [x](https://example.invalid/nope).' >"$scratch/links.d/README.md"
got=$(link_judge "$scratch/links.d" "$scratch/links.d/README.md" | sed '$d' | tr '\n' '|')
same "the link judge names the dead path and the dead fragment, and only those" \
    "3: sub/missing.md  no such file|3: sub/there.md#no-such-heading  no such heading|" \
    "$got"
set -- $(link_judge "$scratch/links.d" "$scratch/links.d/README.md" | tail -1)
same "  and counts five occurrences over three lines" 5 "$1"
same "  four of them relative" 4 "$2"
same "  two of them carrying a fragment" 2 "$3"

# 1g. A CommonMark title after the destination. `[a](p "t")` is correct
#     Markdown and the first cut of this suite carried the title into the path
#     and reported the file as missing — twice, because an image carries one
#     too. A required check that reddens on correct syntax is a check the first
#     person it annoys turns off, so this is a worse defect than a missed one.
#     All three title delimiters are here, and so is an angle-bracketed
#     destination, which is the other shape the destination rule has to read.
mkdir -p "$scratch/titles"
printf '%s\n' '## A real heading' >"$scratch/titles/there.md"
printf '\211PNG\r\n\032\n\0\0\0\rIHDR\0\0\005\0\0\0\002\200' >"$scratch/titles/preview.png"
printf '%s\n' \
    'Double: [a](there.md "a title") and image ![b](preview.png "another").' \
    '' \
    "Single: [c](there.md 'a title'), parens: [d](there.md (a title))." \
    '' \
    'Angled: [e](<there.md>) and angled with a fragment [f](<there.md#a-real-heading>).' >"$scratch/titles/README.md"
got=$(link_judge "$scratch/titles" "$scratch/titles/README.md" | sed '$d' | tr '\n' '|')
same "a CommonMark title is not part of the path" "" "$got"
got=$(image_judge "$scratch/titles" "$scratch/titles/README.md" | sed '$d' | tr '\n' '|')
same "  nor of an image path" "" "$got"
set -- $(link_judge "$scratch/titles" "$scratch/titles/README.md" | tail -1)
same "  and all six links are still read" 6 "$1"

# 1h. A link inside an inline code span. Fenced blocks were exempt from the
#     first line of this suite and inline spans were not, which is one rule
#     applied in one place. A page that prints `[label](path)` as an example of
#     the syntax is not linking anywhere, and the file it names does not have
#     to exist.
mkdir -p "$scratch/spans"
printf '%s\n' '## A real heading' >"$scratch/spans/there.md"
printf '%s\n' \
    'Write a link as `[label](never/written.md)` and it points nowhere.' \
    '' \
    'A double span too: ``[x](also/missing.md)`` beside a live [one](there.md).' \
    '' \
    'And a backtick that never closes: `[y](third/missing.md) — read as text.' \
    '' \
    'Backticks inside the LABEL are not a span around the link: [`there.md`](there.md).' >"$scratch/spans/README.md"
got=$(link_judge "$scratch/spans" "$scratch/spans/README.md" | sed '$d' | tr '\n' '|')
same "a link inside an inline code span is not a link" \
    "5: third/missing.md  no such file|" \
    "$got"
set -- $(link_judge "$scratch/spans" "$scratch/spans/README.md" | tail -1)
same "  and only the three outside a closed span are read" 3 "$1"

echo "the image the first screen opens with"

# 2a-2b. The real page.
image_judge "$root" "$readme" >"$scratch/images.out"
bad=$(sed '$d' "$scratch/images.out")
same "the opening image exists, is 1280 by 640, and is under 1 MB" "" "$bad"
more_than "  images read" 0 "$(tail -1 "$scratch/images.out")"

# 2c. The image judge, provoked, against a PNG of the wrong size and a path that
#     is not there. The IHDR read is four bytes of big-endian width at offset 16
#     and four of height at 20, so a hand-built header is enough to drive it.
mkdir -p "$scratch/images.d"
printf '\211PNG\r\n\032\n\0\0\0\rIHDR\0\0\002\130\0\0\001\054' >"$scratch/images.d/wrong.png"
printf '%s\n' '![wrong size](wrong.png)' '' '![missing](nope.png)' >"$scratch/images.d/README.md"
got=$(image_judge "$scratch/images.d" "$scratch/images.d/README.md" | sed '$d' | tr '\n' '|')
same "the image judge names a wrong size and a missing file" \
    "1: wrong.png  600 by 300, and the social preview is 1280 by 640|3: nope.png  no such file|" \
    "$got"
same "  and the IHDR read agrees with the shipped preview" "1280 640" \
    "$(png_size "$root/assets/headwater-social-preview.png")"

echo "every GitHub URL the page carries"

# 3a-3b. The real page, against the org and repository the remote names.
more_than "the remote names an owner and a repository" 2 "$(printf '%s' "$slug" | wc -c | tr -d ' ')"
badge_judge "$readme" "$slug" >"$scratch/badges.out"
bad=$(sed '$d' "$scratch/badges.out")
same "every GitHub URL names $slug" "" "$bad"
more_than "  GitHub URLs read, badges and the clone command alike" 2 \
    "$(tail -1 "$scratch/badges.out")"

# 3c. The badge judge, provoked. A rename is the silent failure here: the owner
#     and the repository are hand-written into every one of these URLs, and a
#     redirect keeps the page rendering while the badge stops resolving.
printf '%s\n' \
    '[![CI](https://github.com/headwater-ai/headwater/actions/workflows/ci.yml/badge.svg)](https://github.com/headwater-ai/headwater/actions/workflows/ci.yml)' \
    '' \
    '[![License](https://img.shields.io/github/license/oldorg/headwater)](LICENSE)' \
    '' \
    '    git clone https://github.com/headwater-ai/oldname.git' \
    '' \
    'And <https://example.invalid/> names no repository at all.' >"$scratch/badges.md"
got=$(badge_judge "$scratch/badges.md" headwater-ai/headwater | sed '$d' | tr '\n' '|')
same "the badge judge names a stale owner and a stale repository name" \
    "3: https://img.shields.io/github/license/oldorg/headwater  names oldorg/headwater, and this repository is headwater-ai/headwater|5: https://github.com/headwater-ai/oldname.git  names headwater-ai/oldname, and this repository is headwater-ai/headwater|" \
    "$got"
same "  and reads four GitHub URLs, ignoring the host it does not know" 4 \
    "$(badge_judge "$scratch/badges.md" headwater-ai/headwater | tail -1)"

echo "the pinned checkout, and the sentence that hedges it"

# 4a. The tag is read out of the fence rather than written here. An empty
#     population is the failure mode this case exists for: delete the pinned
#     line and every case below it would otherwise judge nothing and pass.
tag=$(pinned_tag_of "$readme")
if [ -n "$tag" ]; then
    pass "the fence pins a tag ($tag)"
else
    fail "the fence pins a tag" \
        "no \`git checkout <tag>\` inside a fence, so the exclusive or below has nothing to be about"
fi

# 4b. The exclusive or, over the real page and the real remote.
resolves=no
if [ -n "$tag" ]; then
    resolves=$(tag_resolves "$(git -C "$root" remote get-url origin)" "$tag")
fi
case $resolves in
    yes|no)
        pass "  and the remote answers whether it resolves ($resolves)"
        same "  and exactly one of the tag and the hedge is present" ok \
            "$(tag_judge "$readme" "$marker" "$resolves")"
        ;;
    *)
        fail "  and the remote answers whether it resolves" \
            "\`git ls-remote\` could not read the remote, so the exclusive or is undecided. This is red rather than quiet, because an unreadable population is not a pass: $(head -1 "$scratch/lsremote.err" 2>/dev/null)"
        ;;
esac

# 4c-4f. The four arms, over scratch pages and stub remotes. Two are green and
#        two are red, because a gate that refuses everything is as useless as
#        one that refuses nothing, and each red arm carries its own sentence so
#        that the verdict says which way the page went wrong.
mkdir -p "$scratch/arms"
printf '%s\n' \
    '## Obtaining a named version' \
    '' \
    "\`v0.1.0\` is the first tagged release. **$marker**, so the third line below fails today." \
    '' \
    '```' \
    'git clone https://github.com/headwater-ai/headwater.git' \
    'cd headwater' \
    'git checkout v0.1.0' \
    '```' >"$scratch/arms/hedged.md"
printf '%s\n' \
    '## Obtaining a named version' \
    '' \
    '`v0.1.0` is the first tagged release.' \
    '' \
    '```' \
    'git clone https://github.com/headwater-ai/headwater.git' \
    'cd headwater' \
    'git checkout v0.1.0' \
    '```' >"$scratch/arms/plain.md"

# The stub remotes. `git ls-remote` reads a local path as a remote, so no
# network and no clone: one bare repository carrying the tag and one without it.
untagged="$scratch/arms/untagged.git"
tagged="$scratch/arms/tagged.git"
git init -q --bare "$untagged" >/dev/null 2>&1
work="$scratch/arms/work"
git init -q "$work" >/dev/null 2>&1
git -C "$work" symbolic-ref HEAD refs/heads/main >/dev/null 2>&1
: >"$work/seed"
git -C "$work" add -A >/dev/null 2>&1
git -C "$work" -c user.name=fixture -c user.email=fixture@example.invalid \
    -c commit.gpgsign=false commit -q -m seed >/dev/null 2>&1
git -C "$work" tag v0.1.0 >/dev/null 2>&1
git clone -q --bare "$work" "$tagged" >/dev/null 2>&1

same "  the stub remote without the tag resolves nothing" no "$(tag_resolves "$untagged" v0.1.0)"
same "  the stub remote with the tag resolves it" yes "$(tag_resolves "$tagged" v0.1.0)"
same "  hedge present, tag absent — the state today" ok \
    "$(tag_judge "$scratch/arms/hedged.md" "$marker" "$(tag_resolves "$untagged" v0.1.0)")"
same "  hedge deleted, tag present — the state after the release" ok \
    "$(tag_judge "$scratch/arms/plain.md" "$marker" "$(tag_resolves "$tagged" v0.1.0)")"
same "  hedge present, tag present — what cutting the tag alone would leave" \
    "the tag resolves and the page still tells the reader to omit the line" \
    "$(tag_judge "$scratch/arms/hedged.md" "$marker" "$(tag_resolves "$tagged" v0.1.0)")"
same "  hedge deleted, tag absent — what deleting the sentence early would leave" \
    "the tag does not resolve and the page does not say so" \
    "$(tag_judge "$scratch/arms/plain.md" "$marker" "$(tag_resolves "$untagged" v0.1.0)")"

# 4g. Position, not presence. A hedge moved out of the paragraph directly above
#     the fence is a hedge a reader meets after the paste, so the judge reads
#     where it stands and not whether the string occurs.
printf '%s\n' \
    '## Obtaining a named version' \
    '' \
    '```' \
    'git checkout v0.1.0' \
    '```' \
    '' \
    "Long afterwards: **$marker**." >"$scratch/arms/below.md"
same "  a hedge below the fence is not a hedge above it" \
    "the tag does not resolve and the hedge is not above the fence" \
    "$(tag_judge "$scratch/arms/below.md" "$marker" no)"

echo "the milestone sentence, which went stale once"

# 5a. The Status section is the population, so its disappearance is a failure
#     rather than a quiet pass.
if grep -q '^## Status' "$readme"; then
    pass "the page carries a Status section"
else
    fail "the page carries a Status section" "no \`## Status\` heading, so 5b judges nothing"
fi

# 5b. No count of milestones. The names on that page have no authority inside
#     this tree and this suite does not pretend to hold them; the count is the
#     part that went stale with nothing to notice, and it is refused outright.
bad=$(milestone_judge "$readme" | tr '\n' '|')
same "the page states no count of milestones" "" "$bad"

# 5c. The milestone judge, provoked, in both shapes it refuses.
printf '%s\n' \
    'Six milestones are open, and the rest are not.' \
    '' \
    'M1 to M5 shipped.' \
    '' \
    'The milestones still open are distribution and the measurement layer.' >"$scratch/status.md"
got=$(milestone_judge "$scratch/status.md" | tr '\n' '|')
same "the milestone judge names a count and a range, and leaves the names alone" \
    "1: a count of milestones, and nothing in this tree can check one|3: a range of milestones, and nothing in this tree can check one|" \
    "$got"

echo
echo "$passed passed, $failed failed"
[ "$failed" -eq 0 ]
