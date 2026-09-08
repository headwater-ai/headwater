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
# it holds one further claim about those same links, that above `## License` the
# only path into the specification shelf is the generated index, and it holds
# six more claims the page makes about itself. It closes none of the other six.
# The section below says so with the measurement, rather than leaving a reader
# to assume from a passing step that the page is covered.
#
# Two of those six are a different KIND of claim from the rest. That count is
# the sentence in this header that goes stale: it read five on the day group 7
# landed and stayed five, because no case reads it and a group added on one
# branch does not conflict with the number written on another. Count the `echo`
# lines below rather than trusting it.
# Groups 1 to 5 read the page and judge what it says. Group 6 takes a command
# the page tells a newcomer to run, runs it against the engine, and reads what
# that newcomer would see. The defect it closes was a command that always exited
# 1 and printed the same refusal whether the reader's digest matched or not, and
# no gate here or anywhere else in this repository saw it, because every gate
# read the page and none of them ran it.
#
# Group 7 is a third kind again: it reads the page against ANOTHER FILE. The
# page offers a stranger a download by name, and `.github/workflows/release.yml`
# uploads a file by name, and until this group existed nothing compared the two
# strings. Nothing could, either — that workflow triggers on a tag, a pull
# request pushes no tag, so the whole file is unexercised until somebody cuts a
# release and the person who finds the mismatch is the stranger who follows the
# page to a 404. Group 7 runs on every push because it reads two files rather
# than a release, and it holds the archive name, the checksum name, the tag
# pattern against the tag the fence pins, and the `contents: write` the upload
# needs. What it cannot hold is that the run succeeded, which only a cut tag
# shows.
#
# Two of its cases read the workflow as a PROGRAM rather than as a set of names,
# because a review of its first cut found two edits that leave every declared
# name in place and still send the reader to a 404. Deleting the whole
# `gh release upload` line changes no name; passing the checksum and not the
# archive changes no name. So one judge resolves the operands that command is
# actually handed, through the shell assignments the file makes, and compares
# THOSE with the page. The other reads for a shell option: GitHub's default
# `run:` shell on Linux is `bash -e {0}` and not `bash -eo pipefail {0}`, so
# `x=$(binary | filter)` takes the filter's status, and the version guard in
# that workflow passed for every tag on a binary that refused to run. Both
# defects are in what the file DOES, and no reader of what it SAYS can see them.
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
# `git`, `awk`, and a built engine. It builds none itself: it takes the newer of
# `engine/target/release/headwater` and `engine/target/dev-release/headwater`,
# which is the rule `.githooks/pre-commit` and `.claude/hooks/lib.sh` both
# implement, and it goes RED when neither is there. The CI step that runs this
# suite sits in the job that builds the release binary, so one is present where
# it runs. A skip would re-create the defect group 6 closes: a check that cannot
# run is not a check that passes.
#
# Case group 4 asks the configured remote whether the tag resolves, which is one
# `ls-remote` and the only network this suite performs; every scratch arm stubs
# that with a local bare repository instead. Group 6 runs the engine, which
# opens no socket at all.
#
# Every scratch file is made under `mktemp -d`, the directory goes on an
# interrupt, and nothing inside this checkout is written. Group 6 matters most
# here, because the arm it runs SUCCEEDS and a successful `taxonomy vendor`
# installs a package over the directory it was handed. So it copies the pin and
# the package into the scratch tree and runs there, with the working directory
# set to the copy rather than an extra `--root` on a command line the page owns.

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

