#!/bin/sh
# What holds `docs/how-to/cut-a-release.md`, the guide to cutting a release.
#
# Run it from anywhere:
#     sh tools/repo/release-guide-fixtures.sh
#
# It needs `python3` and `PyYAML` (`python3 -c 'import yaml'`), which the
# `headwater` job of `.github/workflows/ci.yml` installs before this step, as
# it does for `tools/repo/integrations-fixtures.sh`. Without them this suite is
# red, never skipped.
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
# The tree side is every `(workflow, trigger)` pair a release can start, over
# every `.yml` and `.yaml` file in `.github/workflows/`, because Actions runs
# both extensions:
#
#   - each tag pattern under `on: push: tags:`,
#   - `release` for a workflow that a `release` event starts, and
#   - `workflow_dispatch` for each workflow that a person alone can start,
#     which is a workflow whose `on:` names no other event. That is how
#     `yank-crates.yml` enters the population without being named here.
#
# The guide side is every row of a Markdown table in the guide whose first
# cell is a code span naming `.github/workflows/<name>`, paired with each
# code span in its second cell.
#
# The two sets must be equal. A fourth release workflow added without a guide
# row is red, and a tag pattern changed in a workflow and not in the guide is
# red in both directions at once. Independently, every workflow the guide
# names in a code span anywhere, as a full path or as a bare file name ending
# `.yml` or `.yaml`, must be a file in `.github/workflows/`, so a renamed
# workflow is red by its old name.
#
# # WHY A YAML PARSER AND NOT A LINE WALK
#
# The first cut of this suite walked `on:` line by line, and a verifier left it
# green with a new tag workflow in each of five shapes it did not read: a
# `.yaml` extension, four-space indentation, a flow mapping, a single-quoted
# `'on'` key, and a `release` event. Each is a workflow GitHub runs. A parser
# reads all of them the same way, so the tree side is `yaml.safe_load`. PyYAML
# reads the bare key `on` as the boolean true (YAML 1.1), and both keys are
# accepted. A workflow file that does not parse, or that has no `on`, is a
# finding rather than an empty set, so a new workflow can never drop out of
# the population in silence.
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
# tree_pairs ROOT — prints `<basename> <trigger>` for every release trigger of
# every workflow under ROOT, and `unreadable <basename>: <why>` for a workflow
# this cannot read.
# ---------------------------------------------------------------------------
tree_pairs_py='
import glob, os, sys
try:
    import yaml
except ImportError:
    print("unreadable (all): PyYAML is not installed, so no workflow can be read")
    sys.exit(0)

def as_list(v):
    if v is None:
        return []
    if isinstance(v, (list, tuple)):
        return list(v)
    return [v]

out = []
d = os.path.join(sys.argv[1], ".github", "workflows")
files = sorted(glob.glob(os.path.join(d, "*.yml")) + glob.glob(os.path.join(d, "*.yaml")))
for path in files:
    name = os.path.basename(path)
    try:
        with open(path, encoding="utf-8") as f:
            doc = yaml.safe_load(f)
    except Exception as e:
        out.append("unreadable %s: %s" % (name, str(e).splitlines()[0]))
        continue
    if not isinstance(doc, dict):
        out.append("unreadable %s: the file is not a mapping" % name)
        continue
    on = doc.get("on", doc.get(True))
    if on is None:
        out.append("unreadable %s: the file declares no on:" % name)
        continue
    if isinstance(on, dict):
        events = on
    else:
        events = {e: None for e in as_list(on)}
    push = events.get("push")
    if isinstance(push, dict):
        for tag in as_list(push.get("tags")):
            out.append("%s %s" % (name, tag))
    if "release" in events:
        out.append("%s release" % name)
    if set(events) == {"workflow_dispatch"}:
        out.append("%s workflow_dispatch" % name)
for line in sorted(set(out)):
    print(line)
'

tree_pairs() {
    python3 -c "$tree_pairs_py" "$1" | LC_ALL=C sort -u
}

