#!/bin/sh
# What holds the library index: the admission section of `docs/taxonomies/README.md`,
# and the vendored copy of that page inside the published package.
#
# Run it from anywhere:
#     sh tools/repo/library-index-fixtures.sh
#
# # THE TWO DEFECTS THIS SUITE EXISTS FOR
#
# ## An entry the index never names, which an adopter is still made to select
#
# The library carried six bundles and the admission table listed five. The
# missing one, `evidence-and-obligation`, is not decoration: `design-spec` and
# `decision-record` both declare `requires: [evidence-and-obligation]`, and the
# `headwater/starter` recipe selects it. So a composer reading the index to
# choose entries had to select a bundle the index gave no name, no link and no
# reason for. Nothing reported that. The lock names no `README.md`, no rule of
# the engine reads a table, and `taxonomy resolve --check` and
# `headwater check --strict` both exited 0 over the omission.
#
# The count was not the defect. The section also narrated itself in ordinals —
# "the second entry", "the fifth entry" — and an ordinal is a count wearing a
# different hat. A row added to the table would have left every one of them
# standing and wrong. So this suite writes down no number of bundles, no number
# of rows and no number of carried files. It asserts SET EQUALITY between two
# populations it reads out of the tree, in both directions, so a bundle added
# without an index entry reddens and an index entry for a bundle that is gone
# reddens too.
#
# ## Source prose and vendored prose drifting apart with every gate green
#
# `docs/taxonomies/README.md` is the authored page and
# `packages/headwater-standard/bundles/README.md` is the vendored copy that a
# consumer of `headwater/standard` actually receives. Measured on 2026-09-07:
# appending one line to the authored page alone left `taxonomy resolve --check`
# at exit 0 and `headwater check --strict` at exit 0 with a byte-identical
# report; appending one line to the vendored copy alone did the same. The lock
# names no `README.md` and no `doctrine.md`, and the release digest is compared
# only at `vendor` time, so nothing in the commit gate or in CI reads either
# copy. The two halves of a published page could disagree indefinitely.
#
# Case group 3 compares them by bytes, in both directions, over the population
# the anatomy table itself declares as carried.
#
# # WHAT EACH JUDGE READS
#
# Nothing below is a list of bundle names, entry names or file names. Every
# population is enumerated:
#
#   THE BUNDLES come from `contents.bundles` in
#   `taxonomy-source/headwater-standard/package.yml` — the scalar
#   `headwater_resolve` itself reads — and are the directories under that path
#   which carry a `bundle.yml`. Not a directory listing, because a directory
#   with no `bundle.yml` is not a bundle.
#
#   THE ACCOUNTED-FOR ENTRIES come from the admission section of the same page:
#   the first cell of each table row, plus every bare directory link written
#   anywhere else in the section. A bundle the table admits is accounted for by
#   its row. A bundle the table refuses is accounted for by the paragraph that
#   names it and says why. Either one satisfies the section; neither one is
#   assumed.
#
#   THE CARRIED PARTS come from the "What an entry ships" table on the same
#   page. Each row names a part, a path in the draft, and where it lands when
#   the entry is published. A row whose destination begins "nowhere" is not
#   carried, which is how `fixtures/` leaves itself out of case group 3 without
#   this file naming it.
#
# # THE POPULATION GUARDS, WHICH ARE NOT COUNTS
#
# Each group opens with a floor of zero. A judge whose population came back
# empty reports green for the wrong reason, and a page whose table stopped
# parsing would otherwise look like a page with nothing wrong. A floor is not a
# count of anything: no expected number is written down here, and none can go
# stale.
#
# # WHAT IT NEEDS, AND WHAT IT WRITES
#
# `awk`, `sed` and `cmp`. It builds no engine, runs none, and opens no socket,
# which is why it can be a required step. Every scratch tree is made under
# `mktemp -d`, the directory goes on an interrupt, and nothing inside this
# checkout is written.
#
# # WHAT THIS SUITE DOES NOT HOLD
#
# The prose. `docs/taxonomies/README.md` classifies to no kind, so no language
# rule of this engine reads it, and this suite reimplements none of them — a
# second copy of a rule living in a script is what this repository refuses
# everywhere else. Whether the regime should reach this page is a question
# about the taxonomy and not a gap here.
#
# It also holds no reading of the seven admission criteria. Whether a bundle
# MERITS a row is a judgment a reviewer makes and a ruling settles. This suite
# holds only that the section accounts for every bundle one way or the other.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
manifest="$root/taxonomy-source/headwater-standard/package.yml"
index="$root/docs/taxonomies/README.md"
vendored="$root/packages/headwater-standard/bundles/README.md"