# spec_link_judge FILE — one line per link ABOVE the `## License` heading whose
# target names the specification shelf and is not the generated index, then a
# tail line holding `<index occurrences above> <the line the heading sits on>`.
#
# The region is bounded at `## License` because the citation in that section is
# a licensing fact and not a reading path, and because the boundary has to be
# read out of the page rather than written down as a line number here.
spec_link_judge() {
    slj_file=$1
    slj_stop=$(awk '/^##[ \t]+License[ \t]*$/ { print NR; exit }' "$slj_file")
    slj_stop=${slj_stop:-0}
    if [ "$slj_stop" -eq 0 ]; then
        echo "no \`## License\` heading, so the region above it has no lower edge"
        echo "0 0"
        return
    fi
    links_of "$slj_file" | awk -F'\t' -v stop="$slj_stop" '
        $1 >= stop { next }
        $2 !~ /^docs\/spec\// { next }
        {
            path = $2
            sub(/#.*$/, "", path)
        }
        path == "docs/spec/README.md" { index_seen++; next }
        { print $1 ": " $2 "  a specification part above `## License`" }
        END { print "TAIL " index_seen + 0 }
    ' >"$scratch/spec.links"
    grep -v '^TAIL ' "$scratch/spec.links"
    echo "$(awk '/^TAIL /{print $2}' "$scratch/spec.links") $slj_stop"
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

# release_names_raw FILE MODE — every release asset name the file states, one
# per line, in the order it meets them, with the tag reduced to `<tag>` however
# the file spells it.
#
# MODE `page` reads the fences and the inline code spans of a Markdown file and
# nothing else, because a name a reader is told to type is a name written in
# code. MODE `workflow` reads a YAML file with its comments removed, because a
# comment in a workflow explains a name and never uploads one.
#
# Only the three spellings of the tag that a workflow can put into an asset name
# are normalized. Every other `${{ … }}` is left alone, so a name built from the
# wrong expression reads as the wrong name here rather than as the right one.
release_names_raw() {
    [ -f "$1" ] || return 0
    case "$2" in
        page)
            awk '
                function spans(s,   i, run, open, rest, j) {
                    while ((i = index(s, "`")) > 0) {
                        s = substr(s, i)
                        run = 0
                        while (substr(s, run + 1, 1) == "`") run++
                        open = substr(s, 1, run)
                        rest = substr(s, run + 1)
                        j = index(rest, open)
                        if (j == 0) return
                        print substr(rest, 1, j - 1)
                        s = substr(rest, j + run)
                    }
                }
                /^[ \t]*```/ { fence = 1 - fence; next }
                fence { print; next }
                { spans($0) }
            ' "$1"
            ;;
        workflow)
            awk '
                /^[ \t]*#/ { next }
                { sub(/[ \t]#.*$/, ""); print }
            ' "$1"
            ;;
        *) return ;;
    esac |
        sed -e 's/\${{ *github\.ref_name *}}/<tag>/g' \
            -e 's/\${{ *inputs\.tag *}}/<tag>/g' \
            -e 's/\${{ *steps\.tag\.outputs\.tag *}}/<tag>/g' \
            -e 's/\${TAG}/<tag>/g' -e 's/\$TAG/<tag>/g' |
        awk '
            {
                s = $0
                while (match(s, /headwater-[A-Za-z0-9._<>-]+\.tar\.gz(\.sha256)?/)) {
                    print substr(s, RSTART, RLENGTH)
                    s = substr(s, RSTART + RLENGTH)
                }
            }
        '
}

# release_archives_of FILE MODE — the archive names alone, sorted and unique,
# with a `.sha256` companion folded onto the archive it covers. This is the set
# the judge below compares, and an empty one is red at the call site: a page
# that names no download and a workflow that uploads nothing agree with each
# other and leave a reader nowhere.
release_archives_of() {
    release_names_raw "$1" "$2" | sed 's/\.sha256$//' | LC_ALL=C sort -u
}

# release_checksums_of FILE MODE — the `.sha256` names alone, sorted and unique.
release_checksums_of() {
    release_names_raw "$1" "$2" | grep '\.sha256$' | LC_ALL=C sort -u
}

# oneline — a list of names as one line, for a verdict sentence.
oneline() {
    printf '%s\n' "$1" | tr '\n' ' ' | sed -e 's/  *$//' -e 's/^ *//'
}

# release_asset_judge PAGE WORKFLOW — the coupling this group exists for, as one
# word or one sentence.
#
# A tag-triggered job runs zero times on a pull request, so the name it uploads
# and the name the page tells a stranger to download are two strings that no run
# of anything ever compares. When they differ, the reader gets a 404 and every
# gate in this repository stays green. This judge is the comparison, and it runs
# on every push because it reads two files rather than a release.
release_asset_judge() {
    ra_page=$(release_archives_of "$1" page)
    ra_flow=$(release_archives_of "$2" workflow)
    if [ -z "$ra_page" ] && [ -z "$ra_flow" ]; then
        echo "neither the page nor the workflow names an asset"
    elif [ -z "$ra_page" ]; then
        echo "the workflow uploads an asset and the page names none"
    elif [ -z "$ra_flow" ]; then
        echo "the page names an asset and the workflow uploads none"
    elif [ "$ra_page" = "$ra_flow" ]; then
        echo ok
    else
        echo "the page names $(oneline "$ra_page") and the workflow uploads $(oneline "$ra_flow")"
    fi
}

# release_checksum_judge PAGE WORKFLOW — the same comparison over the checksum
# beside the archive. Separate from the archive judge because a missing checksum
# is a different failure from a wrong name: the download works and nothing the
# reader can run says the bytes are the ones this repository built.
release_checksum_judge() {
    rc_page=$(release_checksums_of "$1" page)
    rc_flow=$(release_checksums_of "$2" workflow)
    if [ -z "$rc_page" ] && [ -z "$rc_flow" ]; then
        echo "neither the page nor the workflow names a checksum"
    elif [ -z "$rc_page" ]; then
        echo "the workflow writes a checksum the page tells nobody to read"
    elif [ -z "$rc_flow" ]; then
        echo "the page names a checksum the workflow does not write"
    elif [ "$rc_page" = "$rc_flow" ]; then
        echo ok
    else
        echo "the page names $(oneline "$rc_page") and the workflow writes $(oneline "$rc_flow")"
    fi
}

# release_yaml_text FILE — a workflow with its comments removed, which is what
# every judge below reads. A rule that a comment can satisfy is not a rule.
release_yaml_text() {
    [ -f "$1" ] || return 0
    awk '
        /^[ \t]*#/ { next }
        { sub(/[ \t]#.*$/, ""); print }
    ' "$1"
}

# release_tag_patterns FILE — the tag globs the workflow triggers on, one per
# line. Both YAML shapes are read, the block sequence and the inline flow list,
# because a file written either way triggers the same runs.
release_tag_patterns() {
    [ -f "$1" ] || return 0
    awk '
        /^[ \t]*#/ { next }
        /^[a-zA-Z]/ { on = ($0 ~ /^on:/); tags = 0 }
        !on { next }
        /^[ \t]*tags:/ {
            rest = $0
            sub(/^[ \t]*tags:[ \t]*/, "", rest)
            if (rest == "") { tags = 1; next }
            gsub(/[][,]/, " ", rest)
            n = split(rest, f, " ")
            for (i = 1; i <= n; i++) if (f[i] != "") print f[i]
            next
        }
        tags && /^[ \t]*-[ \t]*/ {
            v = $0
            sub(/^[ \t]*-[ \t]*/, "", v)
            print v
            next
        }
        tags { tags = 0 }
    ' "$1" | tr -d "'\""
}

# release_trigger_judge WORKFLOW TAG — does a tag of the form the fence pins
# start this workflow at all? A trigger nobody can provoke from a pull request
# is the one line in this change that no run exercises before a tag is cut.
release_trigger_judge() {
    rt_pats=$(release_tag_patterns "$1")
    if [ -z "$rt_pats" ]; then
        echo "the workflow triggers on no tag pattern, so pushing a tag runs nothing"
        return
    fi
    if [ -z "$2" ]; then
        echo "there is no tag to match, because the fence pins none"
        return
    fi
    for rt_p in $rt_pats; do
        # shellcheck disable=SC2254
        case "$2" in
            $rt_p)
                echo ok
                return
                ;;
        esac
    done
    echo "the tag the fence pins ($2) matches none of $(oneline "$rt_pats")"
}

# release_write_judge WORKFLOW — the token permission `gh release upload` needs.
# The workflow-level default in this repository is `contents: read`, and a job
# that inherits it fails at the upload after it has built everything.
release_write_judge() {
    if release_yaml_text "$1" | grep -q '^[ 	]*contents:[ 	]*write[ 	]*$'; then
        echo ok
    else
        echo "no job declares \`contents: write\`, so the upload runs with a read-only token"
    fi
}

# release_dispatch_judge WORKFLOW — a hand entry point against a tag that is
# already cut. Without it the only way to exercise any of this is to cut a tag,
# and a tag is cut once.
release_dispatch_judge() {
    if release_yaml_text "$1" | grep -q '^[ 	]*workflow_dispatch:[ 	]*$'; then
        echo ok
    else
        echo "no \`workflow_dispatch\`, so a tag already cut can never be given an asset"
    fi
}

# release_uploaded_names FILE — the asset names the workflow ACTUALLY HANDS to
# `gh release upload`, one per line, resolved through the shell assignments the
# file makes.
#
# The judges above read the names the workflow DECLARES, and a declaration is
# not an upload. Deleting the upload line entirely, or passing the checksum and
# not the archive, leaves every name in this file exactly where it was and
# reproduces the same 404 the group exists to prevent — which is what a review
# of the first cut of this group found, and which is why this function reads the
# command rather than the file.
#
# Resolution is file-scope rather than step-scope, because the two steps here
# pass the value between them through `$GITHUB_ENV` and a step-scope reader
# would see an unassigned variable in the step that does the upload. An operand
# naming a variable that nothing assigns is printed as `unresolved:<name>`, so a
# renamed variable is a wrong name here rather than a missing one.
#
# Quotes are stripped before parsing because nothing below depends on them and
# an operand is quoted or bare in different hands.
release_uploaded_names() {
    [ -f "$1" ] || return 0
    release_yaml_text "$1" |
        sed -e 's/\${{ *github\.ref_name *}}/<tag>/g' \
            -e 's/\${{ *inputs\.tag *}}/<tag>/g' \
            -e 's/\${{ *steps\.tag\.outputs\.tag *}}/<tag>/g' \
            -e 's/\${TAG}/<tag>/g' -e 's/\$TAG/<tag>/g' |
        tr -d '\42\47' |
        awk '
            {
                line = $0
                sub(/^[ \t]+/, "", line)
                if (match(line, /^[A-Za-z_][A-Za-z0-9_]*=/)) {
                    var[substr(line, 1, RLENGTH - 1)] = substr(line, RLENGTH + 1)
                    next
                }
                if (index(line, "gh release upload") > 0) upload[++u] = line
            }
            END {
                for (i = 1; i <= u; i++) {
                    n = split(upload[i], w, /[ \t]+/)
                    seen = 0
                    for (j = 1; j <= n; j++) {
                        t = w[j]
                        if (t == "upload") { seen = 1; continue }
                        if (!seen || t ~ /^-/) continue
                        if (t ~ /^\$\{?[A-Za-z_][A-Za-z0-9_]*\}?$/) {
                            key = t
                            gsub(/[${}]/, "", key)
                            t = (key in var) ? var[key] : "unresolved:" key
                        }
                        if (t ~ /headwater-[A-Za-z0-9._<>-]+\.tar\.gz/ || t ~ /^unresolved:/)
                            print t
                    }
                }
            }
        ' | LC_ALL=C sort -u
}

# release_upload_judge PAGE WORKFLOW — the page's offer against the command that
# attaches the files, rather than against the names the file writes down.
release_upload_judge() {
    ru_page=$(release_names_raw "$1" page | LC_ALL=C sort -u)
    ru_up=$(release_uploaded_names "$2")
    if [ -z "$ru_up" ]; then
        echo "no \`gh release upload\` hands over an asset, so the tag gets nothing"
    elif [ -z "$ru_page" ]; then
        echo "the upload attaches an asset and the page offers none"
    elif [ "$ru_page" = "$ru_up" ]; then
        echo ok
    else
        echo "the page offers $(oneline "$ru_page") and the upload attaches $(oneline "$ru_up")"
    fi
}

# release_pipe_steps FILE MODE — the steps of a workflow whose shell reads a
# command through a pipe inside a substitution. MODE `offenders` names the ones
# that do not set `pipefail`; MODE `guarded` names the ones that do.
#
# GitHub's default shell for a `run:` block on Linux is `bash -e {0}` and not
# `bash -eo pipefail {0}`, so `x=$(cmd | filter)` takes the filter's exit status
# and a command that refused to run leaves `x` empty at exit 0. The version
# guard in `release.yml` passed for every tag on that alone. It is the same trap
# this repository's own run policy states as "never pipe a gate", and nothing
# read a workflow for it until now.
release_pipe_steps() {
    [ -f "$1" ] || return 0
    awk -v want="$2" '
        function endblock(   t) {
            if (blk != "") {
                t = blk
                gsub(/\|\|/, "", t)
                if (blk ~ /\$\(/ && t ~ /\|/) {
                    if (blk ~ /pipefail/) { if (want == "guarded") print step }
                    else if (want == "offenders") print step
                }
            }
            blk = ""
        }
        /^[ \t]*-[ \t]*name:/ {
            endblock()
            step = $0
            sub(/^[ \t]*-[ \t]*name:[ \t]*/, "", step)
            next
        }
        /^[ \t]*run:[ \t]*\|[ \t]*$/ { endblock(); ind = index($0, "run:"); inblk = 1; next }
        inblk {
            if ($0 ~ /^[ \t]*$/) next
            if (match($0, /[^ \t]/) < ind) { endblock(); inblk = 0 }
            else { blk = blk $0 "\n"; next }
        }
        END { endblock() }
    ' "$1"
}

# vendor_invocations_of FILE — every `headwater taxonomy vendor …` invocation
# the page carries, one per line, read out of the page rather than written here.
# An inline code span carries it today and the paste block could carry it
# tomorrow, so both are read and a page that moves the command into the fence is
# still judged. An empty result is red at the call site: a judge whose
# population went empty is a judge that reports green for the wrong reason.
vendor_invocations_of() {
    awk '
        function spans(s,   i, run, open, rest, j, body) {
            while ((i = index(s, "`")) > 0) {
                s = substr(s, i)
                run = 0
                while (substr(s, run + 1, 1) == "`") run++
                open = substr(s, 1, run)
                rest = substr(s, run + 1)
                j = index(rest, open)
                if (j == 0) return
                body = substr(rest, 1, j - 1)
                if (body ~ /^headwater taxonomy vendor([ \t]|$)/) print body
                s = substr(rest, j + run)
            }
        }
        /^[ \t]*```/ { fence = 1 - fence; next }
        fence {
            line = $0
            sub(/^[ \t]+/, "", line)
            if (line ~ /^headwater taxonomy vendor([ \t]|$)/) print line
            next
        }
        { spans($0) }
    ' "$1"
}

# vendor_operand_of INVOCATION — the directory operand, or the empty string when
# the invocation names none. `vendor <dir> [--expect <digest>]` is the grammar
# `docs/interfaces/headwater-taxonomy.md` publishes, so the operand is the first
# word after the verb that is neither an option nor the value of one. `--expect`
# and `--root` are the two options here that take a value.
vendor_operand_of() {
    printf '%s\n' "$1" | awk '
        {
            for (i = 4; i <= NF; i++) {
                if ($i == "--expect" || $i == "--root") { i++; continue }
                if ($i ~ /^-/) continue
                print $i
                exit
            }
        }
    '
}

# release_digest_of FILE — the `release.digest` field of a release record. That
# is the one value the GitHub release page states, and it is the value the page
# tells a reader to paste.
release_digest_of() {
    awk '
        /^release:/ { block = 1; next }
        /^[^ \t#]/ { block = 0 }
        block && $1 == "digest:" { print $2; exit }
    ' "$1"
}

# vendor_corpus DIR OPERAND — the part of a checkout the page's command reads:
# the pin, and the directory the invocation names. The matching arm WRITES, and
# it vendors the package over itself, so every run below happens over a copy
# under `mktemp -d` and never over this checkout. What is copied is derived from
# the operand the page states, so a page that names a different directory is
# judged against that directory.
vendor_corpus() {
    mkdir -p "$1/.headwater" "$1/$(dirname "$2")" || return 1
    cp "$root/.headwater/taxonomy.yml" "$1/.headwater/" || return 1
    cp -R "$root/$2" "$1/$(dirname "$2")/" || return 1
}

# vendor_arm CORPUS INVOCATION DIGEST OUT — runs the page's own invocation with
# the value after `--expect` replaced, writes both streams to OUT and prints the
# exit status. The working directory is CORPUS, which is what `cd headwater` on
# the page leaves a reader in, so nothing is added to the command line: the
# argument list is the page's, minus the name of the binary. `set -f` is there
# because the arguments come from a file and are split on spaces.
vendor_arm() {
    va_args=${2#headwater }
    va_args=$(printf '%s' "$va_args" | sed "s|--expect [^ ]*|--expect $3|")
    ( set -f; cd "$1" || exit 127; exec "$engine" $va_args ) >"$4" 2>&1
    echo $?
}

# vendor_judge CORPUS INVOCATION DIGEST — `ok`, or the sentence that says how
# the page's command fails the reader who runs it. Two properties, reported
# TOGETHER and never one instead of the other, because they are independent and
# the page broke both: the exit status has to answer the digest, and the two
# reports have to differ.
#
# The second is the half a pair of exit statuses cannot see, and it is the half
# that made this defect quiet. A command that refuses on its grammar before it
# reads `--expect` refuses a correct digest and a mistyped one in the same bytes,
# so the reader is told nothing about the digest they were sent to check. An
# earlier cut of this judge returned on the exit status first and never reached
# the comparison, which reported the page's own defect as an ordinary refusal.
vendor_judge() {
    vj_match=$(vendor_arm "$1" "$2" "$3" "$scratch/vendor.match")
    vj_wrong=$(vendor_arm "$1" "$2" "$bad_digest" "$scratch/vendor.mismatch")
    vj_status=
    if [ "$vj_match" != 0 ]; then
        vj_status="the digest the release states is refused"
    elif [ "$vj_wrong" = 0 ]; then
        vj_status="a wrong digest is accepted"
    fi
    vj_bytes=
    if cmp -s "$scratch/vendor.match" "$scratch/vendor.mismatch"; then
        vj_bytes="the two reports are byte-identical"
    fi
    if [ -n "$vj_status" ] && [ -n "$vj_bytes" ]; then
        echo "$vj_status; $vj_bytes"
    elif [ -n "$vj_status" ]; then
        echo "$vj_status"
    elif [ -n "$vj_bytes" ]; then
        echo "$vj_bytes"
    else
        echo ok
    fi
}

slug=$(repo_slug "$(git -C "$root" remote get-url origin 2>/dev/null)")
marker='The tag is not cut yet'

# The engine, as the newer of the two profiles that write one, which is the rule
# `.githooks/pre-commit` and `.claude/hooks/lib.sh` both implement. A session
# builds whichever profile it likes and the newer binary answers. No engine at
# all is RED below and never skipped, because an arm that passes quietly when
# the binary is missing re-creates the defect group 6 exists to close.
engine=
if [ -x "$root/engine/target/release/headwater" ]; then
    engine="$root/engine/target/release/headwater"
fi
if [ -x "$root/engine/target/dev-release/headwater" ] &&
    { [ -z "$engine" ] || [ "$root/engine/target/dev-release/headwater" -nt "$engine" ]; }; then
    engine="$root/engine/target/dev-release/headwater"
fi

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

# 1i. The specification shelf, above `## License`. The page once carried a
#     hand-kept table of every specification part, and 34 of its 44 link
#     occurrences were that table. The remedy is not a ceiling on the count:
#     cases 1a-1c above refuse a constant on purpose, and the count went from 18
#     to 21 through merges this page's own Done-when required — the release badge
#     is two occurrences and the release-page deep link is a third. So the claim
#     asserted here is structural. A stranger's first screen reaches the
#     specification shelf through the GENERATED index and through nothing else,
#     which is the state a regrown table breaks and a count does not detect.
#
#     The `## License` section is exempt because the part it cites, `09-decisions`
#     at Q11, is the record of the licensing choice a reader of that section
#     wants, and not a reading path into the shelf.
spec_link_judge "$readme" >"$scratch/spec.out"
strays=$(sed '$d' "$scratch/spec.out")
set -- $(tail -1 "$scratch/spec.out")
same "above \`## License\`, the only \`docs/spec/\` link is the generated index" "" "$strays"
more_than "  and the generated index is linked there" 0 "$1"
more_than "  and the \`## License\` heading bounds the region" 0 "$2"

