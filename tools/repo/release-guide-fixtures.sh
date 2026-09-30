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
# # THE REASONS, AND WHERE A WORKFLOW POINTS AT THEM
#
# The reasons the release process is the way it is live in records under
# `docs/decisions/` and `docs/process/decisions/` (#1006), and the guide and
# the header comment of each release workflow cite them rather than restate
# them. The last group below holds the header side, in both directions: each
# identifier the header of `release.yml`, `release-taxonomy.yml` or
# `publish-crates.yml` cites is exactly one record that governs that
# workflow, and each record that governs one of them is cited in its header.
# The reasons for `ci.yml` moved into records the same way (#1007), so its
# header is held by the same judge, though it is not a release workflow.
# So a reason moved back into a comment with no record, or a record that no
# header points at, is red. The `governs` edge is what makes a record suspect
# when its workflow changes, and `headwater check` reads that half.
#
# # WHAT IS NOT HELD
#
# Whether a record states the same reason as the comment it replaced is prose,
# and no suite reads it.
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

# ---------------------------------------------------------------------------
# cites ROOT — the header of each cited workflow and the records that
# govern it are one set. Prints one line per finding, and nothing for a tree
# that holds.
#
# The header is the comment block at the top of the file, before the first
# line that is not a comment. Every `HW-DR-NNNN` and `HW-PD-NNNN` in it must be
# the `id:` of exactly one record under `docs/decisions/` or
# `docs/process/decisions/`, and that record's `relations.governs` must name
# the workflow. In the other direction, every record on those two shelves
# whose `governs` names the workflow must be cited in its header. A header
# that cites nothing is a finding. A record cited in a step comment and not in
# the header is not read here: a step may cite a record about one line.
# ---------------------------------------------------------------------------
cited_workflows="release.yml release-taxonomy.yml publish-crates.yml ci.yml deploy-site.yml"

cites_py='
import fnmatch, glob, os, posixpath, re, sys
try:
    import yaml
except ImportError:
    print("PyYAML is not installed, so no record can be read")
    sys.exit(0)

root = sys.argv[1]
workflows = sys.argv[2].split()
ident = re.compile(r"\bHW-(?:DR|PD)-[0-9]{4}\b")

def as_list(v):
    if v is None:
        return []
    if isinstance(v, list):
        return v
    return [v]

# A `governs` value is one entry or a list of entries. An entry is a pattern,
# a list of patterns, or a map whose `to` is either. The engine reads each
# shape as an edge, so each shape is read here.
def governs_targets(rel):
    out = []
    governs = rel.get("governs") if isinstance(rel, dict) else None
    for entry in as_list(governs):
        if isinstance(entry, dict):
            entry = entry.get("to")
        for p in as_list(entry):
            if isinstance(p, str):
                # The engine reads `./x` and `x` as one path, so the judge does too.
                out.append(posixpath.normpath(p))
    return out

records = {}
for shelf in ("docs/decisions", "docs/process/decisions"):
    for path in sorted(glob.glob(os.path.join(root, shelf, "*.md"))):
        with open(path, encoding="utf-8") as f:
            text = f.read()
        if not text.startswith("---\n"):
            continue
        end = text.find("\n---", 4)
        try:
            front = yaml.safe_load(text[4:end]) or {}
        except Exception:
            continue
        rid = front.get("id")
        if not isinstance(rid, str):
            continue
        rel = os.path.relpath(path, root)
        records.setdefault(rid, []).append((rel, governs_targets(front.get("relations"))))

out = []
for wf in workflows:
    wf_path = ".github/workflows/" + wf
    full = os.path.join(root, wf_path)
    if not os.path.isfile(full):
        out.append("%s is not in the tree" % wf)
        continue
    header = []
    with open(full, encoding="utf-8") as f:
        for line in f:
            if not line.lstrip().startswith("#") and line.strip():
                break
            header.append(line)
    cited = sorted(set(ident.findall("".join(header))))
    if not cited:
        out.append("the header of %s cites no record" % wf)
    for rid in cited:
        found = records.get(rid, [])
        if len(found) != 1:
            out.append("the header of %s cites %s and %d records carry that id" % (wf, rid, len(found)))
            continue
        rel, targets = found[0]
        if not any(fnmatch.fnmatchcase(wf_path, t) for t in targets):
            out.append("the header of %s cites %s and %s does not govern it" % (wf, rid, rel))
    for rid in sorted(records):
        for rel, targets in records[rid]:
            if rid not in cited and any(fnmatch.fnmatchcase(wf_path, t) for t in targets):
                out.append("%s governs %s and its header does not cite %s" % (rel, wf, rid))
