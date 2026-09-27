#!/bin/sh
# What holds `docs/how-to/cut-a-release.md`, the guide to cutting a release.
#
# Run it from anywhere:
#     sh tools/repo/release-guide-fixtures.sh
#
# # WHY THIS GUIDE NEEDS A SUITE
#
# A release is cut rarely, so the page that says how is read rarely, and a
# page nobody reads between two releases goes stale with no exit status
# moving. The release sequence lived in the head of whoever cut the last one
# until #982, and the part of that head most likely to be wrong next time is
# which workflow a tag starts. A tag pattern is a string in a workflow file,
# the guide is prose, and nothing else compares the two.
#
# `headwater check` reads the guide's language and its links. It does not read
# a code span as a claim about a file under `.github/workflows/`, and it never
# opens a workflow. So this suite holds the one part of the guide that is a
# copy of another file: the table of release workflows and the trigger each
# one fires on.
#
# # THE TWO POPULATIONS, IN BOTH DIRECTIONS
#
# The tree side is every `(workflow, trigger)` pair a release can start:
#
#   - each tag pattern listed under `on: push: tags:` in a workflow, and
#   - `workflow_dispatch` for each workflow that a person alone can start,
#     which is a workflow whose `on:` block names no other event. That is how
#     `yank-crates.yml` enters the population without being named here.
#
# The guide side is every row of a Markdown table in the guide whose first
# cell is a code span naming `.github/workflows/<name>.yml`, paired with each
# code span in its second cell.
#
# The two sets must be equal. A fourth release workflow added without a guide
# row is red, and a tag pattern changed in a workflow and not in the guide is
# red in both directions at once. Independently, every workflow path that the
# guide names in a code span anywhere, table or prose, must be a file on disk,
# so a renamed workflow is red by its old name.
#
# The walk of `on:` is written for the shapes these workflows use: a block
# list of tags under `push:` and an inline `tags: [...]` list. A trigger
# written in some other shape is not read, and the floor guard below is what
# stops a parser that reads nothing from reporting a clean run.
#
# # WHAT IS NOT HELD
#
# The steps of the guide, the asset names and the order of the steps are
# prose. The asset names are held against `release.yml` by group 7 of
# `tools/repo/readme-fixtures.sh`, and the guide cites that group rather than
# copying the names a third time. A missing guide is red, never skipped.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
guide_rel=docs/how-to/cut-a-release.md

scratch=$(mktemp -d "${TMPDIR:-/tmp}/release-guide-fixtures.XXXXXXXX") || exit 1
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