# 1j-1k. The judge, provoked in both shapes. A judge nobody has seen refuse a
#        page is a judge nobody has seen work, and a judge that refuses every
#        page holds nothing either. The green arm carries a specification part
#        BELOW the heading, so the boundary is exercised rather than assumed.
mkdir -p "$scratch/spec.d"
printf '%s\n' \
    'Index: [the shelf](docs/spec/README.md).' \
    '' \
    'And a part: [principle 2](docs/spec/00-vision-and-scope.md#design-principles).' \
    '' \
    '## License' \
    '' \
    '[Q11](docs/spec/09-decisions.md#q11--license-and-distribution-posture) records it.' >"$scratch/spec.d/regrown.md"
got=$(spec_link_judge "$scratch/spec.d/regrown.md" | sed '$d' | tr '\n' '|')
same "  a second specification link above the heading is named" \
    "3: docs/spec/00-vision-and-scope.md#design-principles  a specification part above \`## License\`|" \
    "$got"
printf '%s\n' \
    'Index: [the shelf](docs/spec/README.md), and its [reading order](docs/spec/README.md#reading-order).' \
    '' \
    '## License' \
    '' \
    '[Q11](docs/spec/09-decisions.md#q11--license-and-distribution-posture) records it.' >"$scratch/spec.d/folded.md"