for line in out:
    print(line)
'

cites() {
    python3 -c "$cites_py" "$1" "$cited_workflows"
}

# copy_tree DEST — the inputs of both judges, copied under DEST.
copy_tree() {
    mkdir -p "$1/.github/workflows" "$1/docs/how-to" "$1/docs/decisions" "$1/docs/process/decisions"
    for wf in "$root"/.github/workflows/*.yml "$root"/.github/workflows/*.yaml; do
        [ -f "$wf" ] && cp "$wf" "$1/.github/workflows/"
    done
    if [ -f "$root/$guide_rel" ]; then
        cp "$root/$guide_rel" "$1/$guide_rel"
    fi
    cp "$root"/docs/decisions/*.md "$1/docs/decisions/"
    cp "$root"/docs/process/decisions/*.md "$1/docs/process/decisions/"
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
echo "each cited workflow header cites the records that govern it"

same "the headers in this tree and the records that govern them are one set" "" \
    "$(cites "$root" | tr '\n' '|' | sed 's/|$//')"
for wf in $cited_workflows; do
    n=$(sed -n '/^[^#]/q;p' "$root/.github/workflows/$wf" | grep -oE 'HW-(DR|PD)-[0-9]{4}' | LC_ALL=C sort -u | wc -l | tr -d ' ')
    if [ "$n" -ge 1 ]; then
        pass "  the header of $wf cites $n records"
    else
        fail "  the header of $wf cites a record" "it cites none, so its reasons live only in the comment"
    fi
done

echo
echo "the citation judge refuses what it claims to refuse"

# contains NAME NEEDLE HAYSTACK
contains() {
    case "$3" in
        *"$2"*) pass "$1" ;;
        *) fail "$1" "expected a line \`$2\`, got \`$3\`" ;;
    esac
}

# g1. A header that cites an identifier no record carries.
copy_tree "$scratch/g1"
sed '1a\
# HW-PD-9999 is cited here and no record carries it.' \
    "$root/.github/workflows/release.yml" >"$scratch/g1/.github/workflows/release.yml"
contains "a header that cites HW-PD-9999 is red" \
    "the header of release.yml cites HW-PD-9999 and 0 records carry that id" "$(cites "$scratch/g1")"

# g2. A header that cites a record that does not govern the workflow.
copy_tree "$scratch/g2"
sed '1a\
# HW-DR-0028 is cited here and governs only the overlay.' \
    "$root/.github/workflows/release-taxonomy.yml" >"$scratch/g2/.github/workflows/release-taxonomy.yml"
contains "a header that cites a record that governs something else is red" \
    "the header of release-taxonomy.yml cites HW-DR-0028 and docs/decisions/0028-q28-whether-an-evaluation-is-governed-prose.md does not govern it" \
    "$(cites "$scratch/g2")"

# g3. A record that governs release.yml and that the header does not cite.
copy_tree "$scratch/g3"
printf '%s\n' '---' 'id: HW-DR-9998' 'relations:' '  governs:' '    - .github/workflows/release.yml' '---' '' '# A reason nobody cites' \
    >"$scratch/g3/docs/decisions/9998-a-reason-nobody-cites.md"
contains "a record that governs release.yml and is not cited there is red" \
    "docs/decisions/9998-a-reason-nobody-cites.md governs release.yml and its header does not cite HW-DR-9998" \
    "$(cites "$scratch/g3")"

# g3b. The same record, with `governs` written as one string rather than a list.
copy_tree "$scratch/g3b"
printf '%s\n' '---' 'id: HW-DR-9998' 'relations:' '  governs: .github/workflows/release.yml' '---' '' '# A reason nobody cites' \
    >"$scratch/g3b/docs/decisions/9998-a-reason-nobody-cites.md"
contains "  and with \`governs\` written as one string" \
    "docs/decisions/9998-a-reason-nobody-cites.md governs release.yml and its header does not cite HW-DR-9998" \
    "$(cites "$scratch/g3b")"

# g3c. The same record, with `governs` written as one map.
copy_tree "$scratch/g3c"
printf '%s\n' '---' 'id: HW-DR-9998' 'relations:' '  governs:' '    to: .github/workflows/release.yml' '---' '' '# A reason nobody cites' \
    >"$scratch/g3c/docs/decisions/9998-a-reason-nobody-cites.md"
contains "  and with \`governs\` written as one map" \
    "docs/decisions/9998-a-reason-nobody-cites.md governs release.yml and its header does not cite HW-DR-9998" \
    "$(cites "$scratch/g3c")"

# g4. A workflow with every citation removed from its comments.
copy_tree "$scratch/g4"
sed -E 's/HW-(DR|PD)-[0-9]{4}//g' "$root/.github/workflows/publish-crates.yml" \
    >"$scratch/g4/.github/workflows/publish-crates.yml"
out=$(cites "$scratch/g4")
contains "a workflow with its citations removed is red" \
    "the header of publish-crates.yml cites no record" "$out"
contains "  and each record that governs it is named" \
    "governs publish-crates.yml and its header does not cite" "$out"

# g5. ci.yml with every citation removed, so a reason moved back into its
# comment and out of a record is red (#1007).
copy_tree "$scratch/g5"
sed -E 's/HW-(DR|PD)-[0-9]{4}//g' "$root/.github/workflows/ci.yml" \
    >"$scratch/g5/.github/workflows/ci.yml"
out=$(cites "$scratch/g5")
contains "ci.yml with its citations removed is red" \
    "the header of ci.yml cites no record" "$out"
contains "  and each record that governs it is named" \
    "governs ci.yml and its header does not cite" "$out"

# g6. A record that governs `./.github/workflows/ci.yml` and that the header
# does not cite. The engine reads `./` as the same path, so the judge must too.
copy_tree "$scratch/g6"
printf '%s\n' '---' 'id: HW-PD-9997' 'relations:' '  governs:' '    - to: ./.github/workflows/ci.yml' '---' '' '# A reason nobody cites' \
    >"$scratch/g6/docs/process/decisions/9997-a-reason-nobody-cites.md"
contains "a record that governs ./.github/workflows/ci.yml and is not cited there is red" \
    "docs/process/decisions/9997-a-reason-nobody-cites.md governs ci.yml and its header does not cite HW-PD-9997" \
    "$(cites "$scratch/g6")"

# ---------------------------------------------------------------------------
# deploys ROOT — the site is deployed by one job, and a release runs it after
# the release exists (#1316). Prints one line per finding, and nothing for a
# tree that holds.
#
# `tools/site/fetch-apt.sh` copies the APT repository out of
# `releases/latest`, so a deploy that runs before the release exists serves
# the previous version. The push to `main` that bumps the version deploys
# before the tag's `release.yml` has created the release, and v0.4.1 served a
# 404 under `apt/` until the next push. So `release.yml` calls the deploy
# after `publish`. It is not an `on: release` workflow, because `publish`
# creates the release with `GITHUB_TOKEN`, and GitHub starts no workflow from
# an event that token caused.
#
# What is held, each in one line:
#
#   - `deploy-site.yml` is started by `workflow_call` alone, and it is the one
#     workflow whose steps run `tools/site/deploy-site.sh` or `wrangler
#     deploy`, so two callers share one job and one concurrency group
#     (HW-DR-0097's one deploy job).
#   - `ci.yml` has a job that calls it.
#   - `release.yml` has a job that calls it, that `needs` `publish`, that
#     passes `ref: main`, and that carries no `if:`. Inside a called workflow
#     `github.ref` is the caller's tag, so without `ref: main` the site rolls
#     back to the tagged tree. With no `if:`, a skipped or failed `publish`
#     skips the deploy, and an `if: always()` would deploy on a
#     `publish: false` hand run.
#   - `deploy-site.yml` checks out `inputs.ref`, so the `ref: main` a caller
#     passes reaches the checkout, and each caller passes `secrets`, without
#     which the deploy has no Cloudflare token.
#   - No file that describes the APT route says that a build service of
#     Cloudflare deploys it, by any of the names that service goes by (#1339,
#     #1408). HW-DR-0097 moved the deploy into `deploy-site.yml`,
#     and a maintainer who reads the old route looks for a build that does
#     not exist. `apt_route` below reads the five files that describe it,
#     the README among them, because an adopter reads the route there.
# ---------------------------------------------------------------------------
deploys_py='
import glob, os, re, sys
try:
    import yaml
except ImportError:
    print("PyYAML is not installed, so no workflow can be read")
    sys.exit(0)

callee = "./.github/workflows/deploy-site.yml"
runs_deploy = re.compile(r"tools/site/deploy-site\.sh|\bwrangler\s+deploy\b")

def as_list(v):
    if v is None:
        return []
    if isinstance(v, (list, tuple)):
        return list(v)
    return [v]

d = os.path.join(sys.argv[1], ".github", "workflows")
docs = {}
for path in sorted(glob.glob(os.path.join(d, "*.yml")) + glob.glob(os.path.join(d, "*.yaml"))):
    try:
        with open(path, encoding="utf-8") as f:
            docs[os.path.basename(path)] = yaml.safe_load(f)
    except Exception as e:
        print("%s does not parse: %s" % (os.path.basename(path), str(e).splitlines()[0]))

def jobs(name):
    doc = docs.get(name)
    if not isinstance(doc, dict) or not isinstance(doc.get("jobs"), dict):
        return {}
    return {k: v for k, v in doc["jobs"].items() if isinstance(v, dict)}

def callers(name):
    return {k: v for k, v in jobs(name).items() if v.get("uses") == callee}

out = []
if "deploy-site.yml" not in docs:
    out.append("no .github/workflows/deploy-site.yml, so the site has no one deploy job")
else:
    doc = docs["deploy-site.yml"]
    on = doc.get("on", doc.get(True)) if isinstance(doc, dict) else None
    events = set(on) if isinstance(on, dict) else set(as_list(on))
    if events != {"workflow_call"}:
        out.append("deploy-site.yml is started by %s and not by workflow_call alone" % ", ".join(sorted(map(str, events))))

for name in sorted(docs):
    for job, body in jobs(name).items():
        for step in as_list(body.get("steps")):
            if isinstance(step, dict) and runs_deploy.search(str(step.get("run", ""))):
                if name != "deploy-site.yml":
                    out.append("%s job %s runs the deploy itself, so the site has two deploy paths" % (name, job))
if "deploy-site.yml" in docs and not any(
    isinstance(s, dict) and runs_deploy.search(str(s.get("run", "")))
    for b in jobs("deploy-site.yml").values() for s in as_list(b.get("steps"))
):
    out.append("deploy-site.yml runs no step of tools/site/deploy-site.sh")
if "deploy-site.yml" in docs:
    checkouts = [s for b in jobs("deploy-site.yml").values() for s in as_list(b.get("steps"))
                 if isinstance(s, dict) and str(s.get("uses", "")).startswith("actions/checkout@")]
    if not checkouts or any(not isinstance(s.get("with"), dict)
                            or str(s["with"].get("ref", "")).replace(" ", "") != "${{inputs.ref}}"
                            for s in checkouts):
        out.append("deploy-site.yml checks out a tree other than inputs.ref, so a caller cannot name main")

for name in ("ci.yml", "release.yml"):
    for job, body in sorted(callers(name).items()):
        if "secrets" not in body:
            out.append("%s job %s passes no secrets, so the deploy has no Cloudflare token" % (name, job))

if not callers("ci.yml"):
    out.append("ci.yml has no job that calls deploy-site.yml, so a push to main deploys nothing")

rel = callers("release.yml")
if not rel:
    out.append("release.yml has no job that calls deploy-site.yml, so apt/ serves the previous release until the next push to main")
for job, body in sorted(rel.items()):
    if "publish" not in as_list(body.get("needs")):
        out.append("release.yml job %s does not need publish, so it can deploy before the release exists" % job)
    w = body.get("with")
    if not isinstance(w, dict) or w.get("ref") != "main":
        out.append("release.yml job %s does not pass ref: main, so it deploys the tagged tree" % job)
    if "if" in body:
        out.append("release.yml job %s has an if:, so it can deploy when publish did not run" % job)

# `fetch-apt.sh` reads `releases/latest`, so that release must be an engine
# release. A taxonomy release carries only its zip, and when one became
# "latest" every deploy served no apt/ (#1449). So each `gh release create`
# in release-taxonomy.yml passes `--latest=false`.
creates = [(job, s) for job, b in sorted(jobs("release-taxonomy.yml").items()) for s in as_list(b.get("steps"))
           if isinstance(s, dict) and re.search(r"\bgh\s+release\s+create\b", str(s.get("run", "")))]
if "release-taxonomy.yml" in docs and not creates:
    out.append("release-taxonomy.yml creates no release, so nothing here can hold that it passes --latest=false")
for job, s in creates:
    for line in str(s.get("run", "")).splitlines():
        if re.search(r"\bgh\s+release\s+create\b", line) and not re.search(r"--latest=false\b", line):
            out.append("release-taxonomy.yml job %s creates a release without --latest=false, so a taxonomy release can become releases/latest and the site serves no apt/" % job)

for line in out:
    print(line)
'

deploys() {
    python3 -c "$deploys_py" "$1"
}

# apt_route ROOT — prints one line for each file that describes the APT route
# and says that a build service of Cloudflare deploys it, and nothing for a
# tree that holds. The service has more than one name, so the match reads each
# of them: "Cloudflare build", "Cloudflare Workers Build(s)", "Workers
# Build(s)", "Cloudflare Pages build" and "the build service of Cloudflare"
# (#1408). The match ignores case, a `#` and a line break inside the phrase,
# because the shell and YAML comments wrap it.
#
# It reads a present claim and not the vendor's noun. Two lines in these files
# say what deployed the site before HW-DR-0097, and they are true. So a
# sentence that names the service passes only when it is history: it carries a
# past verb ("ran", "deployed", "was", ...) and no present one ("is", "gets",
# "still", "now", "deploys", "serves", ...). Every other sentence that names
# the service is a claim, and a sentence with both is a claim too, because
# "is deployed by", "has deployed" and "deploys apt/ before" are present. The
# bare phrase "Cloudflare build" is a claim in any tense, as it was before
# #1408, so the widened reader loses no line the narrow one caught. A sentence
# ends at `.`, `!`, `?` or `;` and a space, but not after an initial such as
# "J." or after "e.g." and "i.e.". The present list is
# read after the service's own name is removed, so "Workers Builds" is not
# the verb "builds", and after a link target is removed, so a slug is not a
# sentence. d11 holds that the history line passes, and the d10 plants hold
# that each present shape is red.
#
# The reader exits 0 on a claim, 1 on a file that holds, and anything else when
# it could not read the file, and `apt_route` reports that last case as a
# finding. A reader that fails is not a file that holds.
apt_route_py='
import re, sys
service = re.compile(r"cloudflare\s+(?:workers\s+|pages\s+)?builds?\b|\bworkers\s+builds?\b|build\s+service\s+of\s+cloudflare", re.I)
past = re.compile(r"\b(?:ran|deployed|served|built|published|hosted|was|were|had|formerly|previously|used\s+to)\b", re.I)
present = re.compile(r"\b(?:is|are|am|be|has|have|gets?|getting|still|now|currently|deploys|serves|builds|runs|publishes|hosts|does|will|reaches|handles|uploads|pushes|copies|carries)\b|(?<!\bthe )(?<!\ba )\b(?:deploy|serve|run|publish|host)\b", re.I)
always = re.compile(r"cloudflare\s+builds?\b", re.I)
try:
    text = open(sys.argv[1], encoding="utf-8").read()
except Exception as err:
    print(f"{sys.argv[1]}: {err}", file=sys.stderr)
    sys.exit(3)
text = re.sub(r"\]\([^)]*\)", "]", text).replace("#", " ")
text = re.sub(r"\s+", " ", text)
if always.search(text):
    sys.exit(0)
for sentence in re.split(r"(?<=[.!?;])(?<!\b[A-Z]\.)(?<!\be\.g\.)(?<!\bi\.e\.)\s", text):
    if not service.search(sentence):
        continue
    rest = service.sub(" ", sentence)
    if present.search(rest) or not past.search(rest):
        sys.exit(0)
sys.exit(1)
'

apt_route_files="docs/decisions/0094-the-apt-repository-is-served-from-headwater-tools-and-signed-by-a-subkey-the-owner-s-offline-key-certifies.md
tools/site/fetch-apt.sh
.github/workflows/ci.yml
docs/how-to/rotate-or-revoke-the-apt-signing-subkey.md
README.md"

apt_route() {
    printf '%s\n' "$apt_route_files" | while IFS= read -r f; do
        if [ ! -f "$1/$f" ]; then
            echo "$f is missing, so nothing states the APT route there"
        else
            python3 -c "$apt_route_py" "$1/$f"
            case $? in
                0) echo "$f says a build service of Cloudflare deploys the APT repository" ;;
                1) ;;
                *) echo "$f could not be read, so nothing says whether it names a build service of Cloudflare" ;;
            esac
        fi
    done
}

# edit_wf DIR FILE PYTHON — loads DIR/.github/workflows/FILE, runs PYTHON on
# it as `doc`, and writes it back. A copy loses its comments, which no judge
# here reads.
edit_wf() {
    python3 -c '
import sys, yaml
path = sys.argv[1] + "/.github/workflows/" + sys.argv[2]
with open(path, encoding="utf-8") as f:
    doc = yaml.safe_load(f)
exec(sys.argv[3])
with open(path, "w", encoding="utf-8") as f:
    yaml.safe_dump(doc, f, sort_keys=False)
' "$1" "$2" "$3"
}

echo
echo "a release deploys the site after it publishes"

same "the tree deploys the site from one job, after the release exists" "" \
    "$(deploys "$root" | tr '\n' '|' | sed 's/|$//')"

# The name of the job in release.yml that calls the deploy, for the arms.
rel_job=$(python3 -c '
import sys, yaml
jobs = yaml.safe_load(open(sys.argv[1] + "/.github/workflows/release.yml", encoding="utf-8"))["jobs"]
print(" ".join(k for k, v in jobs.items() if isinstance(v, dict) and v.get("uses") == "./.github/workflows/deploy-site.yml"))
' "$root" 2>/dev/null)

if [ -n "$rel_job" ]; then
    # d1. The deploy job is deleted from release.yml: the case #1316 exists for.
    copy_tree "$scratch/d1"
    edit_wf "$scratch/d1" release.yml "del doc['jobs']['$rel_job']"
    same "a release with no deploy job is red" \
        "release.yml has no job that calls deploy-site.yml, so apt/ serves the previous release until the next push to main" \
        "$(deploys "$scratch/d1" | tr '\n' '|' | sed 's/|$//')"

    # d2. The deploy no longer waits for publish, so it races the release.
    copy_tree "$scratch/d2"
    edit_wf "$scratch/d2" release.yml "doc['jobs']['$rel_job']['needs'] = [n for n in (doc['jobs']['$rel_job']['needs'] if isinstance(doc['jobs']['$rel_job']['needs'], list) else [doc['jobs']['$rel_job']['needs']]) if n != 'publish'] or ['tag']"
    same "a deploy that does not need publish is red" \
        "release.yml job $rel_job does not need publish, so it can deploy before the release exists" \
        "$(deploys "$scratch/d2" | tr '\n' '|' | sed 's/|$//')"

    # d3. The deploy checks out the caller's ref, which is the tag.
    copy_tree "$scratch/d3"
    edit_wf "$scratch/d3" release.yml "doc['jobs']['$rel_job']['with'].pop('ref')"
    same "a deploy with no ref: main is red" \
        "release.yml job $rel_job does not pass ref: main, so it deploys the tagged tree" \
        "$(deploys "$scratch/d3" | tr '\n' '|' | sed 's/|$//')"

    # d4. An if: always() deploys on a publish: false hand run.
    copy_tree "$scratch/d4"
    edit_wf "$scratch/d4" release.yml "doc['jobs']['$rel_job']['if'] = 'always()'"
    same "a deploy with if: always() is red" \
        "release.yml job $rel_job has an if:, so it can deploy when publish did not run" \
        "$(deploys "$scratch/d4" | tr '\n' '|' | sed 's/|$//')"

    # d7. The release caller passes no secrets, so the deploy fails on the token.
    copy_tree "$scratch/d7"
    edit_wf "$scratch/d7" release.yml "doc['jobs']['$rel_job'].pop('secrets')"
    same "a release deploy that passes no secrets is red" \
        "release.yml job $rel_job passes no secrets, so the deploy has no Cloudflare token" \
        "$(deploys "$scratch/d7" | tr '\n' '|' | sed 's/|$//')"
else
    fail "release.yml has a job that calls deploy-site.yml" \
        "none, so the arms d1 to d4 have no job to edit"
fi

# d12. A taxonomy release created without --latest=false can become GitHub's
# "latest" release, and fetch-apt.sh then finds no InRelease (#1449).
copy_tree "$scratch/d12"
edit_wf "$scratch/d12" release-taxonomy.yml "[s.__setitem__('run', s['run'].replace(' --latest=false', '')) for j in doc['jobs'].values() for s in j.get('steps', []) if 'gh release create' in str(s.get('run', ''))]"
contains "a taxonomy release created without --latest=false is red" \
    "release-taxonomy.yml job artifact creates a release without --latest=false" "$(deploys "$scratch/d12")"

# d8. The called workflow checks out its caller's ref, whatever the input, so
# a release deploys its tag.
copy_tree "$scratch/d8"
if [ -f "$scratch/d8/.github/workflows/deploy-site.yml" ]; then
    edit_wf "$scratch/d8" deploy-site.yml "[s.pop('with', None) for j in doc['jobs'].values() for s in j.get('steps', []) if str(s.get('uses', '')).startswith('actions/checkout@')]"
fi
contains "a deploy-site.yml that ignores inputs.ref is red" \
    "deploy-site.yml checks out a tree other than inputs.ref" "$(deploys "$scratch/d8")"

# d5. A copy of the deploy steps inlined into release.yml: two deploy paths.
copy_tree "$scratch/d5"
edit_wf "$scratch/d5" release.yml "doc['jobs']['deploy-copy'] = {'needs': 'publish', 'runs-on': 'ubuntu-latest', 'steps': [{'uses': 'actions/checkout@v4'}, {'run': 'sh tools/site/deploy-site.sh'}]}"
contains "a copy of the deploy steps in release.yml is red" \
    "release.yml job deploy-copy runs the deploy itself, so the site has two deploy paths" \
    "$(deploys "$scratch/d5")"

# d9. The APT route as each file that describes it states it (#1339).
same "no file says a build service of Cloudflare deploys the APT repository" "" \
    "$(apt_route "$root" | tr '\n' '|' | sed 's/|$//')"

# d10. The phrase planted in a copy of fetch-apt.sh, wrapped as a comment
# wraps it, so the case can fail.
mkdir -p "$scratch/d10"
printf '%s\n' "$apt_route_files" | while IFS= read -r f; do
    mkdir -p "$scratch/d10/$(dirname "$f")"
    cp "$root/$f" "$scratch/d10/$f"
done
printf '%s\n' '#   The site reaches Cloudflare by one path only, the Cloudflare' \
    '#   Build, so this step reads those assets at build time.' >> "$scratch/d10/tools/site/fetch-apt.sh"
same "fetch-apt.sh that names the Cloudflare build is red" \
    "tools/site/fetch-apt.sh says a build service of Cloudflare deploys the APT repository" \
    "$(apt_route "$scratch/d10" | tr '\n' '|' | sed 's/|$//')"

# d10, the other names of the service (#1408). Each is planted in a fresh copy
# of fetch-apt.sh, wrapped as a comment wraps it, and each is red. The first
# half of each plant goes on one line and the name ends it or starts the next,
# so the break falls inside the phrase where it can.
apt_route_plant() {
    rm -rf "$scratch/$1"
    mkdir -p "$scratch/$1"
    printf '%s\n' "$apt_route_files" | while IFS= read -r f; do
        mkdir -p "$scratch/$1/$(dirname "$f")"
        cp "$root/$f" "$scratch/$1/$f"
    done
    printf '%s\n' "$2" "$3" >> "$scratch/$1/tools/site/fetch-apt.sh"
    if cmp -s "$root/tools/site/fetch-apt.sh" "$scratch/$1/tools/site/fetch-apt.sh"; then
        echo "the plant changed nothing"
    else
        apt_route "$scratch/$1" | tr '\n' '|' | sed 's/|$//'
    fi
}
apt_route_red="tools/site/fetch-apt.sh says a build service of Cloudflare deploys the APT repository"
same "fetch-apt.sh that says Cloudflare Workers Builds deploys the repository is red" "$apt_route_red" \
    "$(apt_route_plant d10w '#   The repository under apt/ is deployed by Cloudflare Workers' '#   Builds, on each push to main.')"
same "fetch-apt.sh that says Workers Build serves the site is red" "$apt_route_red" \
    "$(apt_route_plant d10b '#   Workers' '#   Build serves the site and the APT repository from this tree.')"
same "fetch-apt.sh that says the Cloudflare Pages build deploys the site is red" "$apt_route_red" \
    "$(apt_route_plant d10p '#   The Cloudflare Pages' '#   build deploys the site, and apt/ with it.')"
same "fetch-apt.sh that says the build service of Cloudflare deploys the repository is red" "$apt_route_red" \
    "$(apt_route_plant d10s '#   The APT repository is deployed by the build service of' '#   Cloudflare from this script.')"

# d11. A line that says what deployed the site BEFORE HW-DR-0097 is history,
# and it is true, so it passes. Two of the five files carry one.
same "fetch-apt.sh that says Cloudflare Workers Builds deployed the site before is green" "" \
    "$(apt_route_plant d11 '#   Cloudflare Workers Builds deployed the site before; it could not' '#   wait for CI.')"

# d12. Present claims that carry a past word or a passive, each red. The last
# one names the phrase the check read before #1408, so the widened reader
# keeps every line the narrow one caught.
same "a claim that the repository is now deployed by Cloudflare Workers Builds is red" "$apt_route_red" \
    "$(apt_route_plant d12a '#   The APT repository is now deployed by Cloudflare Workers' '#   Builds.')"
same "a claim that the site gets deployed by Cloudflare Workers Builds is red" "$apt_route_red" \
    "$(apt_route_plant d12b '#   The site gets deployed by Cloudflare Workers Builds' '#   from this tree.')"
same "a claim that apt/ is built by the Cloudflare build is red" "$apt_route_red" \
    "$(apt_route_plant d12c '#   apt/ is automatically built by the Cloudflare' '#   build.')"
same "a claim that Cloudflare Workers Builds still deploys the site, as it was set up to, is red" "$apt_route_red" \
    "$(apt_route_plant d12d '#   Cloudflare Workers Builds still deploys the site, as it was' '#   set up to do.')"
same "a claim that the Cloudflare build deploys apt/ before the notes go out is red" "$apt_route_red" \
    "$(apt_route_plant d12e '#   The Cloudflare build deploys apt/ before the release notes' '#   go out.')"
same "a claim that Cloudflare Workers Builds deploy the repository is red" "$apt_route_red" \
    "$(apt_route_plant d12f '#   Cloudflare Workers Builds deploy the APT repository from' '#   this tree.')"

same "a claim that the Cloudflare build has deployed apt/ since #1316 is red" "$apt_route_red" \
    "$(apt_route_plant d12g '#   The Cloudflare build has deployed apt/ since' '#   #1316.')"
same "a claim that the Cloudflare build ships apt/, as it was set up to, is red" "$apt_route_red" \
    "$(apt_route_plant d12h '#   The Cloudflare build ships apt/, as it was set up' '#   to do.')"
same "a claim that Workers Builds has deployed apt/ since #1316 is red" "$apt_route_red" \
    "$(apt_route_plant d12i '#   Cloudflare Workers Builds has deployed apt/ since' '#   #1316.')"
same "a claim split by an initial, Workers Builds, which was added by J. Baxter, deploys apt/, is red" "$apt_route_red" \
    "$(apt_route_plant d12j '#   Workers Builds, which was added in #1316 by J.' '#   Baxter, deploys apt/.')"

# d13. A file the reader cannot decode is a finding, and not a file that holds.
mkdir -p "$scratch/d13"
printf '%s\n' "$apt_route_files" | while IFS= read -r f; do
    mkdir -p "$scratch/d13/$(dirname "$f")"
    cp "$root/$f" "$scratch/d13/$f"
done
printf '#   \377\376 not UTF-8\n' >> "$scratch/d13/tools/site/fetch-apt.sh"
same "a route file that is not UTF-8 is reported as unread, not as holding" \
    "tools/site/fetch-apt.sh could not be read, so nothing says whether it names a build service of Cloudflare" \
    "$(apt_route "$scratch/d13" 2>/dev/null | tr '\n' '|' | sed 's/|$//')"

# d6. The called workflow gains a second trigger, so a third deploy path.
copy_tree "$scratch/d6"
if [ -f "$scratch/d6/.github/workflows/deploy-site.yml" ]; then
    edit_wf "$scratch/d6" deploy-site.yml "k = 'on' if 'on' in doc else True; doc[k] = dict(doc[k] or {}); doc[k]['push'] = {'branches': ['main']}"
fi
contains "a deploy-site.yml with a trigger of its own is red" \
    "deploy-site.yml is started by" "$(deploys "$scratch/d6")"

echo
echo "$passed passed, $failed failed"
[ "$failed" -eq 0 ]