# guide_pairs GUIDE — every `(workflow, trigger)` row of the guide's tables.
guide_pairs() {
    awk '
        /^[ \t]*\|/ {
            n = split($0, cell, /\|/)
            # cell[1] is empty: the text before the leading pipe.
            if (n < 3) next
            if (!match(cell[2], /`\.github\/workflows\/[^`]+\.ya?ml`/)) next
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

# guide_names GUIDE — the file name of every workflow the guide names in a
# code span: a full `.github/workflows/` path, or a bare file name with no
# directory that ends `.yml` or `.yaml`.
guide_names() {
    {
        grep -o '`\.github/workflows/[^`/]*\.ya\{0,1\}ml`' "$1" | sed 's|.*/||'
        grep -o '`[A-Za-z0-9_.-]*\.ya\{0,1\}ml`' "$1"
    } | tr -d '`' | LC_ALL=C sort -u
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
    for n in $(guide_names "$g"); do
        [ -f "$r/.github/workflows/$n" ] ||
            echo "the guide names $n and .github/workflows/ has no such file"
    done
    tree_pairs "$r" >"$scratch/judge.all"
    grep '^unreadable ' "$scratch/judge.all"
    grep -v '^unreadable ' "$scratch/judge.all" >"$scratch/judge.tree"
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
    for wf in "$root"/.github/workflows/*.yml "$root"/.github/workflows/*.yaml; do
        [ -f "$wf" ] && cp "$wf" "$1/.github/workflows/"
    done
    if [ -f "$root/$guide_rel" ]; then
        cp "$root/$guide_rel" "$1/$guide_rel"
    fi
}

# arm NAME DIR FILE — moves `$scratch/arm.in` to DIR/.github/workflows/FILE
# in a fresh copy of the tree, and requires the judge to report exactly one
# finding: that FILE fires on `v*` and the guide has no row for it. The input
# is a file and not standard input, because a function at the end of a pipe
# runs in a subshell and its pass and fail counts would be lost.
arm() {
    copy_tree "$2"
    mv "$scratch/arm.in" "$2/.github/workflows/$3"
    same "$1" "$3 fires on \`v*\` and the guide has no row for it" \
        "$(judge "$2" | tr '\n' '|' | sed 's/|$//')"
}

# ---------------------------------------------------------------------------
echo "the release workflows on disk"

if python3 -c 'import yaml' 2>/dev/null; then
    pass "python3 reads YAML"
else
    fail "python3 reads YAML" "install PyYAML: every case below reads the workflows with it"
fi

tree_pairs "$root" >"$scratch/real.all"
grep -v '^unreadable ' "$scratch/real.all" >"$scratch/real.tree"
same "every workflow on disk reads" "" \
    "$(grep '^unreadable ' "$scratch/real.all" | tr '\n' '|' | sed 's/|$//')"
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
        *"the guide names publish-crates.yml and .github/workflows/ has no such file"*)
            pass "a renamed publish-crates.yml is red and named" ;;
        *) fail "a renamed publish-crates.yml is red and named" "got \`$out\`" ;;
    esac
    case "$out" in
        *"publish-to-crates.yml fires on \`v*\` and the guide has no row for it"*)
            pass "  and the new name is red in the other direction" ;;
        *) fail "  and the new name is red in the other direction" "got \`$out\`" ;;
    esac

    # c. A new v* workflow, in each shape GitHub runs.
    printf '%s\n' 'name: Extra' 'on:' '  push:' '    tags:' "      - 'v*'" 'jobs: {}' >"$scratch/arm.in"
    arm "a v* workflow the guide does not name is red" "$scratch/c1" extra-release.yml
    printf '%s\n' 'on:' '  push:' "    tags: ['v*']" '  workflow_dispatch:' 'jobs: {}' >"$scratch/arm.in"
    arm "  and written with an inline tag list" "$scratch/c2" inline.yml
    printf '%s\n' 'on:' '  push:' '    tags:' "      - 'v*'" 'jobs: {}' >"$scratch/arm.in"
    arm "  and with a .yaml extension" "$scratch/c3" extra-release.yaml
    printf '%s\n' 'on:' '    push:' '        tags:' "            - 'v*'" 'jobs: {}' >"$scratch/arm.in"
    arm "  and with four-space indentation" "$scratch/c4" four.yml
    printf '%s\n' "on: {push: {tags: ['v*']}}" 'jobs: {}' >"$scratch/arm.in"
    arm "  and as a flow mapping" "$scratch/c5" flow.yml
    printf '%s\n' "'on':" '  push:' '    tags:' "      - 'v*'" 'jobs: {}' >"$scratch/arm.in"
    arm "  and under a single-quoted on key" "$scratch/c6" quoted.yml

    # c7. A workflow a release event starts.
    copy_tree "$scratch/c7"
    printf '%s\n' 'on:' '  release:' '    types: [published]' 'jobs: {}' \
        >"$scratch/c7/.github/workflows/on-release.yml"
    same "a workflow a release event starts is red" \
        "on-release.yml fires on \`release\` and the guide has no row for it" \
        "$(judge "$scratch/c7")"

    # c8. A workflow that does not parse is a finding, not an empty set.
    copy_tree "$scratch/c8"
    printf '%s\n' 'on:' '  push:' '    tags: [v*' 'jobs: {}' \
        >"$scratch/c8/.github/workflows/broken.yml"
    out=$(judge "$scratch/c8")
    case "$out" in
        "unreadable broken.yml: "*) pass "a workflow that does not parse is red and named" ;;
        *) fail "a workflow that does not parse is red and named" "got \`$out\`" ;;
    esac

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

    # f. A bare workflow name in the prose that names no file.
    copy_tree "$scratch/f"
    printf '\n%s\n' 'Run `publish-crate.yml` again.' >>"$scratch/f/$guide_rel"
    same "a bare workflow name in the prose that names no file is red" \
        "the guide names publish-crate.yml and .github/workflows/ has no such file" \
        "$(judge "$scratch/f")"
else
    fail "the arms over a copy of the guide" "no guide at $guide_rel, so none of them can run"
fi

echo
echo "$passed passed, $failed failed"
[ "$failed" -eq 0 ]