got=$(spec_link_judge "$scratch/spec.d/folded.md" | sed '$d' | tr '\n' '|')
same "  and a page whose only path into the shelf is the index is clean" "" "$got"
set -- $(spec_link_judge "$scratch/spec.d/folded.md" | tail -1)
same "  counting both index occurrences, fragment or none" 2 "$1"
same "  and reading the heading off the page rather than a written line number" 3 "$2"
printf '%s\n' 'No heading here, and [a part](docs/spec/09-decisions.md).' >"$scratch/spec.d/unbounded.md"
got=$(spec_link_judge "$scratch/spec.d/unbounded.md" | sed '$d' | tr '\n' '|')
same "  a page with no \`## License\` heading is refused rather than passed" \
    "no \`## License\` heading, so the region above it has no lower edge|" \
    "$got"

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

echo "the command the page tells a newcomer to run"

# The first group here that RUNS a command out of the page. Every group above
# reads the page and judges what it says; this one takes the invocation the page
# states, hands it to the engine, and reads what a reader would see.
#
# The contract it holds is published, not invented here:
# `docs/interfaces/headwater-taxonomy.md` states the grammar as
# `vendor <dir> [--expect <digest>]` and states that `vendor` returns 0 when its
# operation succeeds and 1 on refusal. A page that omits the operand contradicts
# a table this repository publishes, and the reader meets the contradiction as a
# refusal that never looks at the digest they were told to paste.
#
# `import` is the other command that takes `--expect`, and it does NOT carry
# this defect. It takes no directory operand at all: it reads a declared import
# block out of `.headwater/taxonomy.yml`, and its unpinned paths return
# `Refusal::Unpinned` and `Refusal::NoChannel` from
# `engine/crates/import/src/lib.rs:420-435`, which are refusals about the
# declaration rather than a missing operand and differ from a digest mismatch.
# The page names `import` nowhere, so no case below reads it.