for required in "$manifest" "$index" "$vendored"; do
    if [ ! -f "$required" ]; then
        echo "no \`${required#$root/}\`, so every case here has nothing to judge." >&2
        echo "  This suite stops rather than reporting a row of passes over a" >&2
        echo "  population it could not read." >&2
        exit 1
    fi
done

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
# that measured nothing. This is a floor and never an expected value: no case
# below asserts how many bundles, rows or files there are.
more_than() {
    if [ "$3" -gt "$2" ] 2>/dev/null; then
        pass "$1 ($3)"
    else
        fail "$1" "expected more than $2, got \`$3\`"
    fi
}

# bundles_root MANIFEST — the `contents.bundles` scalar, resolved against the
# directory the manifest sits in. This is the path `headwater_resolve` reads
# `<bundles>/<name>/bundle.yml` under, and it is deliberately not a hard-coded
# `docs/taxonomies`: a publish rewrites this one scalar, so the vendored copy
# of the same manifest names `bundles` instead.
bundles_root() {
    br_dir=$(dirname "$1")
    br_val=$(awk '
        /^contents:/ { inc = 1; next }
        inc && /^[^ #]/ { inc = 0 }
        inc && /^[ \t]+bundles:[ \t]*/ {
            sub(/^[ \t]+bundles:[ \t]*/, "")
            sub(/[ \t]*#.*$/, "")
            sub(/[ \t]+$/, "")
            print
            exit
        }
    ' "$1")
    [ -n "$br_val" ] || return 1
    (cd "$br_dir/$br_val" 2>/dev/null && pwd)
}

# bundles_of ROOT — one name per line, sorted: a directory under ROOT that
# carries a `bundle.yml`. A directory without one is not a bundle and is not in
# the population.
bundles_of() {
    for bo_dir in "$1"/*/; do
        [ -f "$bo_dir/bundle.yml" ] || continue
        bo_name=${bo_dir%/}
        echo "${bo_name##*/}"
    done | sort
}

# bundle_key FILE — the `bundle:` key a `bundle.yml` declares.
bundle_key() {
    awk '/^bundle:[ \t]*/ {
        sub(/^bundle:[ \t]*/, "")
        sub(/[ \t]*#.*$/, "")
        sub(/[ \t]+$/, "")
        print
        exit
    }' "$1"
}

# section_of FILE HEADING-PREFIX — the body of one `##` section, from the
# heading that starts with the given text to the next heading of the same level
# or the end of the file. Written to stdout.
section_of() {
    awk -v want="$2" '
        /^## / {
            if (index($0, "## " want) == 1) { inside = 1; next }
            if (inside) exit
        }
        inside { print }
    ' "$1"
}

# rows_of SECTION-FILE — one line per table row, as
# `entry<TAB>bundle-cell<TAB>tradition-cell`. A row is a `|` line whose first
# cell carries a bare directory link, which excludes the header row and the
# delimiter row without either one being named here. The entry is the link
# target with its trailing slash removed; the other two cells are stripped of
# backticks and surrounding space.
rows_of() {
    awk -F'|' '
        function trim(s) { sub(/^[ \t]+/, "", s); sub(/[ \t]+$/, "", s); return s }
        function bare(s,   i, j, t) {
            i = index(s, "](")
            if (i == 0) return ""
            t = substr(s, i + 2)
            j = index(t, ")")
            if (j == 0) return ""
            t = substr(t, 1, j - 1)
            if (t !~ /^[A-Za-z0-9][A-Za-z0-9._-]*\/$/) return ""
            sub(/\/$/, "", t)
            return t
        }
        /^[ \t]*\|/ && NF >= 4 {
            entry = bare($2)
            if (entry == "") next
            b = trim($3); gsub(/`/, "", b)
            t = trim($4)
            print entry "\t" b "\t" t
        }
    ' "$1"
}

# links_outside_rows SECTION-FILE — every bare directory link written on a line
# that is not a table row, one name per line, sorted and deduplicated. This is
# how a bundle the table refuses is accounted for: a paragraph names it and
# links it. A link that carries a file name or a fragment, such as
# `evidence-and-obligation/doctrine.md#findings`, is not a bare directory link
# and does not account for anything.
links_outside_rows() {
    awk '
        function bare(t) {
            if (t !~ /^[A-Za-z0-9][A-Za-z0-9._-]*\/$/) return ""
            sub(/\/$/, "", t)
            return t
        }
        /^[ \t]*\|/ { next }
        {
            rest = $0
            while ((i = index(rest, "](")) > 0) {
                rest = substr(rest, i + 2)
                j = index(rest, ")")
                if (j == 0) break
                n = bare(substr(rest, 1, j - 1))
                if (n != "") print n
                rest = substr(rest, j + 1)
            }
        }
    ' "$1" | sort -u
}

# accounted_of SECTION-FILE — the entries the section accounts for: a row's
# first cell, or a bare directory link in its prose.
accounted_of() {
    { rows_of "$1" | cut -f1; links_outside_rows "$1"; } | sort -u
}

# accounted_judge BUNDLES-FILE ACCOUNTED-FILE — one line per member of the
# symmetric difference, naming which side it is missing from. Empty output is
# set equality.
accounted_judge() {
    comm -23 "$1" "$2" | sed 's/^/a bundle the admission section does not account for: /'
    comm -13 "$1" "$2" | sed 's/^/an entry the admission section accounts for that is no bundle: /'
}

# row_judge BUNDLES-ROOT SECTION-FILE — one line per row that does not hold up:
# a link target that carries no `bundle.yml`, a Bundle cell that disagrees with
# either the directory name or the `bundle:` key inside it, or a tradition cell
# with nothing in it. Criterion 1 gives that third column its heading, so a row
# with an empty cell is a row claiming a tradition it does not name.
row_judge() {
    rj_root=$1
    rows_of "$2" | while IFS='	' read -r entry cell tradition; do
        if [ ! -f "$rj_root/$entry/bundle.yml" ]; then
            echo "$entry: a row whose link resolves to no bundle"
            continue
        fi
        rj_key=$(bundle_key "$rj_root/$entry/bundle.yml")
        [ "$cell" = "$entry" ] || echo "$entry: the Bundle cell reads \`$cell\`"
        [ "$cell" = "$rj_key" ] || echo "$entry: the Bundle cell disagrees with \`bundle: $rj_key\`"
        [ -n "$tradition" ] || echo "$entry: a row that names no tradition"
    done
}

# add_keys FILE — the top-level addresses a `bundle.yml` writes under `add:`,
# one per line, sorted. A key is a two-space-indented dotted path followed by a
# colon, so `regimes.voice.narrative: {}` and `kinds.evaluation:` are both one
# address and anything nested under them is not.
add_keys() {
    awk '
        /^add:/ { ina = 1; next }
        ina && /^[^ #]/ { ina = 0 }
        ina && /^  [A-Za-z][A-Za-z0-9_.]*:/ {
            sub(/^  /, "")
            sub(/:.*$/, "")
            print
        }
    ' "$1" | sort -u
}

# address_judge BUNDLES-ROOT SECTION-FILE — one line per address that a bundle
# WITHOUT A TABLE ROW declares and the section never writes.
#
# A bundle the table admits has a row, a link and a tradition, and a reader
# follows the link for the rest. A bundle the table refuses has none of that,
# so the paragraph naming it is the only place a composer learns what selecting
# it brings. An address left out of that paragraph is invisible, which is the
# defect this case exists for: the first draft of that paragraph listed four of
# the five addresses `evidence-and-obligation` declares and omitted
# `shelves.evaluations`, and nothing anywhere would have reported it.
#
# Only this direction is judged. The reverse — an address written in the
# section that no bundle declares — would refuse `kinds.decision` and
# `voice.declarative`, which the base declares, and the placeholder
# `kinds.<k>.facets.require`. A required check that reddens on correct prose is
# a check the first person it annoys turns off.
address_judge() {
    aj_root=$1
    aj_section=$2
    rows_of "$aj_section" | cut -f1 | sort -u >"$scratch/aj.rows"
    bundles_of "$aj_root" | while read -r aj_b; do
        grep -qx "$aj_b" "$scratch/aj.rows" && continue
        add_keys "$aj_root/$aj_b/bundle.yml" | while read -r aj_k; do
            grep -qF -- "\`$aj_k\`" "$aj_section" ||
                echo "$aj_b: the section never writes \`$aj_k\`"
        done
    done
}

# carried_parts INDEX-FILE — the parts of an entry that a publish carries, one
# per line, read out of the "What an entry ships" table. Column 2 is the path
# in the draft and column 3 says where it lands; a destination beginning
# "nowhere" is a part the artifact does not hold. A directory part keeps its
# trailing slash.
carried_parts() {
    section_of "$1" "What an entry ships" | awk -F'|' '
        function trim(s) { sub(/^[ \t]+/, "", s); sub(/[ \t]+$/, "", s); return s }
        /^[ \t]*\|/ && NF >= 4 {
            p = trim($3); gsub(/`/, "", p)
            d = trim($4)
            if (p == "" || p ~ /^-+$/ || p == "Path in the draft") next
            if (tolower(d) ~ /^nowhere/) next
            print p
        }
    '
}

# files_under DIR — every regular file under DIR, as a path relative to DIR.
files_under() {
    [ -d "$1" ] || return 0
    (cd "$1" && find . -type f | sed 's|^\./||' | sort)
}

# carried_judge SRC VEND PARTS-FILE — one line per carried file that is missing
# from one side or differs in bytes between them. Both directions, so a file
# the vendored copy holds and the source does not is reported too. The index
# page itself is compared as well, because that page is the one both defects
# above were found on.
carried_judge() {
    cj_src=$1
    cj_vend=$2
    cj_parts=$3
    cj_compare() {
        if [ ! -f "$cj_src/$1" ]; then
            echo "$1: carried by the artifact and absent from the source"
        elif [ ! -f "$cj_vend/$1" ]; then
            echo "$1: authored in the source and absent from the artifact"
        elif ! cmp -s "$cj_src/$1" "$cj_vend/$1"; then
            echo "$1: the source and the vendored copy differ"
        fi
    }
    cj_compare "README.md"
    for cj_b in $(bundles_of "$cj_src") $(bundles_of "$cj_vend"); do
        echo "$cj_b"
    done | sort -u | while read -r cj_b; do
        while read -r cj_part; do
            case $cj_part in
                */)
                    { files_under "$cj_src/$cj_b/${cj_part%/}"
                      files_under "$cj_vend/$cj_b/${cj_part%/}"; } | sort -u |
                    while read -r cj_f; do
                        cj_compare "$cj_b/$cj_part$cj_f"
                    done
                    ;;
                *) cj_compare "$cj_b/$cj_part" ;;
            esac
        done <"$cj_parts"
    done
}

# ---------------------------------------------------------------------------

lib=$(bundles_root "$manifest") || lib=
if [ -z "$lib" ] || [ ! -d "$lib" ]; then
    echo "\`contents.bundles\` in the package manifest names no directory that" >&2
    echo "  exists, so the bundle population cannot be read. This suite stops" >&2
    echo "  rather than reporting passes over an empty set." >&2
    exit 1
fi

section_of "$index" "Admission, and what is admitted" >"$scratch/section"
bundles_of "$lib" >"$scratch/bundles"
accounted_of "$scratch/section" >"$scratch/accounted"
carried_parts "$index" >"$scratch/parts"

echo "the bundles this library carries, and the entries its index accounts for"

# 1a. Both populations, and the parts population, are non-empty. A judge whose
#     input went empty passes everything below it for the wrong reason.
more_than "the manifest names a bundles directory that holds bundles" 0 \
    "$(wc -l <"$scratch/bundles" | tr -d ' ')"
more_than "the admission section accounts for entries" 0 \
    "$(wc -l <"$scratch/accounted" | tr -d ' ')"
more_than "the anatomy table names parts a publish carries" 0 \
    "$(wc -l <"$scratch/parts" | tr -d ' ')"

# 1b. Set equality, both directions. This is the case #510 was filed for.
same "every bundle is accounted for, and every entry accounted for is a bundle" \
    "" "$(accounted_judge "$scratch/bundles" "$scratch/accounted" | tr '\n' '|')"

# 1c. Each row resolves, agrees with its directory and with the `bundle:` key
#     inside it, and names a tradition.
same "every admitted row resolves and names its bundle and its tradition" \
    "" "$(row_judge "$lib" "$scratch/section" | tr '\n' '|')"

# 1d. Every bundle a `requires:` line names, and every bundle the publisher's
#     own assembly recipes select, is accounted for by the section. This is the
#     half that makes the omission adopter-facing rather than cosmetic: a
#     composer who takes an admitted entry is made to take its closure too.
{
    for req in "$lib"/*/bundle.yml; do
        [ -f "$req" ] || continue
        sed -n 's/^requires:[ \t]*\[\(.*\)\][ \t]*$/\1/p' "$req" |
            tr ',' '\n' | sed 's/[][ "'"'"']//g' | grep -v '^$' || true
    done
    for rec in "$root"/taxonomy-source/headwater-standard/assemblies/*/assembly.yml; do
        [ -f "$rec" ] || continue
        sed -n 's/^[ \t]*bundles:[ \t]*\[\(.*\)\][ \t]*$/\1/p' "$rec" |
            tr ',' '\n' | sed 's/[][ "'"'"']//g' | grep -v '^$' || true
    done
} | sort -u >"$scratch/depended"
more_than "some bundle is required by another or selected by a recipe" 0 \
    "$(wc -l <"$scratch/depended" | tr -d ' ')"
same "every bundle a requires line or a recipe names is accounted for" \
    "" "$(comm -23 "$scratch/depended" "$scratch/accounted" |
          sed 's/^/an entry a consumer is made to select and the index never names: /' |
          tr '\n' '|')"

# 1e. A bundle the table refuses has no row to follow, so the paragraph that
#     names it is the whole of what a composer learns about it. Every address
#     it declares is written there, enumerated out of its `bundle.yml`. This
#     case exists because the first draft of that paragraph listed four of five
#     and omitted `shelves.evaluations`.
same "every address a refused bundle declares is written in the section" \
    "" "$(address_judge "$lib" "$scratch/section" | tr '\n' '|')"

echo
echo "the judges, provoked over scratch trees"

mkdir -p "$scratch/lib/alpha" "$scratch/lib/beta" "$scratch/lib/loose"
printf 'bundle: alpha\nrequires: []\n\nadd:\n  kinds.one:\n    purpose: rationale\n' \
    >"$scratch/lib/alpha/bundle.yml"
printf 'bundle: beta\nrequires: []\n\nadd:\n  kinds.two:\n    purpose: rationale\n  shelves.two: {}\n' \
    >"$scratch/lib/beta/bundle.yml"
printf 'not a bundle\n' >"$scratch/lib/loose/notes.md"

write_section() {
    ws_out=$1
    shift
    printf '## Admission, and what is admitted\n\n' >"$ws_out"
    printf '%s\n' "$@" >>"$ws_out"
}

# 2a. A directory with no `bundle.yml` is not in the population, so an index
#     that never mentions `loose` is not wrong about it.
write_section "$scratch/arms.a" \
    '| Entry | Bundle | The tradition |' \
    '|---|---|---|' \
    '| [`alpha`](alpha/) | `alpha` | A tradition |' \
    '| [`beta`](beta/) | `beta` | Another tradition |'
accounted_of "$scratch/arms.a" >"$scratch/arms.a.acc"
bundles_of "$scratch/lib" >"$scratch/arms.lib"
same "  a directory carrying no bundle.yml is not a bundle" \
    "" "$(accounted_judge "$scratch/arms.lib" "$scratch/arms.a.acc" | tr '\n' '|')"

# 2b. A bundle with no row and no paragraph link. This is #510 exactly.
write_section "$scratch/arms.b" \
    '| Entry | Bundle | The tradition |' \
    '|---|---|---|' \
    '| [`alpha`](alpha/) | `alpha` | A tradition |'
accounted_of "$scratch/arms.b" >"$scratch/arms.b.acc"
same "  a bundle the section never names is refused" \
    "a bundle the admission section does not account for: beta|" \
    "$(accounted_judge "$scratch/arms.lib" "$scratch/arms.b.acc" | tr '\n' '|')"

# 2c. The same bundle, accounted for by a paragraph rather than by a row. A
#     refusal stated in prose satisfies the section, which is the shape this
#     library uses for a bundle that meets no tradition to put in column three.
write_section "$scratch/arms.c" \
    '| Entry | Bundle | The tradition |' \
    '|---|---|---|' \
    '| [`alpha`](alpha/) | `alpha` | A tradition |' \
    '' \
    'The [`beta`](beta/) bundle is not an admitted entry, and `alpha` requires it.'
accounted_of "$scratch/arms.c" >"$scratch/arms.c.acc"
same "  a paragraph that names and links a bundle accounts for it" \
    "" "$(accounted_judge "$scratch/arms.lib" "$scratch/arms.c.acc" | tr '\n' '|')"

# 2c'. That same paragraph accounts for the bundle and still leaves a composer
#      without one of the two addresses selecting it brings. A row-bearing
#      bundle is exempt: `alpha` declares `kinds.one` and no paragraph writes
#      it, and the judge is silent about that because `alpha` has a row to
#      follow.
same "  a paragraph that names a bundle and omits one of its addresses is refused" \
    "beta: the section never writes \`kinds.two\`|beta: the section never writes \`shelves.two\`|" \
    "$(address_judge "$scratch/lib" "$scratch/arms.c" | tr '\n' '|')"
write_section "$scratch/arms.c2" \
    '| Entry | Bundle | The tradition |' \
    '|---|---|---|' \
    '| [`alpha`](alpha/) | `alpha` | A tradition |' \
    '' \
    'The [`beta`](beta/) bundle is not an admitted entry. It declares `kinds.two` and `shelves.two`.'
same "  and the same paragraph writing both of them is clean" \
    "" "$(address_judge "$scratch/lib" "$scratch/arms.c2" | tr '\n' '|')"

# 2d. A link that carries a file name accounts for nothing, so a doctrine link
#     cannot stand in for the paragraph that owes the reason.
write_section "$scratch/arms.d" \
    '| Entry | Bundle | The tradition |' \
    '|---|---|---|' \
    '| [`alpha`](alpha/) | `alpha` | A tradition |' \
    '' \
    'See the [beta doctrine](beta/doctrine.md#findings) for the rest.'
accounted_of "$scratch/arms.d" >"$scratch/arms.d.acc"
same "  a link to a file inside a bundle does not account for the bundle" \
    "a bundle the admission section does not account for: beta|" \
    "$(accounted_judge "$scratch/arms.lib" "$scratch/arms.d.acc" | tr '\n' '|')"

# 2e. A row for a bundle that is gone.
write_section "$scratch/arms.e" \
    '| Entry | Bundle | The tradition |' \
    '|---|---|---|' \
    '| [`alpha`](alpha/) | `alpha` | A tradition |' \
    '| [`beta`](beta/) | `beta` | Another tradition |' \
    '| [`gamma`](gamma/) | `gamma` | A third tradition |'
accounted_of "$scratch/arms.e" >"$scratch/arms.e.acc"
same "  a row for a bundle that does not exist is refused" \
    "an entry the admission section accounts for that is no bundle: gamma|" \
    "$(accounted_judge "$scratch/arms.lib" "$scratch/arms.e.acc" | tr '\n' '|')"
same "  and the row judge names it too" \
    "gamma: a row whose link resolves to no bundle|" \
    "$(row_judge "$scratch/lib" "$scratch/arms.e" | tr '\n' '|')"

# 2f. A Bundle cell that disagrees with the directory and with the key inside
#     it, and a row with an empty tradition cell.
write_section "$scratch/arms.f" \
    '| Entry | Bundle | The tradition |' \
    '|---|---|---|' \
    '| [`alpha`](alpha/) | `alfa` | A tradition |' \
    '| [`beta`](beta/) | `beta` |  |'
same "  a Bundle cell that names the wrong bundle is refused, twice over" \
    "alpha: the Bundle cell reads \`alfa\`|alpha: the Bundle cell disagrees with \`bundle: alpha\`|beta: a row that names no tradition|" \
    "$(row_judge "$scratch/lib" "$scratch/arms.f" | tr '\n' '|')"

# 2g. A `bundle:` key inside the file that disagrees with its directory, which
#     the row cannot be right about in both places at once.
printf 'bundle: bravo\nrequires: []\n' >"$scratch/lib/beta/bundle.yml"
same "  a bundle.yml whose key disagrees with its directory is refused" \
    "beta: the Bundle cell disagrees with \`bundle: bravo\`|" \
    "$(row_judge "$scratch/lib" "$scratch/arms.a" | tr '\n' '|')"
printf 'bundle: beta\nrequires: []\n' >"$scratch/lib/beta/bundle.yml"

echo
echo "the authored page and the copy a consumer receives"

# 3a. The carried population is what the anatomy table declares, and the part
#     it declares as landing nowhere is absent from it. Reading that exclusion
#     off the page is what keeps `fixtures/` out of case 3b without this file
#     naming a directory.
same "the anatomy table keeps the part that lands nowhere out of the carried set" \
    "" "$(grep -c 'fixtures/' "$scratch/parts" | grep -v '^0$' || true)"
more_than "the carried set holds the schema, the doctrine and the templates" 1 \
    "$(wc -l <"$scratch/parts" | tr -d ' ')"

# 3b. Every carried file, in both directions, by bytes. Measured on
#     2026-09-07: one appended line on either side of this pair passes
#     `taxonomy resolve --check` and `headwater check --strict` at exit 0.
same "the source library and the vendored bundles are byte-identical where carried" \
    "" "$(carried_judge "$lib" "$root/packages/headwater-standard/bundles" \
            "$scratch/parts" | tr '\n' '|')"

# 3c. The drift judge, provoked in all three shapes it refuses, over a scratch
#     pair. A judge whose failure nobody has seen holds nothing.
mkdir -p "$scratch/src/alpha/templates" "$scratch/vend/alpha/templates"
printf 'index\n' >"$scratch/src/README.md"
printf 'index\n' >"$scratch/vend/README.md"
printf 'bundle: alpha\n' >"$scratch/src/alpha/bundle.yml"
printf 'bundle: alpha\n' >"$scratch/vend/alpha/bundle.yml"
printf 'doctrine\n' >"$scratch/src/alpha/doctrine.md"
printf 'doctrine\n' >"$scratch/vend/alpha/doctrine.md"
printf 'template\n' >"$scratch/src/alpha/templates/one.md"
printf 'template\n' >"$scratch/vend/alpha/templates/one.md"
mkdir -p "$scratch/src/alpha/fixtures"
printf 'a corpus the artifact never carries\n' >"$scratch/src/alpha/fixtures/case.md"
same "  a matched pair, with a source-only fixtures directory, is clean" \
    "" "$(carried_judge "$scratch/src" "$scratch/vend" "$scratch/parts" | tr '\n' '|')"

printf '\nA line that exists only in the authored source.\n' >>"$scratch/src/README.md"
same "  one appended line on the index page is drift" \
    "README.md: the source and the vendored copy differ|" \
    "$(carried_judge "$scratch/src" "$scratch/vend" "$scratch/parts" | tr '\n' '|')"
printf 'index\n' >"$scratch/src/README.md"

printf 'edited\n' >>"$scratch/vend/alpha/doctrine.md"
printf 'extra\n' >"$scratch/vend/alpha/templates/two.md"
rm "$scratch/vend/alpha/bundle.yml"
same "  a doctrine edited in the artifact, a template only it holds, and a schema only the source holds" \
    "alpha/bundle.yml: authored in the source and absent from the artifact|alpha/doctrine.md: the source and the vendored copy differ|alpha/templates/two.md: carried by the artifact and absent from the source|" \
    "$(carried_judge "$scratch/src" "$scratch/vend" "$scratch/parts" | tr '\n' '|')"

# ---------------------------------------------------------------------------
# 4. The `procedure` purpose, held as an exclusive or.
#
# The admission section records that no entry of this library serves the
# `procedure` purpose, and that which tradition should supply one is an open
# ruling. That sentence is a claim about the bundle sources, and nothing read
# it. `taxonomy resolve --check` and `headwater check --strict` both exit 0
# over this page either way, and this repository excludes `docs/taxonomies/**`
# from its own corpus, so no rule of the engine opens it at all.
#
# The failure this guards is the quiet one. A later entry admits a procedure
# kind, the ruling closes, and the paragraph goes on telling an adopter to
# write their own overlay because the library has nothing. Nobody rereads a
# gap notice on the day the gap closes.
#
# So the judge is an exclusive or over two populations it reads rather than
# lists: either a source declares the purpose, or the section records that none
# does. Never both, and never neither. Deleting the paragraph reddens this, and
# so does adding the purpose without touching the paragraph.
#
# What it reads for "the section records the gap" is the bare token `procedure`
# in backticks, anywhere in that section, and that proxy has one measured false
# positive. On 2026-09-11 the index gained a paragraph reading criterion 7
# against `diataxis-site`, which named the purpose the entry serves and recorded
# no gap at all. The judge refused it. The prose was reworded rather than the
# proxy loosened, because the arm that fires here has no anchor text to match on
# instead: this library declares the purpose today, so no gap paragraph exists
# to name. A later reader who does write one can tighten this to that sentence.

# procedure_declared FILE... — `yes` when any named source declares the
# `procedure` purpose or a kind that serves it, `no` otherwise. Comments are
# stripped first, because every argument for and against this purpose in this
# repository is written in a comment inside one of these same files, and a
# judge that read those would answer the opposite of the truth.
#
# Three shapes count, because the base package and a bundle write the same
# declaration differently. A `purposes:` block with a `procedure:` member is
# how the base declares one. A `purposes.procedure:` operation is how a bundle
# adds one. A `purpose: procedure` member is a kind that serves it, in block
# style or inside a flow mapping.
procedure_declared() {
    for pdc_f in "$@"; do
        [ -f "$pdc_f" ] || continue
        if awk '
            { sub(/#.*$/, "") }
            /^purposes:[ \t]*$/                          { inp = 1; next }
            /^[^ \t]/                                    { inp = 0 }
            inp && /^[ \t]+procedure[ \t]*:/             { found = 1 }
            /^[ \t]*purposes\.procedure[ \t]*:/          { found = 1 }
            /purpose[ \t]*:[ \t]*procedure([ \t,}]|$)/   { found = 1 }
            END { exit !found }
        ' "$pdc_f"; then
            echo yes
            return
        fi
    done
    echo no
}

# procedure_judge BASE BUNDLES-ROOT INDEX — one line per refusal, nothing when
# the two agree.
procedure_judge() {
    pj_sources="$1"
    for pj_b in "$2"/*/; do
        [ -f "$pj_b/bundle.yml" ] || continue
        pj_sources="$pj_sources $pj_b/bundle.yml"
    done
    # shellcheck disable=SC2086
    pj_declared=$(procedure_declared $pj_sources)
    if section_of "$3" "Admission, and what is admitted" \
        | grep -q '`procedure`'; then
        pj_recorded=yes
    else
        pj_recorded=no
    fi
    if [ "$pj_declared" = yes ] && [ "$pj_recorded" = yes ]; then
        echo "a source declares \`procedure\` and the index still records the gap"
    fi
    if [ "$pj_declared" = no ] && [ "$pj_recorded" = no ]; then
        echo "no source declares \`procedure\` and the index does not say so"
    fi
}

# 4a. This library, now. The paragraph stands and no source declares it.
same "the \`procedure\` gap and the index agree" \
    "" "$(procedure_judge "$root/taxonomy-source/headwater-standard/taxonomy.yml" \
            "$lib" "$index" | tr '\n' '|')"

# 4b. The vendored copy of the page is judged against the vendored sources, so
#     the artifact a consumer receives cannot record a different gap.
same "the vendored copy records the same gap as the vendored sources" \
    "" "$(procedure_judge "$root/packages/headwater-standard/taxonomy.yml" \
            "$root/packages/headwater-standard/bundles" "$vendored" | tr '\n' '|')"

# 4c. Both refusals, provoked over scratch files. A judge whose failure nobody
#     has seen holds nothing, and each half of an exclusive or fails for its
#     own reason.
mkdir -p "$scratch/pd/beta"
printf 'purposes:\n  rationale:\n    intent: why\n' >"$scratch/pd/base.yml"
printf 'bundle: beta\nadd:\n  kinds.note:\n    purpose: rationale\n' \
    >"$scratch/pd/beta/bundle.yml"
printf '## Admission, and what is admitted\n\nNothing here names the missing purpose.\n' \
    >"$scratch/pd/index.md"
same "  a library with no procedure purpose and an index that does not say so" \
    "no source declares \`procedure\` and the index does not say so|" \
    "$(procedure_judge "$scratch/pd/base.yml" "$scratch/pd/beta/.." \
        "$scratch/pd/index.md" | tr '\n' '|')"

printf '## Admission, and what is admitted\n\nNo entry serves the `procedure` purpose.\n' \
    >"$scratch/pd/index.md"
same "  the same library once the index records the gap" \
    "" "$(procedure_judge "$scratch/pd/base.yml" "$scratch/pd/beta/.." \
        "$scratch/pd/index.md" | tr '\n' '|')"

printf 'bundle: beta\nadd:\n  purposes.procedure:\n    intent: the steps\n' \
    >"$scratch/pd/beta/bundle.yml"
same "  a bundle that adds the purpose, under an index that still records the gap" \
    "a source declares \`procedure\` and the index still records the gap|" \
    "$(procedure_judge "$scratch/pd/base.yml" "$scratch/pd/beta/.." \
        "$scratch/pd/index.md" | tr '\n' '|')"

printf 'bundle: beta\nadd:\n  kinds.playbook: {purpose: procedure, lifecycle: standard}\n' \
    >"$scratch/pd/beta/bundle.yml"
same "  a kind serving the purpose in flow style is found the same way" \
    "a source declares \`procedure\` and the index still records the gap|" \
    "$(procedure_judge "$scratch/pd/base.yml" "$scratch/pd/beta/.." \
        "$scratch/pd/index.md" | tr '\n' '|')"

printf 'purposes:\n  procedure:\n    intent: the steps\n' >"$scratch/pd/base.yml"
printf 'bundle: beta\nadd:\n  kinds.note:\n    purpose: rationale\n' \
    >"$scratch/pd/beta/bundle.yml"
same "  the base declaring it in a nested block is found the same way" \
    "a source declares \`procedure\` and the index still records the gap|" \
    "$(procedure_judge "$scratch/pd/base.yml" "$scratch/pd/beta/.." \
        "$scratch/pd/index.md" | tr '\n' '|')"

# 4d. A comment naming the purpose is not a declaration. Every paragraph of
#     argument about this purpose in this repository sits in a comment, so a
#     judge that counted one would report the gap closed on the day somebody
#     wrote down why it is open.
printf 'purposes:\n  rationale:\n    intent: why\n' >"$scratch/pd/base.yml"
printf '# purposes.procedure: the ruling this entry waits on\n# kinds.playbook: {purpose: procedure}\nbundle: beta\nadd:\n  kinds.note:\n    purpose: rationale\n' \
    >"$scratch/pd/beta/bundle.yml"
same "  a comment naming the purpose declares nothing" \
    "" \
    "$(procedure_judge "$scratch/pd/base.yml" "$scratch/pd/beta/.." \
        "$scratch/pd/index.md" | tr '\n' '|')"

echo
echo "$passed passed, $failed failed"
[ "$failed" -eq 0 ]