# ---------------------------------------------------------------------------
# triggers FILE — prints `<basename> <trigger>` for each tag pattern under
# `on: push: tags:`, and `<basename> workflow_dispatch` when the `on:` block
# names that event and no other.
# ---------------------------------------------------------------------------
triggers() {
    awk -v name="$(basename "$1")" '
        function indent(s) { match(s, /^ */); return RLENGTH }
        function unquote(s) {
            gsub(/^[ \t]+|[ \t]+$/, "", s)
            gsub(/^["\x27]|["\x27]$/, "", s)
            return s
        }
        /^[ \t]*#/ { next }
        /^[ \t]*$/ { next }
        {
            ind = indent($0)
            line = $0
            sub(/[ \t]+#.*$/, "", line)
        }
        # `on: workflow_dispatch` or `on: [a, b]` on one line.
        !inon && line ~ /^("on"|on):[ \t]*[^ \t]/ {
            v = line; sub(/^[^:]*:[ \t]*/, "", v); gsub(/[\[\]]/, "", v)
            n = split(v, ev, /,/)
            for (i = 1; i <= n; i++) events[unquote(ev[i])] = 1
            next
        }
        !inon && line ~ /^("on"|on):[ \t]*$/ { inon = 1; next }
        inon && ind == 0 { inon = 0; inpush = 0; intags = 0 }
        !inon { next }
        ind == 2 {
            ev1 = line; sub(/^ */, "", ev1); sub(/:.*$/, "", ev1)
            events[ev1] = 1
            inpush = (ev1 == "push"); intags = 0
            next
        }
        inpush && ind > 2 && line ~ /^ *tags:/ {
            tagind = ind
            v = line; sub(/^ *tags:[ \t]*/, "", v)
            if (v ~ /^\[/) {
                gsub(/[\[\]]/, "", v)
                n = split(v, tg, /,/)
                for (i = 1; i <= n; i++) print name " " unquote(tg[i])
                intags = 0
            } else intags = 1
            next
        }
        intags && ind <= tagind { intags = 0 }
        intags && line ~ /^ *- / {
            v = line; sub(/^ *- */, "", v)
            print name " " unquote(v)
            next
        }
        END {
            only = 1; any = 0
            for (e in events) { any = 1; if (e != "workflow_dispatch") only = 0 }
            if (any && only) print name " workflow_dispatch"
        }
    ' "$1"
}

# tree_pairs ROOT — every release trigger of every workflow under ROOT.
tree_pairs() {
    for wf in "$1"/.github/workflows/*.yml; do
        [ -f "$wf" ] && triggers "$wf"
    done | LC_ALL=C sort -u
}

# guide_pairs GUIDE — every `(workflow, trigger)` row of the guide's tables.
guide_pairs() {
    awk '
        /^[ \t]*\|/ {
            n = split($0, cell, /\|/)
            # cell[1] is empty: the text before the leading pipe.
            if (n < 3) next
            if (!match(cell[2], /`\.github\/workflows\/[^`]+\.yml`/)) next
            wf = substr(cell[2], RSTART + 1, RLENGTH - 2)
            sub(/^.*\//, "", wf)
            rest = cell[3]
            while (match(rest, /`[^`]+`/)) {
                print wf " " substr(rest, RSTART + 1, RLENGTH - 2)
                rest = substr(rest, RSTART + RLENGTH)
            }
        }
    ' "$1" | LC_ALL=C sort -u
}

# guide_paths GUIDE — every workflow path the guide names in a code span.
guide_paths() {
    grep -o '`\.github/workflows/[^`]*\.yml`' "$1" | tr -d '`' | LC_ALL=C sort -u
}

# ---------------------------------------------------------------------------
# judge ROOT — prints one line per finding, and nothing for a guide that holds.
# ---------------------------------------------------------------------------
judge() {
    r=$1
    g=$r/$guide_rel
    if [ ! -f "$g" ]; then
        echo "no guide at $guide_rel"
        return
    fi
    for p in $(guide_paths "$g"); do
        [ -f "$r/$p" ] || echo "the guide names $p and no such file exists"
    done
    tree_pairs "$r" >"$scratch/judge.tree"
    guide_pairs "$g" >"$scratch/judge.guide"
    LC_ALL=C comm -23 "$scratch/judge.tree" "$scratch/judge.guide" |
        while read -r wf tr; do
            echo "$wf fires on \`$tr\` and the guide has no row for it"
        done
    LC_ALL=C comm -13 "$scratch/judge.tree" "$scratch/judge.guide" |
        while read -r wf tr; do
            echo "the guide says $wf fires on \`$tr\` and the workflow does not"
        done
}

# copy_tree DEST — the two inputs of the judge, copied under DEST.
copy_tree() {
    mkdir -p "$1/.github/workflows" "$1/docs/how-to"
    cp "$root"/.github/workflows/*.yml "$1/.github/workflows/"
    if [ -f "$root/$guide_rel" ]; then
        cp "$root/$guide_rel" "$1/$guide_rel"
    fi
}

# ---------------------------------------------------------------------------
echo "the release workflows on disk"

tree_pairs "$root" >"$scratch/real.tree"
n_tree=$(wc -l <"$scratch/real.tree" | tr -d ' ')
if [ "$n_tree" -ge 4 ]; then
    pass "the tree side is not implausibly small ($n_tree release triggers)"
else
    fail "the tree side is not implausibly small" \
        "read $n_tree release triggers; on 2026-09-27 there were 4, so the walk of \`on:\` has stopped matching"
fi
same "  release.yml fires on v*" "release.yml v*" \
    "$(grep '^release.yml ' "$scratch/real.tree")"
same "  release-taxonomy.yml fires on its own namespace" \
    "release-taxonomy.yml taxonomy/headwater-standard/v*" \
    "$(grep '^release-taxonomy.yml ' "$scratch/real.tree")"
same "  yank-crates.yml is started by a person alone" \
    "yank-crates.yml workflow_dispatch" \
    "$(grep '^yank-crates.yml ' "$scratch/real.tree")"
same "  ci.yml, which fires on every push, is not a release workflow" "" \
    "$(grep '^ci.yml ' "$scratch/real.tree")"

echo
echo "the guide and the workflows are one set"

same "the guide in this tree holds against its workflows" "" \
    "$(judge "$root" | tr '\n' '|' | sed 's/|$//')"

echo
echo "the judge refuses what it claims to refuse"

# a. A missing guide is red, never skipped.
mkdir -p "$scratch/a/.github/workflows"
cp "$root"/.github/workflows/*.yml "$scratch/a/.github/workflows/"
same "a tree with no guide is red" "no guide at $guide_rel" "$(judge "$scratch/a")"

if [ -f "$root/$guide_rel" ]; then
    # b. A renamed workflow is red by its old name.
    copy_tree "$scratch/b"
    mv "$scratch/b/.github/workflows/publish-crates.yml" \
        "$scratch/b/.github/workflows/publish-to-crates.yml"
    out=$(judge "$scratch/b")
    case "$out" in
        *"the guide names .github/workflows/publish-crates.yml and no such file exists"*)
            pass "a renamed publish-crates.yml is red and named" ;;
        *) fail "a renamed publish-crates.yml is red and named" "got \`$out\`" ;;
    esac
    case "$out" in
        *"publish-to-crates.yml fires on \`v*\` and the guide has no row for it"*)
            pass "  and the new name is red in the other direction" ;;
        *) fail "  and the new name is red in the other direction" "got \`$out\`" ;;
    esac

    # c. A fourth release workflow the guide does not name.
    copy_tree "$scratch/c"
    printf '%s\n' 'name: Extra' 'on:' '  push:' '    tags:' "      - 'v*'" \
        'jobs: {}' >"$scratch/c/.github/workflows/extra-release.yml"
    same "a v* workflow the guide does not name is red" \
        "extra-release.yml fires on \`v*\` and the guide has no row for it" \
        "$(judge "$scratch/c")"

    # c2. The same, written with an inline list and a dispatch trigger beside it.
    copy_tree "$scratch/c2"
    printf '%s\n' 'on:' '  push:' "    tags: ['nightly/*']" '  workflow_dispatch:' \
        'jobs: {}' >"$scratch/c2/.github/workflows/nightly.yml"
    same "  and an inline tag list is read too" \
        "nightly.yml fires on \`nightly/*\` and the guide has no row for it" \
        "$(judge "$scratch/c2")"

    # d. A tag pattern changed in one workflow.
    copy_tree "$scratch/d"
    sed "s|'taxonomy/headwater-standard/v\*'|'taxonomy/standard/v*'|" \
        "$root/.github/workflows/release-taxonomy.yml" \
        >"$scratch/d/.github/workflows/release-taxonomy.yml"
    same "a tag pattern changed in a workflow is red in both directions" \
        "release-taxonomy.yml fires on \`taxonomy/standard/v*\` and the guide has no row for it|the guide says release-taxonomy.yml fires on \`taxonomy/headwater-standard/v*\` and the workflow does not" \
        "$(judge "$scratch/d" | tr '\n' '|' | sed 's/|$//')"

    # e. A dispatch-only workflow gains a push trigger.
    copy_tree "$scratch/e"
    sed 's/^on:$/on:\n  push:\n    branches: [main]/' \
        "$root/.github/workflows/yank-crates.yml" \
        >"$scratch/e/.github/workflows/yank-crates.yml"
    same "a yank workflow that a push can start no longer matches the guide" \
        "the guide says yank-crates.yml fires on \`workflow_dispatch\` and the workflow does not" \
        "$(judge "$scratch/e")"
else
    fail "the arms over a copy of the guide" "no guide at $guide_rel, so none of them can run"
fi

echo
echo "$passed passed, $failed failed"
[ "$failed" -eq 0 ]