# 6a. The population, read out of the page. Delete the invocation and every case
#     below it would otherwise judge nothing and pass.
vendor_cmds=$(vendor_invocations_of "$readme")
vendor_count=$(printf '%s' "$vendor_cmds" | grep -c . || true)
more_than "the page states a \`taxonomy vendor\` invocation" 0 "$vendor_count"

# 6b. An engine, or red. This suite ran none until this group, and the CI step
#     that runs it sits in the job that builds one.
if [ -n "$engine" ]; then
    pass "  and an engine is built to run it against"
else
    fail "  and an engine is built to run it against" \
        "neither \`engine/target/release/headwater\` nor \`engine/target/dev-release/headwater\` is executable, so the arms below are undecided rather than passing. Build one with \`cargo build --profile dev-release -p headwater-cli --manifest-path engine/Cargo.toml --locked\`"
fi

vendor_cmd=$(printf '%s\n' "$vendor_cmds" | head -1)
operand=$(vendor_operand_of "$vendor_cmd")
good_digest=$(release_digest_of "$root/packages/headwater-standard/release.yml")
bad_digest="${good_digest%%:*}:$(printf '%064d' 0)"

# 6c. The grammar the interface contract publishes. This is the syntactic half,
#     and 6f is the half that runs.
if [ -n "$operand" ]; then
    pass "  and it names a directory operand ($operand)"
else
    fail "  and it names a directory operand" \
        "\`$vendor_cmd\` names none, and \`docs/interfaces/headwater-taxonomy.md\` states the grammar as \`vendor <dir> [--expect <digest>]\`"
fi

# 6d. A reader who has run the four lines of the paste block has the directory
#     the page then names, or the command is unrunnable for a second reason.
if [ -n "$operand" ] && [ -d "$root/$operand" ]; then
    pass "  and that directory is in the checkout the paste block produced"
else
    fail "  and that directory is in the checkout the paste block produced" \
        "no directory \`$operand\` at the root of this repository"
fi

# 6e. The digest the reader pastes has somewhere to go.
case " $vendor_cmd " in
    *" --expect "*) pass "  and it passes the stated digest through \`--expect\`" ;;
    *) fail "  and it passes the stated digest through \`--expect\`" \
        "\`$vendor_cmd\` carries no \`--expect\`, so the digest the paragraph above it tells the reader to paste is not checked" ;;
esac

# 6f. The three properties that fail together, run over a scratch copy of the
#     checkout. The digest is read from the release record rather than written
#     here, so a republished package moves this case with it.
if [ -n "$engine" ] && [ -n "$operand" ] && [ -d "$root/$operand" ] && [ -n "$good_digest" ]; then
    if [ "$good_digest" = "$bad_digest" ]; then
        fail "  the stated digest is accepted, a wrong one is refused, and the two reports differ" \
            "the corrupted digest equals the stated one, so the comparison below would prove nothing"
    else
        vendor_corpus "$scratch/vendor-real" "$operand" || exit 1
        verdict=$(vendor_judge "$scratch/vendor-real" "$vendor_cmd" "$good_digest")
        if [ "$verdict" = ok ]; then
            pass "  the stated digest is accepted, a wrong one is refused, and the two reports differ"
        else
            fail "  the stated digest is accepted, a wrong one is refused, and the two reports differ" \
                "$verdict — the arm carrying the stated digest printed: $(head -1 "$scratch/vendor.match")"
        fi
    fi
else
    fail "  the stated digest is accepted, a wrong one is refused, and the two reports differ" \
        "the arms did not run: engine \`${engine:-none}\`, operand \`${operand:-none}\`, digest \`${good_digest:-none}\`"
fi

# 6g-6h. The judge, provoked, in the two shapes it refuses. A gate that refuses
#        everything is as useless as one that refuses nothing, and each red arm
#        carries its own sentence so the verdict says which way the page went
#        wrong. 6g is the state this page was in: no operand, so the engine
#        refuses on the grammar before it reads `--expect`, and a reader with a
#        correct digest and a reader with a mistyped one see the same bytes.
#
#        Both arms name their own invocation, so neither reads the page and
#        both stay green while 6f is red. That is deliberate: a page that breaks
#        6f must still leave a working judge behind, or the failure above cannot
#        be told from a judge that stopped working.
mkdir -p "$scratch/vendor-arms"
if [ -n "$engine" ] && [ -n "$good_digest" ]; then
    vendor_corpus "$scratch/vendor-arms/corpus" packages/headwater-standard || exit 1
    same "  an invocation with no directory refuses both digests in the same words" \
        "the digest the release states is refused; the two reports are byte-identical" \
        "$(vendor_judge "$scratch/vendor-arms/corpus" "headwater taxonomy vendor --expect <digest>" "$good_digest")"
    same "  an invocation with a directory and no \`--expect\` checks no digest at all" \
        "a wrong digest is accepted; the two reports are byte-identical" \
        "$(vendor_judge "$scratch/vendor-arms/corpus" "headwater taxonomy vendor packages/headwater-standard" "$good_digest")"
    same "  an invocation naming a directory that is not there never reaches the digest" \
        "the digest the release states is refused; the two reports are byte-identical" \
        "$(vendor_judge "$scratch/vendor-arms/corpus" "headwater taxonomy vendor packages/not-a-package --expect <digest>" "$good_digest")"
else
    fail "  the judge is provoked in both shapes it refuses" \
        "the arms did not run: engine \`${engine:-none}\`, digest \`${good_digest:-none}\`"
fi

echo "the binary the page offers, and the run that uploads it"

# The second group here that reads something other than the page, and it reads
# it for the same reason group 6 runs a command: the claim is about a thing
# outside this file, and a judge that only reads the page cannot see it.
#
# What it holds is one string in two places. `.github/workflows/release.yml`
# names the asset it uploads; `README.md` names the asset it tells a stranger to
# download. Nothing else compares them, and nothing else can: the workflow runs
# on a tag and a tag is not pushed by a pull request, so the whole file is
# unexercised until the day somebody cuts a release, and the reader who finds
# the mismatch is the stranger following the page to a 404.
#
# The four cases below are therefore the only run either side ever gets before a
# tag exists, and each one is provoked in a scratch copy underneath.

release_wf="$root/.github/workflows/release.yml"

# 7a. The population. A missing workflow is red rather than skipped, because
#     every case below it would otherwise judge two empty sets and agree.
if [ -f "$release_wf" ]; then
    pass "a release workflow exists (.github/workflows/release.yml)"
else
    fail "a release workflow exists (.github/workflows/release.yml)" \
        "no such file, so the four cases below have nothing to compare the page against"
fi

# 7b-7e. The real page against the real workflow.
same "the page names the asset the workflow uploads" ok \
    "$(release_asset_judge "$readme" "$release_wf")"
same "  and the checksum beside it" ok \
    "$(release_checksum_judge "$readme" "$release_wf")"
same "  and a tag of the form the fence pins starts that workflow" ok \
    "$(release_trigger_judge "$release_wf" "$tag")"
same "  and the job that uploads may write to the release" ok \
    "$(release_write_judge "$release_wf")"
same "  and a tag already cut can be given one by hand" ok \
    "$(release_dispatch_judge "$release_wf")"
same "  and the upload hands over exactly what the page offers" ok \
    "$(release_upload_judge "$readme" "$release_wf")"
same "  and no step reads a command through an unguarded pipe" "" \
    "$(release_pipe_steps "$release_wf" offenders | tr '\n' '|')"
more_than "  over the asset names the page carries" 0 \
    "$(release_archives_of "$readme" page | grep -c .)"
more_than "  and the asset names the workflow carries" 0 \
    "$(release_archives_of "$release_wf" workflow | grep -c .)"
more_than "  and the names the upload command hands over" 0 \
    "$(release_uploaded_names "$release_wf" | grep -c .)"
more_than "  and the steps whose pipe is guarded" 0 \
    "$(release_pipe_steps "$release_wf" guarded | grep -c .)"

# 7f-7k. The judges, provoked. Each arm mutates ONE side in a scratch copy and
#        requires the verdict to name which way the pair went wrong. The
#        expected sentences are built from the names read out of the real files
#        and the same substitution the mutation makes, so nothing here writes an
#        asset name down: a page that ships a different triple tomorrow is
#        judged against that triple.
mkdir -p "$scratch/release"
if [ -f "$release_wf" ]; then
    real_name=$(release_archives_of "$readme" page)
    sed 's/x86_64/aarch64/g' "$release_wf" >"$scratch/release/other-arch.yml"
    if cmp -s "$release_wf" "$scratch/release/other-arch.yml"; then
        fail "  a workflow uploading a different triple is named on both sides" \
            "the planted edit changed nothing, so this case measured nothing"
    else
        same "  a workflow uploading a different triple is named on both sides" \
            "the page names $(oneline "$real_name") and the workflow uploads $(oneline "$(printf '%s\n' "$real_name" | sed 's/x86_64/aarch64/g')")" \
            "$(release_asset_judge "$readme" "$scratch/release/other-arch.yml")"
    fi

    grep -v 'tar\.gz' "$readme" >"$scratch/release/no-download.md"
    same "  a page that offers no download is not agreement" \
        "the workflow uploads an asset and the page names none" \
        "$(release_asset_judge "$scratch/release/no-download.md" "$release_wf")"

    grep -v 'contents: write' "$release_wf" >"$scratch/release/read-only.yml"
    same "  a job with no \`contents: write\` is refused" \
        "no job declares \`contents: write\`, so the upload runs with a read-only token" \
        "$(release_write_judge "$scratch/release/read-only.yml")"

    sed "s/'v\*'/'w*'/" "$release_wf" >"$scratch/release/wrong-glob.yml"
    same "  a trigger that the pinned tag does not match is refused" \
        "the tag the fence pins ($tag) matches none of $(oneline "$(release_tag_patterns "$scratch/release/wrong-glob.yml")")" \
        "$(release_trigger_judge "$scratch/release/wrong-glob.yml" "$tag")"

    grep -v 'tags:' "$release_wf" | grep -v "'v\*'" >"$scratch/release/no-tags.yml"
    same "  a workflow with no tag trigger at all is refused" \
        "the workflow triggers on no tag pattern, so pushing a tag runs nothing" \
        "$(release_trigger_judge "$scratch/release/no-tags.yml" "$tag")"

    grep -v 'workflow_dispatch:' "$release_wf" >"$scratch/release/no-dispatch.yml"
    same "  a workflow with no hand entry point is refused" \
        "no \`workflow_dispatch\`, so a tag already cut can never be given an asset" \
        "$(release_dispatch_judge "$scratch/release/no-dispatch.yml")"

    printf '%s\n' '# The name lives in a comment only' \
        '# headwater-<tag>-x86_64-unknown-linux-gnu.tar.gz' \
        '#   contents: write' >"$scratch/release/comments.yml"
    same "  a name that lives only in a comment uploads nothing" \
        "the page names an asset and the workflow uploads none" \
        "$(release_asset_judge "$readme" "$scratch/release/comments.yml")"
    same "  and a permission that lives only in a comment grants nothing" \
        "no job declares \`contents: write\`, so the upload runs with a read-only token" \
        "$(release_write_judge "$scratch/release/comments.yml")"

    # The two edits that leave every declared name in place and still send the
    # reader to a 404. A review of the first cut of this group found both: the
    # judges above read what the file writes down, and neither of these changes
    # a single name it writes down.
    grep -v 'gh release upload' "$release_wf" >"$scratch/release/no-upload.yml"
    same "  a workflow that declares the names and uploads neither is refused" \
        "no \`gh release upload\` hands over an asset, so the tag gets nothing" \
        "$(release_upload_judge "$readme" "$scratch/release/no-upload.yml")"

    sed 's/upload "\$TAG" "\$asset" "\$checksum"/upload "$TAG" "$checksum"/' \
        "$release_wf" >"$scratch/release/checksum-only.yml"
    if cmp -s "$release_wf" "$scratch/release/checksum-only.yml"; then
        fail "  a workflow that uploads the checksum and not the archive is refused" \
            "the planted edit changed nothing, so this case measured nothing"
    else
        same "  a workflow that uploads the checksum and not the archive is refused" \
            "the page offers $(oneline "$(release_names_raw "$readme" page | LC_ALL=C sort -u)") and the upload attaches $(oneline "$(release_checksums_of "$readme" page)")" \
            "$(release_upload_judge "$readme" "$scratch/release/checksum-only.yml")"
    fi

    sed 's/upload "\$TAG" "\$asset"/upload "$TAG" "$archive"/' \
        "$release_wf" >"$scratch/release/renamed-var.yml"
    if [ "$(release_upload_judge "$readme" "$scratch/release/renamed-var.yml")" = ok ]; then
        fail "  an operand naming a variable nothing assigns is refused" \
            "the judge returned \`ok\` for an upload whose first operand resolves to nothing"
    else
        pass "  an operand naming a variable nothing assigns is refused"
    fi

    # The guard on the guard. `bash -e` without `pipefail` takes the LAST
    # command's status, so a version check written as `$(binary | awk …)` passes
    # for every tag when the binary refuses to run. That is what this workflow
    # did, and no judge above could see it, because the defect is in a shell
    # option and not in a name.
    guarded=$(release_pipe_steps "$release_wf" guarded | tr '\n' '|')
    sed 's/^          set -eo pipefail$//' "$release_wf" >"$scratch/release/no-pipefail.yml"
    if cmp -s "$release_wf" "$scratch/release/no-pipefail.yml"; then
        fail "  a step reading a command through an unguarded pipe is named" \
            "the planted edit changed nothing, so this case measured nothing"
    else
        same "  a step reading a command through an unguarded pipe is named" \
            "$guarded" \
            "$(release_pipe_steps "$scratch/release/no-pipefail.yml" offenders | tr '\n' '|')"
    fi
else
    fail "  the judges are provoked in the shapes they refuse" \
        "the arms did not run: no \`.github/workflows/release.yml\` to mutate"
fi

echo
echo "$passed passed, $failed failed"
[ "$failed" -eq 0 ]
