#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
#
# The fixtures of tools/probe/try.sh, the eleven verbs of the additive
# campaign's tuning round (#1821, #1822, #1838).
#
#     sh tools/probe/try-fixtures.sh
#
# No case spends, calls a model or builds a real tree. Each case runs the
# script against a declaration the case writes, a stub driver that appends
# each call to a log and exits, and a stub `campaign.sh` whose
# `--scenario-tree` copies a tree the case wrote for the pin or for `HEAD`.
# Every refused `launch` asserts that the driver's log is empty, and every
# refusal of spend asserts that no tree was built.
#
# The decisive case is the first: a round root that holds two earlier try
# roots, one of them dot-named and holding a `spent-before`, refuses a try
# that would take the round one cent past its approval, and passes the same
# layout at the approval exactly.
#
# The decisive case of `result` (#1822) is the table of `result: the keep
# rule`: a stub engine prints the pooled line that `headwater probe compare`
# prints for each try against a best of 39 of 72, and 40 of 72, which the
# issue's first rule kept, is not kept under the owner's rule of 2026-10-20.
#
# The decisive case of the rounds (#1838) is `rank and confirm: usage before
# pass rate`: a variant that passes 22 of 25 and never calls the skill ranks
# below one that passes 14 of 25 and calls it in 12, and is not confirmed.
#
# The decisive case of the fixed draw (#1875) is `the fixed draw`: two
# variants of one round of 22 tuning probes at `screen: 22` each plan 22
# sessions, and plan and launch the same draw of phrasings, byte for byte.
#
# Exit status: 0 when every case passes, 1 when one fails, 3 when a tool it
# needs is missing.

set -u

for tool in git awk python3 diff sha256sum setsid timeout; do
    command -v "$tool" >/dev/null 2>&1 || { echo "try-fixtures: $tool is missing, so no case runs." >&2; exit 3; }
done
python3 -c 'import yaml' 2>/dev/null || { echo "try-fixtures: python3 has no PyYAML, so no case runs." >&2; exit 3; }

here=$(cd "$(dirname "$0")" && pwd)
try=${HEADWATER_TRY_UNDER_TEST:-$here/try.sh}
fix=$(mktemp -d "${TMPDIR:-/tmp}/try-fixtures.XXXXXX")

# The python3 of CI's self-hosted runner. It installs PyYAML with
# `pip install --user`, and `python3 -I` drops the user site, so under `-I`
# the runner has no PyYAML. This host may have PyYAML in the system site,
# where `-I` still reaches it, so every case runs a python3 that hides PyYAML
# from an isolated run. A script that reads the declaration must therefore run
# without `-I` (#1838: 26 cases passed here and failed on the runner).
real_python3=$(command -v python3)
mkdir "$fix/bin"
cat > "$fix/bin/python3" <<EOF
#!/bin/sh
case "\$1" in
    -I) shift
        exec "$real_python3" -I -c 'import runpy, sys; sys.modules["yaml"] = None; sys.argv = sys.argv[1:]; runpy.run_path(sys.argv[0], run_name="__main__")' "\$@" ;;
esac
exec "$real_python3" "\$@"
EOF
chmod +x "$fix/bin/python3"
PATH=$fix/bin:$PATH
export PATH

# The checkouts the driver runs from. `repo` is in the one state the driver's
# pin check accepts: detached, at a commit `origin/main` holds, and clean.
# Each other one is in a state that check refuses.
make_repo() {
    git init -q "$1"
    git -C "$1" -c user.name=fixture -c user.email=fixture@example.org commit -q --allow-empty -m base
    git -C "$1" update-ref refs/remotes/origin/main HEAD
    git -C "$1" checkout -q --detach
}
make_repo "$fix/repo"
make_repo "$fix/repo-branch"
git -C "$fix/repo-branch" checkout -q -b work
make_repo "$fix/repo-dirty"
: > "$fix/repo-dirty/untracked"
make_repo "$fix/repo-ahead"
git -C "$fix/repo-ahead" -c user.name=fixture -c user.email=fixture@example.org commit -q --allow-empty -m ahead
make_repo "$fix/repo-moves"
# A variant screened before it merges: ahead of origin/main, on a pushed
# branch under origin/tune/.
make_repo "$fix/repo-tune"
git -C "$fix/repo-tune" -c user.name=fixture -c user.email=fixture@example.org commit -q --allow-empty -m variant
git -C "$fix/repo-tune" checkout -q --detach
git -C "$fix/repo-tune" update-ref refs/remotes/origin/tune/fixture-v1 HEAD
checkout=$fix/repo
passed=0
failed=0
cases=0

ok() { passed=$((passed + 1)); }
bad() { failed=$((failed + 1)); echo "FAIL: $1" >&2; }
check() {
    # check <description> <command...>
    desc=$1
    shift
    if "$@"; then ok; else bad "$desc"; fi
}

# ---------------------------------------------------------------- the stubs

cat > "$fix/driver-stub.sh" <<'EOF'
#!/bin/sh
{
    printf 'argv'
    for arg in "$@"; do printf ' %s' "$arg"; done
    printf '\n'
} >> "$TRY_CALLS"
echo "env versions=${HEADWATER_DRIVE_CLAUDE_VERSIONS:-} target=${CARGO_TARGET_DIR:-} tmp=${TMPDIR:-} pwd=$PWD stage=${HEADWATER_TUNING_STAGE:-}" >> "$TRY_CALLS"
echo "usage stop=${HEADWATER_DRIVE_USAGE_STOP:-} late=${HEADWATER_DRIVE_USAGE_LATE:-} hours=${HEADWATER_DRIVE_USAGE_LATE_HOURS:-}" >> "$TRY_CALLS"
echo "draw ${HEADWATER_CAMPAIGN_DRAW:-}" >> "$TRY_CALLS"
if [ -f "$TRY_SLEEP" ]; then sleep 30; fi
exit 0
EOF
cat > "$fix/campaign-stub.sh" <<'EOF'
#!/bin/sh
n= out= at= assemble=
while [ $# -gt 0 ]; do
    case $1 in
        --assemble) assemble=1; shift; continue ;;
        --scenario-tree) n=$2 ;;
        --out) out=$2 ;;
        --at) at=$2 ;;
        *) echo "stub: unknown $1" >&2; exit 2 ;;
    esac
    shift 2
done
if [ -n "$assemble" ]; then
    # The assembly of a tuning try: one line, as campaign.sh names its files.
    echo "assemble $out" >> "$CASE/trees.log"
    [ -f "$CASE/assemble-fails" ] && { echo "stub: another identity" >&2; exit 2; }
    mkdir -p "$out/assembled"
    stem=L1-additive-scenario-1-tuning
    printf 'model: claude-sonnet-5\ntier: additive\narm: scenario-1\nat: 2026-10-21\ncost_cents: 2452\n' > "$out/assembled/$stem.identity.yaml"
    printf -- '- probe: "HW-PROBE-a"\n  session: "L1-additive-scenario-1-p1-r1"\n' > "$out/assembled/$stem.events.yaml"
    echo '72 sessions, 2452 cents, the intent hook live in 0' > "$out/assembled/$stem.summary"
    exit 0
fi
echo "tree $n $at" >> "$CASE/trees.log"
[ -f "$CASE/tree-fails" ] && { echo "stub: the tree did not build" >&2; exit 3; }
if [ "$at" = HEAD ]; then src=$CASE/tree-head; else src=$CASE/tree-pin; fi
mkdir -p "$out/trees"
cp -a "$src" "$out/trees/additive-scenario-$n"
EOF
# The engine of `result`: `new` writes a scaffold with the facets it was
# given, and `probe compare` prints a per-probe line and the pooled line of
# the case: the base row against the scenario-0 screening transcript, and the
# keep row against anything else.
cat > "$fix/engine-stub.sh" <<'EOF'
#!/bin/sh
printf '%s\n' "$*" >> "$CASE/engine.log"
case $1 in
    new)
        shift 2
        r= t= facets=
        while [ $# -gt 0 ]; do
            case $1 in
                --root) r=$2 ;;
                --title) t=$2 ;;
                --facet) facets="$facets
${2%%=*}: ${2#*=}" ;;
            esac
            shift 2
        done
        slug=$(printf '%s' "$t" | tr 'A-Z' 'a-z' | sed 's/[^a-z0-9][^a-z0-9]*/-/g; s/^-//; s/-$//')
        mkdir -p "$r/docs/probe-runs"
        printf -- '---\nid: HW-RUN-%s\nstatus: draft%s\ntitle: "%s"\n---\n\n# %s\n\n## Run identity\n\nTODO write this section.\n\n## Events\n\nTODO write this section.\n' \
            "$slug" "$facets" "$t" "$t" > "$r/docs/probe-runs/$slug.md"
        echo "wrote docs/probe-runs/$slug.md"
        ;;
    probe)
        # The plan of a hook variant's selection (#1875): the probes of the
        # case's moment, or of its action.
        if [ "$2" = plan ]; then
            case "$*" in
                *--moment*) cat "$CASE/plan-moment" 2>/dev/null ;;
                *--action*) cat "$CASE/plan-action" 2>/dev/null ;;
                # The plain plan of the round's set, which `result` reads
                # for each probe's action and oracle (#1875).
                *) cat "$CASE/plan-plain" 2>/dev/null ;;
            esac
            exit 0
        fi
        treated= control=
        for each in "$@"; do treated=$control control=$each; done
        if [ -f "$CASE/probe-rows" ]; then
            cat "$CASE/probe-rows"
        else
            echo "HW-PROBE-a: 9/18 against 9/18, +0.0 points [-30.0, +30.0]"
        fi
        # A transcript against itself is the grader's count of each probe,
        # which `result` reads (#1875), and its pooled line is not read.
        if [ "$treated" = "$control" ]; then
            echo "pooled: 9/18 against 9/18, +0.0 points [-30.0, +30.0]"
            exit 0
        fi
        case $control in
            */additive-screening-scenario-0.md) cat "$CASE/pooled-base" ;;
            *) cat "$CASE/pooled-keep" ;;
        esac
        ;;
esac
EOF
chmod +x "$fix/engine-stub.sh"
# The counter of use for the docs layer, which the tuning round does not tune
# and tools/probe/usage.py refuses: no session used it. Any other layer runs
# the real counter, isolated when the driver isolated the stub, so that a
# `python3 -I` in the driver fails here as it fails on the runner.
cat > "$fix/usage-stub.py" <<EOF
import os, sys
if sys.argv[2] != "docs":
    os.execvp("python3", ["python3"] + (["-I"] if sys.flags.isolated else []) + ["$here/usage.py"] + sys.argv[1:])
print("used 0 sessions %d" % len(os.listdir(os.path.join(sys.argv[3], "sessions"))))
EOF

OPEN_DOCS='{layer: docs, scenario: 1, try: 2, cap_cents: 3000, plan: "https://example.org/accepted", change: none}'
OPEN_HOOK='{layer: hooks, scenario: 2, try: 1, cap_cents: 3000, plan: "https://example.org/accepted", change: none, moment: intent}'

# decl <open> <approved_cents> [<ceiling_cents>] [<base_ceiling_cents>]: the
# declaration of the case. A session costs one cent, so a round of 4 variants
# at 25 sessions and a confirmation at 145 is priced at 245 cents under the
# tuning ceiling, and a first best so far at 145 at 145 cents under the base
# ceiling. The base ceiling is 1000 and the tuning ceiling the rest of the
# approval where the case names neither.
decl() {
    cat > "$CASE/probe.yml" <<EOF
layers:
  docs: {}
  hooks: {}
sets:
  tuning: [HW-PROBE-a, HW-PROBE-b, HW-PROBE-c, HW-PROBE-d, HW-PROBE-e]
scenarios:
  0: {layers: []}
  1: {base: 0, layers: [docs]}
  2: {base: 0, layers: [hooks]}
tiers:
  additive: {session_cost_cents: 1}
rounds:
  tuning:
    set: tuning
    rates: [0.625, 0.375]
    tries: 4
    approved_cents: $2
    screen: 24
    confirm: 144
    variants: 4
    stop_after_empty: 2
    order: [docs, hooks]
    ceiling_cents: ${3:-$(($2 - ${4:-1000}))}
    base_ceiling_cents: ${4:-1000}
    open: $1
base_invalidators:
  - engine/crates/*/src/**
  - tools/probe/campaign.sh
  - tools/probe/ablate.sh
EOF
}

# The campaign.sh of the base tree: a refusal inside a marked region, the
# line `additive=1` outside it, the check of the tools, and a tree step.
campaign_copy() {
    cat <<'EOF'
#!/bin/sh
# The batch driver.
additive=0
if [ -n "$spec" ]; then
    additive=1
    # base-neutral: begin the repetitions refusal
    if [ -n "$repetitions" ]; then
        echo "campaign: an additive spec takes no --repetitions." >&2
        exit 2
    fi
    # base-neutral: end
fi
for tool in git jq tar awk cargo claude bwrap; do
    command -v "$tool" >/dev/null 2>&1 || exit 3
done
git -C "$root" archive "$head" | tar -x -C "$out/trees/base" || exit 3
EOF
}

tree_files() {
    # tree_files <dir>: the base tree of a case.
    mkdir -p "$1/engine/crates/x/src" "$1/engine/crates/x/tests" "$1/tools/probe"
    echo 'pub fn x() {}' > "$1/engine/crates/x/src/lib.rs"
    echo '#[test] fn t() {}' > "$1/engine/crates/x/tests/t.rs"
    echo 'seal' > "$1/tools/probe/seal.sh"
    echo 'leak' > "$1/tools/probe/leak.py"
    echo 'readme' > "$1/README.md"
    campaign_copy > "$1/tools/probe/campaign.sh"
}

new_case() {
    cases=$((cases + 1))
    CASE=$fix/case.$cases
    mkdir -p "$CASE/root" "$CASE/tmp" "$CASE/versions"
    export CASE
    decl "$OPEN_DOCS" 10000
    # The floors are 0, so the free space of the host running the case never
    # decides one; the floor case sets its own.
    printf 'HEADWATER_DRIVE_CLAUDE_VERSIONS=%s\nCARGO_TARGET_DIR=%s\nTMPDIR=%s\nTRY_FLOOR_ROOT_KB=0\nTRY_FLOOR_TRY_KB=0\n' \
        "$CASE/versions" "$CASE/target" "$CASE/tmp" > "$CASE/root/env"
    echo '0 2d212a269a111fddb7af5b2b75ebaf2f9ab32f29' > "$CASE/root/base-pin"
    tree_files "$CASE/tree-pin"
    tree_files "$CASE/tree-head"
}

run() {
    HEADWATER_TRY_DECLARATION=$CASE/probe.yml \
    HEADWATER_TRY_DRIVER=$fix/driver-stub.sh \
    HEADWATER_TRY_CAMPAIGN=$fix/campaign-stub.sh \
    HEADWATER_TRY_REPO=$checkout \
    HEADWATER_TRY_ENGINE=$fix/engine-stub.sh \
    HEADWATER_TRY_USAGE=$fix/usage-stub.py \
    TRY_CALLS=$CASE/calls TRY_SLEEP=$CASE/driver-sleeps TMPDIR=$CASE/tmp \
        timeout 60 sh "$try" "$@" > "$CASE/out" 2> "$CASE/err"
    echo $? > "$CASE/code"
}

code_is() { [ "$(cat "$CASE/code")" = "$1" ]; }
out_has() { grep -qF -- "$1" "$CASE/out"; }
err_has() { grep -qF -- "$1" "$CASE/err"; }
no_calls() { [ ! -s "$CASE/calls" ]; }
no_trees() { [ ! -s "$CASE/trees.log" ]; }
calls_count() { if [ -f "$CASE/calls" ]; then grep -c '^argv ' "$CASE/calls"; else echo 0; fi; }
# await_calls <n>: the detached stub has logged n calls, within 10 s.
await_calls() {
    i=0
    while [ "$(grep -c '^env ' "$CASE/calls" 2>/dev/null || echo 0)" -lt "$1" ] && [ "$i" -lt 100 ]; do
        sleep 0.1
        i=$((i + 1))
    done
}

cost() {
    # cost <dir> <cents> [<spent-before>]
    mkdir -p "$1"
    echo "$2" > "$1/cost"
    [ -z "${3:-}" ] || echo "$3" > "$1/spent-before"
}

# The layout of the decisive case: earlier try roots of each shape a glob
# must name (`docs-try1`, `.docs-try1-moved`, `..docs-try0`), and under the
# dot-named one a batch of each dot-named shape (`.a-moved`, `..b`), as
# `spent_all` of #1775 reads them. Each cost decides the refusal by one cent.
two_roots() {
    cost "$CASE/root/docs-try1/a/sessions/s1" 3000
    cost "$CASE/root/..docs-try0/a/sessions/s1" 1000
    cost "$CASE/root/.docs-try1-moved/.a-moved/sessions/s1" 1000 "$1"
    cost "$CASE/root/.docs-try1-moved/..b/sessions/s1" 1000
    # Nothing at another depth counts: a session under a try root with no
    # batch, and a file one level below a session.
    cost "$CASE/root/tree/sessions/x" 50000
    cost "$CASE/root/docs-try1/a/sessions/s1/deeper" 50000
}

# ------------------------- the decisive case: cross-root spend, by one cent

new_case
two_roots 1001
find "$CASE/root" | LC_ALL=C sort > "$CASE/before"
run spent "$CASE/root"
check "spent: the sum of the three try roots, the dot-named ones, the two dot-named batches and the spent-before included, is 7001" out_has 7001
check "spent: exits 0" code_is 0
run launch "$CASE/root" screen 1
check "decisive: launch at approval + 1 exits 5" code_is 5
check "decisive: it prints what was spent" out_has "spent: 7001"
check "decisive: it prints the cap" out_has "cap: 3000"
check "decisive: it prints the approval" out_has "approved: 10000"
check "decisive: the driver is not invoked" no_calls
check "decisive: no tree is built" no_trees
check "decisive: no try root is made" [ ! -e "$CASE/root/docs-r2-v1" ]
run plan "$CASE/root"
check "decisive: plan at approval + 1 exits 5" code_is 5
find "$CASE/root" | LC_ALL=C sort > "$CASE/after"
check "spent and plan store nothing under the root" cmp -s "$CASE/before" "$CASE/after"
new_case
two_roots 1000
run plan "$CASE/root"
check "decisive: plan at the approval exactly exits 0" code_is 0
check "decisive: and builds the pin's tree and HEAD's" sh -c "grep -qx 'tree 0 2d212a269a111fddb7af5b2b75ebaf2f9ab32f29' '$CASE/trees.log' && grep -qx 'tree 0 HEAD' '$CASE/trees.log'"

# ------------------------------------------------------------------ spent

new_case
printf '30' > "$CASE/root/x.txt"
mkdir -p "$CASE/root/t/a/sessions/s1" "$CASE/root/t/a/sessions/s2"
printf '30' > "$CASE/root/t/a/sessions/s1/cost"
printf '40' > "$CASE/root/t/a/sessions/s2/cost"
run spent "$CASE/root"
check "spent: two costs with no final newline add to 70, not 3040" out_has 70
check "spent: and not 3040" sh -c "! grep -q 3040 '$CASE/out'"
check "spent: the try's own sum is 70 in either order of the two files, not 3040 or 4030" out_has "tries: 70 of the tuning ceiling"
run status "$CASE/root"
check "status: the try's line sums the two files to 70" out_has "t spent=70 "
cost "$CASE/root/t/a/sessions/s3" 'twelve'
run spent "$CASE/root"
check "spent: a cost that is not whole cents exits 2" code_is 2
check "spent: and names the file" err_has "s3/cost"
new_case
run spent "$CASE/root"
check "spent: a round with no session prints 0" out_has 0
run spent "$CASE/nowhere"
check "spent: a root that does not exist exits 2" code_is 2

# ------------------------------------------------------------------ status

new_case
two_roots 1001
: > "$CASE/root/docs-try1/canary-passed"
: > "$CASE/root/docs-try1/done"
run status "$CASE/root"
check "status: exits 0" code_is 0
check "status: one line for the first try, with what it spent and its markers" out_has "docs-try1 spent=3000 driver=none markers=canary-passed done"
check "status: one line for the dot-named try" out_has ".docs-try1-moved spent=3001"
: > "$CASE/root/docs-try1/usage-gated"
run status "$CASE/root"
check "status: a stop of the usage gate is among the markers (#1876)" out_has "docs-try1 spent=3000 driver=none markers=canary-passed usage-gated done"
check "status: the total against the approval" out_has "spent: 7001 of 10000 approved"
check "status: the open layer" out_has "open: docs 1 2 3000"

# -------------------------------------------------------------- plan: spend

new_case
decl none 10000
run plan "$CASE/root"
check "plan: no open layer exits 5" code_is 5
check "plan: no open layer builds no tree" no_trees
check "plan: and says the round is open for no layer" err_has "open for no layer"
new_case
two_roots 1001
echo 'x' >> "$CASE/tree-head/engine/crates/x/src/lib.rs"
run plan "$CASE/root"
check "plan: a refusal of spend is 5 even where the base is stale, never 4" code_is 5

# --------------------------------------------------------------- plan: base

new_case
run plan "$CASE/root"
check "base: identical trees exit 0" code_is 0
check "base: and say that 0 paths differ" out_has "0 paths differ"
check "base: and the round may start" out_has "plan: round 2 of docs on scenario 1 may start"

new_case
sed 's/takes no --repetitions\./takes no --repetitions at all./' "$CASE/tree-pin/tools/probe/campaign.sh" > "$CASE/tree-head/tools/probe/campaign.sh"
sed -i '/# base-neutral: end/i\        echo "campaign: one more line of the refusal." >&2' "$CASE/tree-head/tools/probe/campaign.sh"
sed -i 's/^# The batch driver\./# The batch driver of a campaign./' "$CASE/tree-head/tools/probe/campaign.sh"
run plan "$CASE/root"
check "base: a campaign.sh hunk inside a marked refusal region, and a comment, exit 0" code_is 0
check "base: and say campaign.sh is neutral" out_has "neutral: tools/probe/campaign.sh"

new_case
sed 's/^git -C "\$root" archive "\$head"/git -C "$root" archive --worktree-attributes "$head"/' "$CASE/tree-pin/tools/probe/campaign.sh" > "$CASE/tree-head/tools/probe/campaign.sh"
run plan "$CASE/root"
check "base: a one-line change to the tree step exits 4" code_is 4
check "base: and names campaign.sh and the line" out_has "invalidates: tools/probe/campaign.sh (tools/probe/campaign.sh): removed line 16: git -C"

new_case
sed 's/^    additive=1$/    additive=2/' "$CASE/tree-pin/tools/probe/campaign.sh" > "$CASE/tree-head/tools/probe/campaign.sh"
run plan "$CASE/root"
check "base: a change to additive=1, outside every region, exits 4" code_is 4
check "base: and names campaign.sh" out_has "invalidates: tools/probe/campaign.sh"

new_case
# A line only added, outside every region, removes nothing: the added side
# alone has to hold it.
sed 's/^    additive=1$/&\n    retries=3/' "$CASE/tree-pin/tools/probe/campaign.sh" > "$CASE/tree-head/tools/probe/campaign.sh"
run plan "$CASE/root"
check "base: a code line only added, outside every region, exits 4" code_is 4
check "base: and names the added line" out_has "added line 6:     retries=3"

new_case
# The base pin's copy has no markers, so a change to its refusal is removed
# code that no region of the older copy holds.
grep -v 'base-neutral' "$CASE/tree-pin/tools/probe/campaign.sh" > "$CASE/tree-pin/tools/probe/campaign.sh.new"
mv "$CASE/tree-pin/tools/probe/campaign.sh.new" "$CASE/tree-pin/tools/probe/campaign.sh"
sed 's/takes no --repetitions\./takes no --repetitions at all./' "$CASE/tree-pin/tools/probe/campaign.sh" > "$CASE/tree-head/tools/probe/campaign.sh"
sed -i 's/^    if \[ -n "\$repetitions" \]; then$/    # base-neutral: begin the refusal\n&/; s/^    fi$/&\n    # base-neutral: end/' "$CASE/tree-head/tools/probe/campaign.sh"
run plan "$CASE/root"
check "base: a changed line whose older copy no region holds exits 4" code_is 4
check "base: and names the removed line" out_has "removed line"

new_case
# Lines added inside a region the newer copy marks, over a pin with no
# markers, are neutral: the #1819 gate over 2d212a26.
grep -v 'base-neutral' "$CASE/tree-pin/tools/probe/campaign.sh" > "$CASE/tree-pin/tools/probe/campaign.sh.new"
mv "$CASE/tree-pin/tools/probe/campaign.sh.new" "$CASE/tree-pin/tools/probe/campaign.sh"
sed '/^fi$/i\    # base-neutral: begin the open-layer refusal\n    if [ "$open" = none ]; then\n        exit 5\n    fi\n    # base-neutral: end' "$CASE/tree-pin/tools/probe/campaign.sh" > "$CASE/tree-head/tools/probe/campaign.sh"
run plan "$CASE/root"
check "base: a refusal added inside a marked region over an unmarked pin exits 0" code_is 0

# The committed campaign.sh: each refusal it marks reads as neutral when it
# is added over a copy without it, and its markers close before the check of
# the tools. A change to `additive=1`, outside every region, is not neutral.
new_case
awk '/^ *# base-neutral: begin/ { skip = 1 } !skip { print } /^ *# base-neutral: end/ { skip = 0 }' "$here/campaign.sh" > "$CASE/tree-pin/tools/probe/campaign.sh"
cp "$here/campaign.sh" "$CASE/tree-head/tools/probe/campaign.sh"
run plan "$CASE/root"
check "base: the committed campaign.sh's refusals, added over a copy without them, exit 0" code_is 0
check "base: and campaign.sh reads as neutral" out_has "neutral: tools/probe/campaign.sh"
check "base: the committed campaign.sh marks three regions" [ "$(grep -c '^ *# base-neutral: begin' "$here/campaign.sh")" = 3 ]
new_case
cp "$here/campaign.sh" "$CASE/tree-pin/tools/probe/campaign.sh"
sed 's/^    additive=1$/    additive=1 # set here/' "$here/campaign.sh" > "$CASE/tree-head/tools/probe/campaign.sh"
run plan "$CASE/root"
check "base: a change to the committed campaign.sh's additive=1 exits 4" code_is 4

new_case
sed -i '/^for tool in git jq/d; /^    command -v "\$tool"/d; /^done$/d' "$CASE/tree-head/tools/probe/campaign.sh"
echo 'for tool in git jq tar awk cargo claude bwrap; do :; done' > "$CASE/tail"
awk -v t="$CASE/tail" '/# base-neutral: end/ { while ((getline l < t) > 0) print l } { print }' "$CASE/tree-head/tools/probe/campaign.sh" > "$CASE/c"
mv "$CASE/c" "$CASE/tree-head/tools/probe/campaign.sh"
run plan "$CASE/root"
check "base: a region that reaches the check of the tools exits 2" code_is 2
check "base: and says so" err_has "reaches the check of the tools"

new_case
echo 'pub fn y() {}' >> "$CASE/tree-head/engine/crates/x/src/lib.rs"
run plan "$CASE/root"
check "base: a change to engine/crates/x/src/lib.rs exits 4" code_is 4
check "base: and names the path and the pattern" out_has "invalidates: engine/crates/x/src/lib.rs (engine/crates/*/src/**)"

new_case
echo '#[test] fn u() {}' >> "$CASE/tree-head/engine/crates/x/tests/t.rs"
echo 'more' >> "$CASE/tree-head/README.md"
run plan "$CASE/root"
check "base: a change to engine/crates/x/tests/t.rs and README.md alone exits 0" code_is 0
check "base: and lists both as differing" sh -c "grep -qx 'differs: engine/crates/x/tests/t.rs' '$CASE/out' && grep -qx 'differs: README.md' '$CASE/out'"

new_case
echo 'sealed again' >> "$CASE/tree-head/tools/probe/seal.sh"
echo 'leak again' >> "$CASE/tree-head/tools/probe/leak.py"
run plan "$CASE/root"
check "base: a change to seal.sh and leak.py alone exits 0" code_is 0

new_case
mkdir -p "$CASE/tree-head/tools/probe"
echo 'new' > "$CASE/tree-head/tools/probe/ablate.sh"
run plan "$CASE/root"
check "base: a path on the list that only HEAD holds exits 4" code_is 4

new_case
rm "$CASE/root/base-pin"
run plan "$CASE/root"
check "base: no base-pin exits 2" code_is 2
check "base: and builds no tree" no_trees
new_case
echo '3 2d212a26' > "$CASE/root/base-pin"
run plan "$CASE/root"
check "base: a base-pin of another scenario than the open one's base exits 2" code_is 2
new_case
: > "$CASE/tree-fails"
run plan "$CASE/root"
check "base: a tree that does not build exits 3" code_is 3

# ------------------------------------------------------------------ launch

new_case
two_roots 1000
run launch "$CASE/root" screen 1
await_calls 1
check "launch: exits 0" code_is 0
check "launch: the driver is invoked once" [ "$(calls_count)" = 1 ]
check "launch: the try root is <layer>-r<n>-v<k>, with the open layer's cap" grep -q -- "--root $CASE/root/docs-r2-v1 .* --cap-cents 3000 " "$CASE/calls"
check "launch: with the constants of additive-launch.env" grep -q -- "--claude-version 2.1.288 --spec-a $CASE/root/docs-r2-v1/spec --model claude-sonnet-5 --session-cents 300 --cap-cents 3000 --parallel 4 --overshoot-cents 100 --seed 0 " "$CASE/calls"
check "launch: with --on-canary running canary on the try" grep -q -- "--on-canary sh '.*/try.sh' canary '$CASE/root' 'docs-r2-v1'\$" "$CASE/calls"
check "launch: the host facts of <root>/env reach the driver" grep -qx "env versions=$CASE/versions target=$CASE/target tmp=$CASE/tmp pwd=.*" "$CASE/calls"
check "launch: and the driver of a screen reads the screen's count" grep -q ' stage=screen$' "$CASE/calls"
check "launch: the stage is written in the try root" sh -c "[ \"\$(cat '$CASE/root/docs-r2-v1/stage')\" = 'screen 1' ]"
check "launch: with the digest of the variant's tree" [ -s "$CASE/root/docs-r2-v1/tree" ]
check "launch: the spec is one line, outside the checkout" sh -c "[ \"\$(cat '$CASE/root/docs-r2-v1/spec')\" = 'additive scenario-1 tuning' ]"
check "launch: driver.argv, driver.pid and launched are written" sh -c "[ -s '$CASE/root/docs-r2-v1/driver.argv' ] && [ -s '$CASE/root/docs-r2-v1/driver.pid' ] && [ -s '$CASE/root/docs-r2-v1/launched' ]"
run launch "$CASE/root" screen 1
check "launch: a try root that exists exits 6" code_is 6
check "launch: and invokes nothing more" [ "$(calls_count)" = 1 ]

new_case
rm "$CASE/root/env"
run launch "$CASE/root" screen 1
check "launch: no <root>/env exits 2" code_is 2
check "launch: no env invokes nothing" no_calls
new_case
echo 'PATH=/tmp/evil' >> "$CASE/root/env"
run launch "$CASE/root" screen 1
check "launch: an env line of another key exits 2" code_is 2
check "launch: an env of another key invokes nothing" no_calls
new_case
decl none 10000
run launch "$CASE/root" screen 1
check "launch: no open layer exits 5" code_is 5
check "launch: no open layer invokes nothing" no_calls
new_case
echo 'pub fn y() {}' >> "$CASE/tree-head/engine/crates/x/src/lib.rs"
run launch "$CASE/root" screen 1
check "launch: a stale base exits 4" code_is 4
check "launch: a stale base invokes nothing" no_calls
check "launch: a stale base makes no try root" [ ! -e "$CASE/root/docs-r2-v1" ]

# The driver's pin check refuses each of these checkouts, so a launch that
# started it would report a driver that exits at once.
for state in branch dirty ahead; do
    new_case
    checkout=$fix/repo-$state
    run launch "$CASE/root" screen 1
    checkout=$fix/repo
    check "launch: a checkout that is $state exits 6" code_is 6
    check "launch: a checkout that is $state invokes nothing" no_calls
    check "launch: a checkout that is $state makes no try root" [ ! -e "$CASE/root/docs-r2-v1" ]
done
check "launch: and the last one names origin/main" err_has "is not on origin/main"
new_case
checkout=$fix/repo-tune
run launch "$CASE/root" screen 1
checkout=$fix/repo
await_calls 1
check "launch: a pin on a branch under origin/tune/ exits 0" code_is 0
check "launch: a pin on a branch under origin/tune/ starts the driver" [ "$(calls_count)" = 1 ]
new_case
run launch "$CASE/root" screen 1
await_calls 1
check "launch: the driver runs from the checkout, pinned at its HEAD" sh -c "grep -qx 'env .* pwd=$fix/repo stage=screen' '$CASE/calls' && grep -q -- '--pin $(git -C "$fix/repo" rev-parse HEAD) ' '$CASE/calls'"

# ---------------------------------------------------------------- relaunch

new_case
run launch "$CASE/root" screen 1
await_calls 1
run relaunch "$CASE/root" docs-r2-v1
await_calls 2
check "relaunch: exits 0" code_is 0
check "relaunch: invokes the driver again with the same arguments" sh -c "[ \"\$(grep -c '^argv ' '$CASE/calls')\" = 2 ] && [ \"\$(grep '^argv ' '$CASE/calls' | sort -u | wc -l)\" = 1 ]"
mkdir -p "$CASE/root/docs-r2-v1"
for marker in done capped canary-failed stop; do
    : > "$CASE/root/docs-r2-v1/$marker"
    run relaunch "$CASE/root" docs-r2-v1
    check "relaunch: $marker present exits 6" code_is 6
    rm "$CASE/root/docs-r2-v1/$marker"
done
check "relaunch: no refused relaunch invokes the driver" [ "$(calls_count)" = 2 ]
# A stop of the usage gate is not a refusal: the driver reads the gate again
# (#1876), and each start hands it the values of additive-launch.env.
check "launch: the driver is handed the usage gate of additive-launch.env" sh -c "[ \"\$(grep -c '^usage stop=85 late=100 hours=24\$' '$CASE/calls')\" = 2 ]"
echo "2026-10-29T00:00:00Z usage-gated: 7-day at 86% (from x), threshold 85%, resets 2026-10-30T00:00:00Z" > "$CASE/root/docs-r2-v1/usage-gated"
run relaunch "$CASE/root" docs-r2-v1
await_calls 3
check "relaunch: usage-gated present exits 0" code_is 0
check "relaunch: and invokes the driver" [ "$(calls_count)" = 3 ]
check "relaunch: and hands it the usage gate again" sh -c "[ \"\$(grep -c '^usage stop=85 late=100 hours=24\$' '$CASE/calls')\" = 3 ]"
rm "$CASE/root/docs-r2-v1/usage-gated"
# The values come from additive-launch.env as it reads now, not from a figure
# in try.sh and not from the launch: a copy of try.sh beside an env file that
# says 70, 95 and 12 hands the driver those on a relaunch.
mkdir -p "$fix/env-copy/tools/probe"
cp "$try" "$fix/env-copy/tools/probe/try.sh"
sed -e 's/^TRY_USAGE_STOP_PERCENT=.*/TRY_USAGE_STOP_PERCENT=70/' -e 's/^TRY_USAGE_LATE_PERCENT=.*/TRY_USAGE_LATE_PERCENT=95/' \
    -e 's/^TRY_USAGE_LATE_HOURS=.*/TRY_USAGE_LATE_HOURS=12/' "$here/additive-launch.env" > "$fix/env-copy/tools/probe/additive-launch.env"
try_was=$try
try=$fix/env-copy/tools/probe/try.sh
run relaunch "$CASE/root" docs-r2-v1
try=$try_was
await_calls 4
check "relaunch: hands the driver the usage gate of the env file as it reads now" grep -q '^usage stop=70 late=95 hours=12$' "$CASE/calls"
check "relaunch: no relaunch writes a.done" [ ! -e "$CASE/root/docs-r2-v1/a.done" ]
run relaunch "$CASE/root" docs-try9
check "relaunch: a try that does not exist exits 6" code_is 6
new_case
checkout=$fix/repo-moves
run launch "$CASE/root" screen 1
await_calls 1
git -C "$fix/repo-moves" -c user.name=fixture -c user.email=fixture@example.org commit -q --allow-empty -m moved
run relaunch "$CASE/root" docs-r2-v1
checkout=$fix/repo
check "relaunch: a checkout whose HEAD left the pin of driver.argv exits 6" code_is 6
check "relaunch: and names the pin" err_has "not the pin"
check "relaunch: and invokes the driver no second time" [ "$(calls_count)" = 1 ]
new_case
: > "$CASE/driver-sleeps"
run launch "$CASE/root" screen 1
await_calls 1
run relaunch "$CASE/root" docs-r2-v1
check "relaunch: a driver that runs exits 6" code_is 6
run wait "$CASE/root" docs-r2-v1
check "wait: a driver that runs and needs nobody exits 1" code_is 1
kill "$(cat "$CASE/root/docs-r2-v1/driver.pid")" 2>/dev/null
check "relaunch: a running driver is not invoked again" [ "$(calls_count)" = 1 ]

# ------------------------------------------------------------------ canary

# record <job> [<sed expression>]: a session of the canary under docs-r2-v1,
# with the sentences of a clean record.
record() {
    s=$CASE/root/docs-r2-v1/a/sessions/$1
    mkdir -p "$s"
    echo "$1 1 additive scenario-1 tuning HW-PROBE-a" >> "$CASE/root/docs-r2-v1/a/jobs"
    echo 0 > "$s/status"
    echo 250 > "$s/cost"
    cat > "$s/record.md" <<'EOF'
The intent hook was not live in this session. It wrote no shadow-mode log line under `probe_session: x`, so no pointer reached this session before it read a file.

The proxy refused a request to the provider API that asked the provider to fetch from another host for the session. It refused no request of that kind, and no request whose head, body, method or path it does not forward.

The session named 0 paths outside its workspace that the confinement does not bind, so no file of the host at any of them was readable.

The session made 0 calls to a web tool.
EOF
}
canary_case() {
    new_case
    decl "${1:-$OPEN_DOCS}" 10000
    mkdir -p "$CASE/root/docs-r2-v1/a" "$CASE/root/hooks-try1/a"
    echo "additive scenario-1 tuning" > "$CASE/root/docs-r2-v1/spec"
    : > "$CASE/root/docs-r2-v1/canary-recorded"
    for j in 1 2 3 4 5 6 7 8; do record "L1-p1-r$j"; done
    # A job of the jobs list that has not started is not a record.
    echo "L1-p2-r1 1 additive scenario-1 tuning HW-PROBE-b" >> "$CASE/root/docs-r2-v1/a/jobs"
}
rec() { echo "$CASE/root/docs-r2-v1/a/sessions/$1"; }

canary_case
run canary "$CASE/root" docs-r2-v1
check "canary: eight clean records write canary-passed" [ -f "$CASE/root/docs-r2-v1/canary-passed" ]
check "canary: and exit 0" code_is 0
check "canary: and write no canary-needs-reader" [ ! -e "$CASE/root/docs-r2-v1/canary-needs-reader" ]

canary_case
sed -i 's/named 0 paths outside/named 1 path outside/' "$(rec L1-p1-r3)/record.md"
run canary "$CASE/root" docs-r2-v1
check "canary: one record naming 1 path outside exits 7" code_is 7
check "canary: and writes no canary-passed" [ ! -e "$CASE/root/docs-r2-v1/canary-passed" ]
check "canary: canary-needs-reader names that session and its sentence" grep -qx "L1-p1-r3: outside: The session named 1 path outside its workspace" "$CASE/root/docs-r2-v1/canary-needs-reader"
check "canary: and no other session" [ "$(wc -l < "$CASE/root/docs-r2-v1/canary-needs-reader")" = 1 ]

canary_case
sed -i 's/made 0 calls to a web tool/made 1 call to a web tool/' "$(rec L1-p1-r5)/record.md"
run canary "$CASE/root" docs-r2-v1
check "canary: one record with 1 call to a web tool exits 7" code_is 7
check "canary: and names it" grep -qx "L1-p1-r5: web: The session made 1 call to a web tool" "$CASE/root/docs-r2-v1/canary-needs-reader"

canary_case
sed -i 's/made 0 calls to a web tool/made 2 calls to a web tool/; s/named 0 paths outside/named 3 paths outside/' "$(rec L1-p1-r6)/record.md"
run canary "$CASE/root" docs-r2-v1
check "canary: the plural of each is offending too" sh -c "grep -q 'L1-p1-r6: web: The session made 2 calls' '$CASE/root/docs-r2-v1/canary-needs-reader' && grep -q 'L1-p1-r6: outside: The session named 3 paths' '$CASE/root/docs-r2-v1/canary-needs-reader'"

canary_case
sed -i 's/^The session made 0 calls to a web tool\./The calls this session made to a web tool were not counted, because its stream did not parse as JSON./; s/^The session named 0 paths outside.*/The paths outside its workspace this session named were not counted, because its stream did not parse as JSON./' "$(rec L1-p1-r2)/record.md"
run canary "$CASE/root" docs-r2-v1
check "canary: a not-counted sentence is offending and never a pass" code_is 7
check "canary: and names both sentences" sh -c "grep -q 'L1-p1-r2: web: The calls this session made to a web tool were not counted' '$CASE/root/docs-r2-v1/canary-needs-reader' && grep -q 'L1-p1-r2: outside: The paths outside its workspace this session named were not counted' '$CASE/root/docs-r2-v1/canary-needs-reader'"

canary_case
sed -i 's/It refused no request of that kind/It refused 1 request of that kind/' "$(rec L1-p1-r1)/record.md"
run canary "$CASE/root" docs-r2-v1
check "canary: a refused provider fetch exits 7" code_is 7
check "canary: and names it" grep -q "L1-p1-r1: provider: It refused 1 request of that kind" "$CASE/root/docs-r2-v1/canary-needs-reader"

canary_case
sed -i 's/The intent hook was not live in this session\./The intent hook was live in this session./' "$(rec L1-p1-r4)/record.md"
run canary "$CASE/root" docs-r2-v1
check "canary: a live hook in a scenario without the hook layer exits 7" code_is 7
check "canary: and names the hook state" grep -q "L1-p1-r4: hook: the scenario declares it not live" "$CASE/root/docs-r2-v1/canary-needs-reader"

canary_case
echo "additive scenario-2 tuning" > "$CASE/root/docs-r2-v1/spec"
run canary "$CASE/root" docs-r2-v1
check "canary: a hook that was not live in a scenario that holds the hook layer exits 7" code_is 7
check "canary: on every record" [ "$(grep -c ': hook: the scenario declares it live' "$CASE/root/docs-r2-v1/canary-needs-reader")" = 8 ]
canary_case
echo "additive scenario-2 tuning" > "$CASE/root/docs-r2-v1/spec"
for j in 1 2 3 4 5 6 7 8; do
    sed -i 's/The intent hook was not live in this session\./The intent hook was live in this session./' "$(rec "L1-p1-r$j")/record.md"
done
run canary "$CASE/root" docs-r2-v1
check "canary: a live hook in every record of a hook scenario passes" code_is 0

canary_case
echo 400 > "$(rec L1-p1-r7)/cost"
run canary "$CASE/root" docs-r2-v1
check "canary: a cost of exactly 300 + 100 passes" code_is 0
canary_case
echo 401 > "$(rec L1-p1-r7)/cost"
run canary "$CASE/root" docs-r2-v1
check "canary: a cost of 401 exits 7" code_is 7
check "canary: and names it" grep -qx "L1-p1-r7: cost 401 above 400" "$CASE/root/docs-r2-v1/canary-needs-reader"

canary_case
echo 5 > "$(rec L1-p1-r8)/status"
run canary "$CASE/root" docs-r2-v1
check "canary: a status of 5 exits 7" code_is 7
check "canary: and names it" grep -qx "L1-p1-r8: status 5" "$CASE/root/docs-r2-v1/canary-needs-reader"

canary_case
rm -r "$(rec L1-p1-r8)"
run canary "$CASE/root" docs-r2-v1
check "canary: seven records exit 7" code_is 7
check "canary: and say so" grep -qx "7 records of the canary, not 8" "$CASE/root/docs-r2-v1/canary-needs-reader"

canary_case
rm "$CASE/root/docs-r2-v1/canary-recorded"
run canary "$CASE/root" docs-r2-v1
check "canary: before canary-recorded exits 6" code_is 6
check "canary: and writes nothing" sh -c "[ ! -e '$CASE/root/docs-r2-v1/canary-passed' ] && [ ! -e '$CASE/root/docs-r2-v1/canary-needs-reader' ]"

canary_case
sed -i 's/named 0 paths outside/named 1 path outside/' "$(rec L1-p1-r3)/record.md"
: > "$CASE/root/docs-r2-v1/canary-passed"
run canary "$CASE/root" docs-r2-v1
check "canary: a canary-passed a person wrote stands" code_is 0
check "canary: and is not overwritten" [ ! -s "$CASE/root/docs-r2-v1/canary-passed" ]

# ------------------------------------------------------ result: the keep rule

# screening <file> <tier> <round> <arm>: a committed transcript of the case's
# checkout, with the front matter `result` selects on.
screening() {
    mkdir -p "$checkout/docs/probe-runs"
    printf -- '---\nid: HW-RUN-%s\ntier: %s\nround: %s\narm: %s\ntitle: "x"\n---\n\n# x\n' \
        "$1" "$2" "$3" "$4" > "$checkout/docs/probe-runs/$1.md"
}
# finished_try <try> [<all>] [<stage>] [<jobs>]: a try root whose driver
# wrote `done`, two sessions of 1200 and 1252 cents, a drive.out whose last
# `all=` line is <all>, 2452 when it is not given, the stage, a confirmation
# of docs-r<n>-v1 when it is not given, and a batch that planned <jobs> jobs,
# 145 when it is not given.
finished_try() {
    t=$CASE/root/$1
    mkdir -p "$t/a"
    echo "additive scenario-1 tuning" > "$t/spec"
    round=${1#docs-r}
    round=${round%%-*}
    echo "${3:-confirm docs-r$round-v1}" > "$t/stage"
    : > "$t/a/jobs"
    j=0
    while [ "$j" -lt "${4:-145}" ]; do
        j=$((j + 1))
        echo "L1-p1-r$j 1 additive scenario-1 tuning HW-PROBE-a" >> "$t/a/jobs"
    done
    cost "$t/a/sessions/L1-p1-r1" 1200
    cost "$t/a/sessions/L1-p1-r2" 1252
    printf 'campaign-drive: canary exit=0 recorded=8/72 new=8 cents=400 all=400/3000\ncampaign-drive: a exit=0 recorded=72/72 new=64 cents=%s all=%s/3000\n' \
        "${2:-2452}" "${2:-2452}" > "$t/drive.out"
    : > "$t/done"
}
result_case() {
    new_case
    checkout=$CASE/checkout
    screening additive-screening-scenario-0 additive screening scenario-0
    screening additive-screening-scenario-1 additive screening scenario-1
    # Neither of these is the best so far of scenario 1: a prefix of another
    # arm, and the same arm in another tier and round.
    screening additive-screening-scenario-10 additive screening scenario-10
    screening ablation-scenario-1 ablation none scenario-1
    finished_try docs-r2-confirm
    echo 'pooled: 11/72 against 11/72, +0.0 points [-12.8, +12.8]' > "$CASE/pooled-base"
    # The best so far of docs, measured at the confirmation's count.
    best_event additive-screening-scenario-1 145
}
# best_event <transcript> <sessions>: a `best` event of docs in the ledger.
best_event() {
    printf '{"event": "best", "try": "docs-r1-best", "layer": "docs", "round": 1, "transcript": "docs/probe-runs/%s.md", "used": 0, "sessions": %s, "count": %s}\n' \
        "$1" "$2" "$2" >> "$CASE/root/ledger.jsonl"
}
ledger_has() { grep -qF -- "$1" "$CASE/root/ledger.jsonl"; }
# measured_base: a `base` event at 145 sessions at the pin of the case's
# base-pin, which a confirmation's plan reads (#1838).
measured_base() {
    printf '{"event": "base", "try": "base-1", "scenario": "0", "pin": "%s", "count": 145, "transcript": "docs/probe-runs/additive-screening-scenario-0.md"}\n' \
        "$(awk '{ print $2 }' "$CASE/root/base-pin")" >> "$CASE/root/ledger.jsonl"
}
result_md_has() { grep -qxF -- "$1" "$CASE/root/docs-r2-confirm/result.md"; }

# Each row is the pooled keep row that `headwater probe compare` prints for
# that try against a best of 39 of 72, computed with the Newcombe interval of
# grade.rs, then the verdict and the direction. The rule is the owner's of
# 2026-10-20 on #1777: kept only when the whole interval sits above zero. The
# rule the issue first wrote, "strictly above the best so far", keeps 40.
while IFS='|' read -r row_try row_line row_keep row_direction; do
    result_case
    echo "$row_line" > "$CASE/pooled-keep"
    run result "$CASE/root" docs-r2-confirm
    check "result $row_try: exits 0, kept or not" code_is 0
    check "result $row_try: prints keep: $row_keep" out_has "keep: $row_keep"
    check "result $row_try: prints direction: $row_direction" out_has "direction: $row_direction"
    check "result $row_try: result.md holds keep: $row_keep" result_md_has "keep: $row_keep"
    check "result $row_try: result.md holds the keep row verbatim" grep -qF -- "$row_line" "$CASE/root/docs-r2-confirm/result.md"
    if [ "$row_keep" = kept ]; then row_kept=true; else row_kept=false; fi
    check "result $row_try: the ledger records kept $row_kept" ledger_has "\"kept\": $row_kept"
    check "result $row_try: and closes round 2 of docs" ledger_has "\"event\": \"round\", \"layer\": \"docs\", \"round\": 2, \"kept\": $row_kept"
done <<'EOF'
51/72|pooled: 51/72 against 39/72, +16.7 points [+0.9, +31.4]|kept|up
50/72|pooled: 50/72 against 39/72, +15.3 points [-0.6, +30.1]|not kept|up
40/72|pooled: 40/72 against 39/72, +1.4 points [-14.5, +17.2]|not kept|up
39/72|pooled: 39/72 against 39/72, +0.0 points [-15.9, +15.9]|not kept|flat
32/72|pooled: 32/72 against 39/72, -9.7 points [-25.2, +6.5]|not kept|down
low +0.0|pooled: 61/100 against 39/100, +15.0 points [+0.0, +30.0]|not kept|up
low -0.0|pooled: 61/100 against 39/100, +15.0 points [-0.0, +30.0]|not kept|up
flat -0.0|pooled: 39/72 against 39/72, -0.0 points [-15.9, +15.9]|not kept|flat
EOF

# What one result writes, on the row that is kept.
result_case
echo 'pooled: 51/72 against 39/72, +16.7 points [+0.9, +31.4]' > "$CASE/pooled-keep"
run result "$CASE/root" docs-r2-confirm
check "result: the keep row compares the new transcript against the scenario-1 screening transcript" grep -q 'probe compare .*docs/probe-runs/additive-tuning-of-.*-docs-round-2-confirmation-scenario-1\.md .*/docs/probe-runs/additive-screening-scenario-1\.md$' "$CASE/engine.log"
check "result: the base row compares it against the scenario-0 screening transcript" grep -q 'probe compare .*docs/probe-runs/additive-tuning-of-.*-docs-round-2-confirmation-scenario-1\.md .*/docs/probe-runs/additive-screening-scenario-0\.md$' "$CASE/engine.log"
check "result: result.md holds the base row verbatim" result_md_has "base row: pooled: 11/72 against 11/72, +0.0 points [-12.8, +12.8]"
check "result: result.md holds the spend" result_md_has "spent: 2452"
check "result: result.md states the rule and cites it" grep -qF 'owner, 2026-10-20, #1777' "$CASE/root/docs-r2-confirm/result.md"
check "result: the transcript is written with the facets tier, arm and round" grep -q '^new probe_transcript .*--facet tier=additive --facet arm=scenario-1 --facet round=tuning' "$CASE/engine.log"
result_transcript=$(ls "$checkout"/docs/probe-runs/additive-tuning-of-*-docs-round-2-confirmation-scenario-1.md 2>/dev/null)
check "result: the transcript is in the checkout" [ -f "$result_transcript" ]
check "result: its Run identity holds the assembled identity block" grep -qx 'cost_cents: 2452' "$result_transcript"
check "result: its Events hold the assembled events block" grep -qx '  session: "L1-additive-scenario-1-p1-r1"' "$result_transcript"
check "result: no TODO is left in the two sections" sh -c "! grep -q TODO '$result_transcript'"
check "result: the ledger records the result event" ledger_has '"event": "result"'
check "result: with its layer and scenario" sh -c "grep -qF '\"layer\": \"docs\"' '$CASE/root/ledger.jsonl' && grep -qF '\"scenario\": \"1\"' '$CASE/root/ledger.jsonl'"
check "result: and its direction" ledger_has '"direction": "up"'
check "result: and the spend" ledger_has '"spent": 2452'
run result "$CASE/root" docs-r2-confirm
check "result: a try that has a result.md exits 6" code_is 6
check "result: and appends no second event" [ "$(grep -c '"event": "result"' "$CASE/root/ledger.jsonl")" = 1 ]

# The best so far is the latest kept confirmation of the layer, and one that
# is not kept never becomes it.
finished_try docs-r3-confirm
echo 'pooled: 40/72 against 51/72, -15.3 points [-30.1, +0.6]' > "$CASE/pooled-keep"
run result "$CASE/root" docs-r3-confirm
check "result: round 3 exits 0" code_is 0
check "result: its keep row is against round 2, which was kept" grep -q 'probe compare .*-docs-round-3-confirmation-scenario-1\.md .*-docs-round-2-confirmation-scenario-1\.md$' "$CASE/engine.log"
finished_try docs-r4-confirm
run result "$CASE/root" docs-r4-confirm
check "result: round 4's keep row is against round 2 still, because round 3 was not kept" grep -q 'probe compare .*-docs-round-4-confirmation-scenario-1\.md .*-docs-round-2-confirmation-scenario-1\.md$' "$CASE/engine.log"

# A kept try of another layer is not the best so far of this one.
result_case
printf '%s\n' '{"event": "result", "try": "hooks-r1-confirm", "layer": "hooks", "scenario": "2", "stage": "confirm", "transcript": "docs/probe-runs/additive-screening-scenario-10.md", "used": 9, "sessions": 145, "count": 145, "kept": true}' >> "$CASE/root/ledger.jsonl"
echo 'pooled: 51/72 against 39/72, +16.7 points [+0.9, +31.4]' > "$CASE/pooled-keep"
run result "$CASE/root" docs-r2-confirm
check "result: a kept try of the hooks layer is not the best so far of docs" grep -q 'probe compare .*-docs-round-2-confirmation-scenario-1\.md .*/additive-screening-scenario-1\.md$' "$CASE/engine.log"

# The spend: the session files against the driver's last `all=` line.
result_case
rm -r "$CASE/root/docs-r2-confirm"
finished_try docs-r2-confirm 2451
echo 'pooled: 51/72 against 39/72, +16.7 points [+0.9, +31.4]' > "$CASE/pooled-keep"
run result "$CASE/root" docs-r2-confirm
check "result: session files of 2452 against all=2451 exit 2" code_is 2
check "result: and name both figures" sh -c "grep -q 2452 '$CASE/err' && grep -q 2451 '$CASE/err'"
check "result: and write no result.md" [ ! -e "$CASE/root/docs-r2-confirm/result.md" ]
check "result: and assemble nothing" sh -c "! grep -q assemble '$CASE/trees.log' 2>/dev/null"
result_case
sed -i '/all=/d' "$CASE/root/docs-r2-confirm/drive.out"
run result "$CASE/root" docs-r2-confirm
check "result: a drive.out with no all= line exits 2" code_is 2

# The keep row has no interval when a side graded no session.
result_case
echo 'pooled: 0/0 against 39/72, no difference: the treated transcript graded no session' > "$CASE/pooled-keep"
run result "$CASE/root" docs-r2-confirm
check "result: a keep row with no interval exits 2" code_is 2
check "result: and says the keep row has no interval" err_has "the keep row has no interval"
check "result: and writes no result.md" [ ! -e "$CASE/root/docs-r2-confirm/result.md" ]
check "result: and records no result event" sh -c "! grep -q '\"event\": \"result\"' '$CASE/root/ledger.jsonl' 2>/dev/null"

# The base row's control must be one transcript.
result_case
screening additive-screening-scenario-0-again additive screening scenario-0
echo 'pooled: 51/72 against 39/72, +16.7 points [+0.9, +31.4]' > "$CASE/pooled-keep"
run result "$CASE/root" docs-r2-confirm
check "result: two screening transcripts of the base scenario exit 2" code_is 2
check "result: and name both" sh -c "grep -q additive-screening-scenario-0.md '$CASE/err' && grep -q additive-screening-scenario-0-again.md '$CASE/err'"
result_case
rm "$checkout/docs/probe-runs/additive-screening-scenario-0.md"
run result "$CASE/root" docs-r2-confirm
check "result: no screening transcript of the base scenario exits 2" code_is 2

# A confirmation is kept against a best so far at its own count alone.
result_case
: > "$CASE/root/ledger.jsonl"
echo 'pooled: 51/72 against 39/72, +16.7 points [+0.9, +31.4]' > "$CASE/pooled-keep"
run result "$CASE/root" docs-r2-confirm
check "result: a confirmation with no best so far exits 2" code_is 2
check "result: and says so" err_has "no best so far at the confirmation's count"
check "result: and writes no transcript" sh -c "! grep -q '^new ' '$CASE/engine.log' 2>/dev/null"
result_case
: > "$CASE/root/ledger.jsonl"
best_event additive-screening-scenario-1 25
run result "$CASE/root" docs-r2-confirm
check "result: a best so far of 25 sessions, a screen's count, exits 2" code_is 2
check "result: and names the count the keep rule reads" err_has "at 145 alone"
result_case
: > "$CASE/root/ledger.jsonl"
printf '%s\n' '{"event": "screen", "try": "docs-r1-v1", "layer": "docs", "round": 1, "transcript": "docs/probe-runs/additive-screening-scenario-1.md", "used": 9, "sessions": 25}' >> "$CASE/root/ledger.jsonl"
run result "$CASE/root" docs-r2-confirm
check "result: a screened variant is never the best so far" code_is 2

# The batch ran the stage's count, and the try root is the stage's.
result_case
rm -r "$CASE/root/docs-r2-confirm"
finished_try docs-r2-confirm 2452 "confirm docs-r2-v1" 25
run result "$CASE/root" docs-r2-confirm
check "result: a confirmation whose batch planned 25 jobs exits 2" code_is 2
check "result: and names both counts" sh -c "grep -q 'runs 145 sessions' '$CASE/err' && grep -q 'planned 25 jobs' '$CASE/err'"
result_case
finished_try docs-r2-v1 2452 "screen 1" 145
run result "$CASE/root" docs-r2-v1
check "result: a screen whose batch planned 145 jobs exits 2" code_is 2
result_case
rm -r "$CASE/root/docs-r2-confirm"
finished_try docs-r2-confirm 2452 "screen 2" 25
run result "$CASE/root" docs-r2-confirm
check "result: a screen of 25 jobs in the confirmation's try root exits 2" code_is 2
check "result: and records nothing" sh -c "! grep -q '\"event\": \"screen\"' '$CASE/root/ledger.jsonl'"
finished_try docs-r2-v1 2452 "screen 2" 25
run result "$CASE/root" docs-r2-v1
check "result: a screen of variant 2 in the try root of variant 1 exits 2" code_is 2
result_case
rm "$CASE/root/docs-r2-confirm/stage"
run result "$CASE/root" docs-r2-confirm
check "result: a try root with no stage exits 2" code_is 2
result_case
finished_try docs-try2
run result "$CASE/root" docs-try2
check "result: a try root of the form <layer>-try<n>, from before the rounds, exits 2" code_is 2

result_case
: > "$CASE/assemble-fails"
run result "$CASE/root" docs-r2-confirm
check "result: an assembly that refuses exits 2" code_is 2
check "result: and writes no transcript" sh -c "! grep -q '^new ' '$CASE/engine.log' 2>/dev/null"

# A try with no `done`: a halt, and no verdict.
result_case
rm "$CASE/root/docs-r2-confirm/done"
: > "$CASE/root/docs-r2-confirm/capped"
echo 'pooled: 51/72 against 39/72, +16.7 points [+0.9, +31.4]' > "$CASE/pooled-keep"
run result "$CASE/root" docs-r2-confirm
check "result: a try with no done exits 7" code_is 7
check "result: and appends a halt event" ledger_has '"event": "halt"'
check "result: with the spend" ledger_has '"spent": 2452'
check "result: and the markers" ledger_has '"markers": "capped"'
check "result: and prints no keep: line" sh -c "! grep -q '^keep:' '$CASE/out'"
check "result: and writes no result.md" [ ! -e "$CASE/root/docs-r2-confirm/result.md" ]
check "result: and writes no transcript" sh -c "! grep -q '^new ' '$CASE/engine.log' 2>/dev/null"

# result: the oracle's verdict is the pass (#1875, clause 4). A probe that
# grades against a command oracle shows the oracle's verdict, read from each
# session's `oracles` record, beside the grader's count, and never the
# grader's count under the oracle's name. A probe with no command oracle
# shows the grader's count. Here the oracle passed one session of two and the
# grader passed both, so the line says they disagree.
oracle_session() {
    # oracle_session <session> <probe> <status> <verdict>
    mkdir -p "$CASE/root/docs-r2-confirm/a/sessions/$1"
    printf -- '--probe\n%s\n--session\n%s\n' "$2" "$1" > "$CASE/root/docs-r2-confirm/a/sessions/$1/record.argv"
    echo "$3" > "$CASE/root/docs-r2-confirm/a/sessions/$1/status"
    [ -z "${4:-}" ] || printf 'fixture-oracle\t%s\tfixture-oracle: %s: x\n' "$4" "$4" > "$CASE/root/docs-r2-confirm/a/sessions/$1/oracles"
}
oracle_case() {
    result_case
    echo 'pooled: 51/72 against 39/72, +16.7 points [+0.9, +31.4]' > "$CASE/pooled-keep"
    printf -- '- HW-PROBE-a (docs/probes/a.md)\n    category: sufficiency\n    action: change_governed_code\n    oracle: command:fixture-oracle\n- HW-PROBE-b (docs/probes/b.md)\n    category: discovery\n    action: ask_or_locate\n' > "$CASE/plan-plain"
    printf '%s\n' 'HW-PROBE-a: 2/2 against 2/2, +0.0 points [-50.0, +50.0]' 'HW-PROBE-b: 1/2 against 1/2, +0.0 points [-50.0, +50.0]' 'pooled: 3/4 against 3/4, +0.0 points [-40.0, +40.0]' > "$CASE/probe-rows"
    oracle_session L1-p1-r1 HW-PROBE-a 0 pass
    oracle_session L1-p1-r2 HW-PROBE-a 0 fail
}
oracle_case
run result "$CASE/root" docs-r2-confirm
check "result: the oracle's verdict: exits 0" code_is 0
check "result: a command-oracle probe shows the oracle's pass beside the grader's" result_md_has "probe HW-PROBE-a (change_governed_code): 1 of 2 by the oracle fixture-oracle (grader 2 of 2) disagree"
check "result: a probe with no command oracle shows the grader's count" result_md_has "probe HW-PROBE-b (ask_or_locate): 1 of 2 by the grader"
check "result: a try with oracle records prints no not-recorded line" sh -c "! grep -q '^oracle: not recorded' '$CASE/root/docs-r2-confirm/result.md'"
check "result: the plan is the round's set in the try's scenario" grep -q '^probe plan .*--tier additive --arm scenario-1 --set ' "$CASE/engine.log"
check "result: the probe lines are not in the ledger" sh -c "! grep -q 'by the oracle' '$CASE/root/ledger.jsonl'"
# A session that did not record (status other than 0), and a session of
# another probe, count in no oracle pass. Both sessions pass, so the two
# counts agree and the line carries no disagree.
oracle_case
oracle_session L1-p1-r2 HW-PROBE-a 0 pass
oracle_session L1-p1-r3 HW-PROBE-a 3 fail
oracle_session L1-p1-r4 HW-PROBE-b 0 fail
run result "$CASE/root" docs-r2-confirm
check "result: sessions that did not record, and other probes' sessions, are not counted" result_md_has "probe HW-PROBE-a (change_governed_code): 2 of 2 by the oracle fixture-oracle (grader 2 of 2)"
# A probe whose plan line is `oracle: test:<name>` grades against a
# cargo-test oracle, which is a row 2-9 or 11 oracle too (HW-DR-0112), so it
# shows the oracle's verdict as a `command:` probe does, read from the record
# line of the bare name.
oracle_case
printf -- '- HW-PROBE-a (docs/probes/a.md)\n    category: sufficiency\n    action: change_governed_code\n    oracle: test:fixture-test\n- HW-PROBE-b (docs/probes/b.md)\n    category: discovery\n    action: ask_or_locate\n' > "$CASE/plan-plain"
printf 'fixture-oracle\tpass\t-\nfixture-test\tpass\ttest result: ok. 1 passed; 0 failed\n' > "$CASE/root/docs-r2-confirm/a/sessions/L1-p1-r1/oracles"
printf 'fixture-oracle\tpass\t-\nfixture-test\tfail\ttest result: FAILED. 0 passed; 1 failed\n' > "$CASE/root/docs-r2-confirm/a/sessions/L1-p1-r2/oracles"
run result "$CASE/root" docs-r2-confirm
check "result: a test-oracle probe shows the oracle's pass beside the grader's" result_md_has "probe HW-PROBE-a (change_governed_code): 1 of 2 by the oracle fixture-test (grader 2 of 2) disagree"
# A try whose sessions hold no record at all predates the record, and still
# renders, with the grader's count for every probe.
oracle_case
rm "$CASE/root/docs-r2-confirm/a/sessions/L1-p1-r1/oracles" "$CASE/root/docs-r2-confirm/a/sessions/L1-p1-r2/oracles"
run result "$CASE/root" docs-r2-confirm
check "result: a try with no oracle record exits 0" code_is 0
check "result: and says the record predates #1875" result_md_has "oracle: not recorded (the batch predates #1875)"
check "result: and shows the grader's count for a command-oracle probe" result_md_has "probe HW-PROBE-a (change_governed_code): 2 of 2 by the grader"
# A screen writes the same lines.
result_case
rm -r "$CASE/root/docs-r2-confirm"
finished_try docs-r2-v1 2452 "screen 1" 25
printf -- '- HW-PROBE-a (docs/probes/a.md)\n    action: change_governed_code\n    oracle: command:fixture-oracle\n' > "$CASE/plan-plain"
printf '%s\n' 'HW-PROBE-a: 2/2 against 2/2, +0.0 points [-50.0, +50.0]' > "$CASE/probe-rows"
mkdir -p "$CASE/root/docs-r2-v1/a/sessions/L1-p1-r1"
printf -- '--probe\nHW-PROBE-a\n' > "$CASE/root/docs-r2-v1/a/sessions/L1-p1-r1/record.argv"
echo 0 > "$CASE/root/docs-r2-v1/a/sessions/L1-p1-r1/status"
printf 'fixture-oracle\tpass\t-\n' > "$CASE/root/docs-r2-v1/a/sessions/L1-p1-r1/oracles"
run result "$CASE/root" docs-r2-v1
check "result: a screen shows the oracle's verdict too" grep -qxF "probe HW-PROBE-a (change_governed_code): 1 of 1 by the oracle fixture-oracle (grader 2 of 2) disagree" "$CASE/root/docs-r2-v1/result.md"
rm -f "$CASE/plan-plain" "$CASE/probe-rows"
checkout=$fix/repo

# ------------------------------------------------------------------ ledger

result_case
rm "$CASE/root/ledger.jsonl"
run ledger "$CASE/root"
check "ledger: no ledger.jsonl prints no events" out_has "no events"
check "ledger: and exits 0" code_is 0
best_event additive-screening-scenario-1 145
echo 'pooled: 51/72 against 39/72, +16.7 points [+0.9, +31.4]' > "$CASE/pooled-keep"
run result "$CASE/root" docs-r2-confirm
finished_try docs-r3-confirm
rm "$CASE/root/docs-r3-confirm/done"
run result "$CASE/root" docs-r3-confirm
run ledger "$CASE/root"
check "ledger: four events print four lines and the total" [ "$(grep -c '^[a-z]*: ' "$CASE/out")" = 5 ]
check "ledger: the best so far" out_has "best: try=docs-r1-best"
check "ledger: the result" out_has "result: try=docs-r2-confirm"
check "ledger: the round it closed" out_has "round: layer=docs round=2 kept=true"
check "ledger: the halt" out_has "halt: try=docs-r3-confirm"
check "ledger: the total is the round's spend" out_has "spent: 4904"
echo 'not json' >> "$CASE/root/ledger.jsonl"
run ledger "$CASE/root"
check "ledger: a line that is not JSON exits 2" code_is 2
checkout=$fix/repo

# ------------------------------------------------------------------ accept

RULING=https://github.com/headwater-ai/headwater/issues/1777#issuecomment-123456
accept_case() {
    new_case
    echo 'pub fn x() { changed() }' > "$CASE/tree-head/engine/crates/x/src/lib.rs"
}
accept_case
run plan "$CASE/root"
check "accept: a changed engine source makes plan exit 4" code_is 4
run accept "$CASE/root" "https://example.org/ruling"
check "accept: a URL that is not a GitHub ruling exits 2" code_is 2
check "accept: and records nothing" [ ! -e "$CASE/root/ledger.jsonl" ]
run accept "$CASE/root" "https://github.com/headwater-ai/headwater/issues/1777"
check "accept: an issue URL with no comment exits 2" code_is 2
run accept "$CASE/root" "$RULING"
check "accept: a ruling URL exits 0" code_is 0
check "accept: and prints the path it accepts" out_has "engine/crates/x/src/lib.rs"
check "accept: and records the accept event" ledger_has '"event": "accept"'
check "accept: with the ruling" ledger_has "\"ruling\": \"$RULING\""
run plan "$CASE/root"
check "accept: plan then exits 0" code_is 0
check "accept: and prints accepted: with the ruling" out_has "accepted: engine/crates/x/src/lib.rs ($RULING)"
echo 'pub fn x() { changed_again() }' > "$CASE/tree-head/engine/crates/x/src/lib.rs"
run plan "$CASE/root"
check "accept: the path changed again after the acceptance makes plan exit 4" code_is 4
check "accept: and plan prints no accepted: line" sh -c "! grep -q '^accepted:' '$CASE/out'"
accept_case
run accept "$CASE/root" "$RULING"
echo 'pub fn y() {}' > "$CASE/tree-head/engine/crates/x/src/other.rs"
run plan "$CASE/root"
check "accept: a second invalidating path the acceptance does not list makes plan exit 4" code_is 4
check "accept: while the first is still accepted" out_has "accepted: engine/crates/x/src/lib.rs"
accept_case
run accept "$CASE/root" "$RULING"
echo '0 1111111111111111111111111111111111111111' > "$CASE/root/base-pin"
run plan "$CASE/root"
check "accept: an acceptance at another pin covers nothing" code_is 4
accept_case
rm "$CASE/tree-head/engine/crates/x/src/lib.rs"
run accept "$CASE/root" "https://github.com/headwater-ai/headwater/pull/1827#discussion_r99"
check "accept: a pull request URL and a removed path exit 0" code_is 0
check "accept: and record the path as absent" ledger_has '"engine/crates/x/src/lib.rs": "absent"'
run plan "$CASE/root"
check "accept: plan then accepts the removal" code_is 0
new_case
run accept "$CASE/root" "$RULING"
check "accept: nothing invalidating exits 0" code_is 0
check "accept: and says nothing to accept" out_has "nothing to accept"
check "accept: and records nothing" [ ! -e "$CASE/root/ledger.jsonl" ]

# -------------------------------------------------------------------- wait

new_case
mkdir -p "$CASE/root/docs-r2-v1"
for marker in done canary-failed capped stop; do
    : > "$CASE/root/docs-r2-v1/$marker"
    run wait "$CASE/root" docs-r2-v1
    check "wait: $marker exits 0" code_is 0
    check "wait: and prints $marker" sh -c "[ \"\$(cat '$CASE/out')\" = '$marker' ]"
    rm "$CASE/root/docs-r2-v1/$marker"
done
: > "$CASE/root/docs-r2-v1/canary-recorded"
run wait "$CASE/root" docs-r2-v1
check "wait: canary-recorded without canary-passed exits 0" code_is 0
check "wait: and prints canary-recorded" out_has canary-recorded
: > "$CASE/root/docs-r2-v1/canary-needs-reader"
run wait "$CASE/root" docs-r2-v1
check "wait: canary-needs-reader is printed before canary-recorded" sh -c "[ \"\$(cat '$CASE/out')\" = canary-needs-reader ]"
: > "$CASE/root/docs-r2-v1/canary-passed"
run wait "$CASE/root" docs-r2-v1
check "wait: a canary that passed and a driver that is gone print driver-exited" sh -c "[ \"\$(cat '$CASE/out')\" = driver-exited ]"
echo 'TRY_FLOOR_TRY_KB=999999999999' >> "$CASE/root/env"
run wait "$CASE/root" docs-r2-v1
check "wait: below the disk floor exits 0" code_is 0
check "wait: and prints disk-floor" out_has disk-floor
check "wait: and writes stop" [ -s "$CASE/root/docs-r2-v1/stop" ]
# A driver that stopped on the usage gate (#1876): `wait` names the stop,
# and the ledger records it once over two reads, and again for a new stop.
new_case
mkdir -p "$CASE/root/docs-r2-v1"
printf '%s\n' "2026-10-29T00:00:00Z usage-gated: 7-day at 86% (from /u/s1.json), threshold 85%, resets 2026-10-30T00:00:00Z" \
    'percent: 86' 'resets-at: 1793318400' 'threshold: 85' 'source: /u/s1.json' > "$CASE/root/docs-r2-v1/usage-gated"
run wait "$CASE/root" docs-r2-v1
check "wait: usage-gated exits 0" code_is 0
check "wait: and prints usage-gated with the line" out_has "usage-gated: 2026-10-29T00:00:00Z usage-gated: 7-day at 86%"
run wait "$CASE/root" docs-r2-v1
check "wait: usage-gated read twice exits 0" code_is 0
check "wait: and the ledger holds one gate event" [ "$(grep -c '"event": "gate"' "$CASE/root/ledger.jsonl")" = 1 ]
check "wait: which names the try, the figure, the reset and the threshold" grep -q '"try": "docs-r2-v1", "percent": "86", "resets_at": 1793318400, "threshold": 85' "$CASE/root/ledger.jsonl"
sed -i '1s/^2026-10-29T00:00:00Z/2026-10-29T06:00:00Z/' "$CASE/root/docs-r2-v1/usage-gated"
run wait "$CASE/root" docs-r2-v1
check "wait: a second stop appends a second gate event" [ "$(grep -c '"event": "gate"' "$CASE/root/ledger.jsonl")" = 2 ]
: > "$CASE/root/docs-r2-v1/stop"
run wait "$CASE/root" docs-r2-v1
check "wait: stop is printed before usage-gated" sh -c "[ \"\$(cat '$CASE/out')\" = stop ]"
new_case
run wait "$CASE/root"
check "wait: with no try named exits 2, because a round runs several tries" code_is 2
run wait "$CASE/root" ../x
check "wait: a try that is not a directory name exits 2" code_is 2

# ------------------------- rank and confirm: usage before pass rate (#1838)

# The decisive case of the rounds of #1838. The owner ruled on 2026-10-22
# (#1777) that a round screens 4 variants at 24 sessions, ranks them on the
# use of the layer first and on the pass rate second, and confirms only a
# variant that raises the use, at 144 sessions against the best so far at
# 144. Over the five tuning probes each count rounds up: ceil(24 / 5) = 5
# repetitions, 25 sessions, and ceil(144 / 5) = 29, 145. Variant 1 passes 22
# of 25 and never calls the skill, variant 2 passes 14 of 25 and calls it in
# 12, and the best so far calls it in 3 of 145. Variant 2 is ranked first and
# confirmed at 145. Variant 1 is not confirmed, whatever its pass rate.

# rank_decl <open> [<ceiling_cents>] [<base_ceiling_cents>]: a declaration
# with the skills layer and the seven fields of the rounds, at one cent a
# session, so a cap of 3000 holds a confirmation.
rank_decl() {
    cat > "$CASE/probe.yml" <<EOF
layers:
  skills: {paths: [.claude/skills/hw-x]}
  docs: {}
  hooks: {}
sets:
  tuning: [HW-PROBE-a, HW-PROBE-b, HW-PROBE-c, HW-PROBE-d, HW-PROBE-e]
scenarios:
  0: {layers: []}
  1: {base: 0, layers: [skills]}
  2: {base: 0, layers: [docs]}
  3: {base: 0, layers: [hooks]}
tiers:
  additive: {session_cost_cents: 1}
rounds:
  tuning:
    set: tuning
    rates: [0.625, 0.375]
    tries: 4
    approved_cents: 100000
    screen: 24
    confirm: 144
    variants: 4
    stop_after_empty: 2
    order: [skills, docs, hooks]
    ceiling_cents: ${2:-99000}
    base_ceiling_cents: ${3:-1000}
    open: $1
base_invalidators:
  - engine/crates/*/src/**
  - tools/probe/campaign.sh
EOF
}
OPEN_SKILLS='{layer: skills, scenario: 1, try: 1, cap_cents: 3000, plan: "https://example.org/accepted", change: none}'

# skill_batch <try> <stage> <sessions> <used>: a try root the driver
# finished, whose batch planned <sessions> jobs, each session one cent, the
# first <used> of which call the shipped skill hw-x. Every other session
# calls a skill the layer does not ship, which is not a use of it.
skill_batch() {
    t=$CASE/root/$1
    mkdir -p "$t/a/sessions"
    echo "additive scenario-1 tuning" > "$t/spec"
    echo "$2" > "$t/stage"
    : > "$t/a/jobs"
    i=1
    while [ "$i" -le "$3" ]; do
        s=$t/a/sessions/L1-p1-r$i
        mkdir -p "$s"
        echo "L1-p1-r$i 1 additive scenario-1 tuning HW-PROBE-a" >> "$t/a/jobs"
        echo 0 > "$s/status"
        echo 1 > "$s/cost"
        if [ "$i" -le "$4" ]; then skill=hw-x; else skill=other; fi
        printf '{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Skill","input":{"skill":"%s"}}]}}\n' "$skill" > "$s/raw.jsonl"
        i=$((i + 1))
    done
    printf 'campaign-drive: a exit=0 recorded=%s/%s new=%s cents=%s all=%s/3000\n' "$3" "$3" "$3" "$3" "$3" > "$t/drive.out"
    : > "$t/done"
}
rank_case() {
    new_case
    rank_decl "$OPEN_SKILLS" "${1:-99000}"
    checkout=$CASE/checkout
    screening additive-screening-scenario-0 additive screening scenario-0
    screening additive-screening-scenario-1 additive screening scenario-1
    # The best so far, at the confirmation's count: 3 of 145 call the skill.
    skill_batch skills-r1-best best 145 3
    echo 'pooled: 80/145 against 78/145, +1.4 points [-10.0, +12.8]' > "$CASE/pooled-base"
    run result "$CASE/root" skills-r1-best
    measured_base
}
# screened <variant k> <used> <passed>: one screened variant, recorded.
screened() {
    skill_batch "skills-r1-v$1" "screen $1" 25 "$2"
    echo "pooled: $3/25 against 11/25, +0.0 points [-1.0, +1.0]" > "$CASE/pooled-base"
    run result "$CASE/root" "skills-r1-v$1"
}

rank_case
check "rank: the best so far at 145 is recorded" code_is 0
check "rank: and its use, 3 of 145" ledger_has '"used": 3, "sessions": 145'
screened 1 0 22
check "rank: variant 1 is recorded" code_is 0
check "rank: a screen is never kept" out_has "keep: never"
check "rank: its use is 0 of 25" ledger_has '"try": "skills-r1-v1"'
screened 2 12 14
run rank "$CASE/root"
check "decisive: rank exits 0" code_is 0
check "decisive: variant 2, which uses the skill in 12 of 25, ranks first though it passes 14 of 25" \
    out_has "rank 1: skills-r1-v2 used in 12 of 25 sessions, passed 14 of 25, gain"
check "decisive: variant 1, which passes 22 of 25 and never uses it, ranks second, with no gain" \
    out_has "rank 2: skills-r1-v1 used in 0 of 25 sessions, passed 22 of 25, no gain"
run launch "$CASE/root" confirm skills-r1-v1
check "decisive: a confirmation of variant 1 exits 6" code_is 6
check "decisive: and says no gain in use" err_has "no gain in use"
check "decisive: and invokes nothing" no_calls
check "decisive: and makes no confirmation root" [ ! -e "$CASE/root/skills-r1-confirm" ]
check "decisive: and closes no round, because variant 1 is not the top" sh -c "! grep -q '\"event\": \"round\"' '$CASE/root/ledger.jsonl'"
run plan "$CASE/root" confirm skills-r1-v2
check "decisive: a confirmation of variant 2 plans" code_is 0
check "decisive: at 145 sessions, ceil(144 / 5) = 29 repetitions of 5 probes" out_has "stage: confirm skills-r1-v2, 145 sessions"
printf -- '--pin\n%s\n' "$(git -C "$fix/repo" rev-parse HEAD)" > "$CASE/root/skills-r1-v2/driver.argv"
checkout=$fix/repo
run launch "$CASE/root" confirm skills-r1-v2
await_calls 1
check "decisive: the confirmation launches" code_is 0
check "decisive: in its own try root, with its stage" sh -c "[ \"\$(cat '$CASE/root/skills-r1-confirm/stage')\" = 'confirm skills-r1-v2' ]"
check "decisive: and the driver reads the confirmation's count" grep -q ' stage=confirm$' "$CASE/calls"
checkout=$CASE/checkout
rm -f "$CASE/root/skills-r1-confirm/driver.pid"
skill_batch skills-r1-confirm "confirm skills-r1-v2" 145 30
echo 'pooled: 82/145 against 80/145, +1.4 points [-10.0, +12.8]' > "$CASE/pooled-base"
echo 'pooled: 82/145 against 80/145, +1.4 points [-10.0, +12.8]' > "$CASE/pooled-keep"
run result "$CASE/root" skills-r1-confirm
check "decisive: the confirmation is recorded" code_is 0
check "decisive: and kept on its own interval, which holds zero, so not kept" out_has "keep: not kept"
check "decisive: its keep row is against the best so far at 145" grep -q 'probe compare .*-skills-round-1-confirmation-scenario-1\.md .*-skills-round-1-best-so-far-scenario-1\.md$' "$CASE/engine.log"
check "decisive: and the round is closed with nothing kept" ledger_has '"event": "round", "layer": "skills", "round": 1, "kept": false'
checkout=$fix/repo

# The top variant does not raise the use: the round closes with nothing kept.
rank_case
screened 1 0 25
run launch "$CASE/root" confirm skills-r1-v1
check "rank: a top variant with no gain in use exits 6" code_is 6
check "rank: and closes the round with nothing kept" ledger_has '"reason": "no-gain-in-use"'
run plan "$CASE/root" screen 2
check "rank: a closed round screens no more variants" code_is 6
check "rank: and says it is closed" err_has "round 1 of the skills layer is closed"
checkout=$fix/repo

# A screen of 25 of 25 does not keep its variant: the confirmation does,
# and its interval holds zero.
rank_case
screened 1 12 25
printf -- '--pin\n%s\n' "$(git -C "$fix/repo" rev-parse HEAD)" > "$CASE/root/skills-r1-v1/driver.argv"
checkout=$fix/repo
run launch "$CASE/root" confirm skills-r1-v1
await_calls 1
checkout=$CASE/checkout
skill_batch skills-r1-confirm "confirm skills-r1-v1" 145 40
echo 'pooled: 85/145 against 80/145, +3.4 points [-7.9, +14.6]' > "$CASE/pooled-keep"
run result "$CASE/root" skills-r1-confirm
check "rank: a variant that screened 25 of 25, confirmed with an interval that holds zero, is not kept" out_has "keep: not kept"
check "rank: and the ledger keeps nothing of the screen" sh -c "! grep -q '\"kept\": true' '$CASE/root/ledger.jsonl'"
checkout=$fix/repo

# ---------------------------------------------------- rounds: stages (#1838)

# A screen is one of `variants` variants, each on a tree of its own.
new_case
run plan "$CASE/root" screen 4
check "screen: the fourth of 4 variants plans" code_is 0
check "screen: at 25 sessions, ceil(24 / 5) = 5 repetitions of 5 probes" out_has "stage: screen 4, 25 sessions, 25 cents at 1 a session, in docs-r2-v4"
check "screen: and builds the open scenario's tree at HEAD" grep -qx 'tree 1 HEAD' "$CASE/trees.log"
run plan "$CASE/root" screen 5
check "screen: a fifth variant of 4 exits 6" code_is 6
check "screen: and says how many a round screens" err_has "at most 4 variants"
new_case
run launch "$CASE/root" screen 1
await_calls 1
run plan "$CASE/root" screen 2
check "screen: a variant on the tree another variant screened exits 6" code_is 6
check "screen: and names that variant" err_has "docs-r2-v1 screened"
run launch "$CASE/root" screen 2
check "screen: and is not launched" [ "$(calls_count)" = 1 ]
echo 'another variant' >> "$CASE/tree-head/README.md"
run plan "$CASE/root" screen 2
check "screen: a variant on a tree of its own plans" code_is 0

# A best so far at the confirmation's count runs only where none is.
new_case
run plan "$CASE/root" best
check "best: with no measurement at 145, a best so far plans" code_is 0
check "best: at 145 sessions" out_has "stage: best, 145 sessions"
check "best: the first best so far of the layer is charged to the base ceiling" out_has "left of the bases: 145 sessions, 145 cents"
check "best: and what is left under the tuning ceiling is the confirmation" out_has "left: 145 sessions"
printf '%s\n' '{"event": "best", "try": "docs-r1-best", "layer": "docs", "round": 1, "transcript": "x.md", "used": 1, "sessions": 145, "count": 145}' > "$CASE/root/ledger.jsonl"
run plan "$CASE/root" best
check "best: with a best so far measured at 145, exits 6" code_is 6
run plan "$CASE/root"
check "best: and a round with one prices no second" out_has "left: 245 sessions"

# A confirmation stands on a ranked round and a best so far.
new_case
printf '%s\n' '{"event": "screen", "try": "docs-r2-v1", "layer": "docs", "round": 2, "used": 5, "sessions": 25, "passed": 10, "graded": 25}' > "$CASE/root/ledger.jsonl"
mkdir -p "$CASE/root/docs-r2-v1"
: > "$CASE/root/docs-r2-v1/result.md"
run plan "$CASE/root" confirm docs-r2-v1
check "confirm: with no best so far exits 6" code_is 6
check "confirm: and names the best so far to launch" err_has "launch $CASE/root best"
printf '%s\n' '{"event": "best", "try": "docs-r1-best", "layer": "docs", "round": 1, "transcript": "x.md", "used": 1, "sessions": 145, "count": 145}' >> "$CASE/root/ledger.jsonl"
measured_base
mkdir -p "$CASE/root/docs-r2-v2"
run plan "$CASE/root" confirm docs-r2-v1
check "confirm: while a screen of the round has no result exits 6" code_is 6
check "confirm: and names it" err_has "docs-r2-v2 has no result.md"
: > "$CASE/root/docs-r2-v2/result.md"
printf '%s\n' '{"event": "screen", "try": "docs-r2-v2", "layer": "docs", "round": 2, "used": 6, "sessions": 25, "passed": 3, "graded": 25}' >> "$CASE/root/ledger.jsonl"
run plan "$CASE/root" confirm docs-r2-v1
check "confirm: a variant with a gain in use that is not the top exits 6" code_is 6
check "confirm: and names the top" err_has "docs-r2-v2 is"
run plan "$CASE/root" confirm docs-r2-v2
check "confirm: the top variant, with a gain in use, plans" code_is 0
check "confirm: and prints both uses" sh -c "grep -qx 'best so far: x.md, used in 1 of 145 sessions' '$CASE/out' && grep -qx 'variant: docs-r2-v2, used in 6 of 25 sessions, passed 3 of 25' '$CASE/out'"
run plan "$CASE/root" confirm docs-r1-v2
check "confirm: a variant of another round exits 6" code_is 6
run launch "$CASE/root" confirm docs-r2-v2
check "confirm: from a checkout that is not at the variant's pin exits 6" code_is 6
check "confirm: and names the pin it needs" err_has "no pin it records"
check "confirm: and invokes nothing" no_calls
# The equal share is no gain: 29 of 145 and 5 of 25 are one fifth each.
new_case
printf '%s\n' '{"event": "best", "try": "docs-r1-best", "layer": "docs", "round": 1, "transcript": "x.md", "used": 29, "sessions": 145, "count": 145}' > "$CASE/root/ledger.jsonl"
printf '%s\n' '{"event": "screen", "try": "docs-r2-v1", "layer": "docs", "round": 2, "used": 5, "sessions": 25, "passed": 25, "graded": 25}' >> "$CASE/root/ledger.jsonl"
measured_base
run plan "$CASE/root" confirm docs-r2-v1
check "confirm: a use of 5 of 25 over a best of 29 of 145, the same share, is no gain" code_is 6
check "confirm: and plan closes no round" sh -c "! grep -q '\"event\": \"round\"' '$CASE/root/ledger.jsonl'"
sed -i 's/"used": 5,/"used": 6,/' "$CASE/root/ledger.jsonl"
run plan "$CASE/root" confirm docs-r2-v1
check "confirm: and 6 of 25 is a gain" code_is 0

# A change to the engine's source makes the base stale for a confirmation,
# whatever the owner accepted, because it can change what route prints.
new_case
printf '%s\n' '{"event": "best", "try": "docs-r1-best", "layer": "docs", "round": 1, "transcript": "x.md", "used": 1, "sessions": 145, "count": 145}' > "$CASE/root/ledger.jsonl"
printf '%s\n' '{"event": "screen", "try": "docs-r2-v1", "layer": "docs", "round": 2, "used": 5, "sessions": 25, "passed": 10, "graded": 25}' >> "$CASE/root/ledger.jsonl"
echo 'pub fn x() { routed() }' > "$CASE/tree-head/engine/crates/x/src/lib.rs"
run accept "$CASE/root" "https://github.com/headwater-ai/headwater/issues/1777#issuecomment-1"
run plan "$CASE/root" screen 1
check "route: an accepted engine change lets a screen plan" code_is 0
run plan "$CASE/root" confirm docs-r2-v1
check "route: and refuses a confirmation with 4" code_is 4
check "route: naming the path" out_has "route: engine/crates/x/src/lib.rs"
check "route: and the re-measure it needs" err_has "sh tools/probe/try.sh launch $CASE/root base"

# ------ base: a route change waits for a measured base at 145 (#1838)

# The decisive case of the base stage. A pin written by hand is not a
# measurement, so it does not clear the route gate: only a `base` event at the
# confirmation's count does, and `result` of a `base` try writes both.
head_sha=$(git -C "$fix/repo" rev-parse HEAD)
# The hand-written pin names another commit than the launch's, so the case
# also reads that `result` rewrites it.
echo '0 1111111111111111111111111111111111111111' > "$CASE/root/base-pin"
cp "$CASE/tree-head/engine/crates/x/src/lib.rs" "$CASE/tree-pin/engine/crates/x/src/lib.rs"
run plan "$CASE/root" confirm docs-r2-v1
check "base: a pin written by hand at the engine's source does not let the confirmation plan" code_is 4
check "base: and says no base is measured at that pin" err_has "no base event at 145 sessions"
check "base: and names the launch that measures one" err_has "sh tools/probe/try.sh launch $CASE/root base"
run launch "$CASE/root" base
await_calls 1
check "base: launch base starts the driver" code_is 0
check "base: in base-1" [ -d "$CASE/root/base-1" ]
check "base: on the base scenario's tuning line" sh -c "[ \"\$(cat '$CASE/root/base-1/spec')\" = 'additive scenario-0 tuning' ]"
check "base: with the stage base" sh -c "[ \"\$(cat '$CASE/root/base-1/stage')\" = base ]"
check "base: and the driver reads the confirmation's count" grep -q ' stage=confirm$' "$CASE/calls"
rm -f "$CASE/root/base-1/driver.pid"
checkout=$CASE/checkout
screening additive-screening-scenario-0 additive screening scenario-0
finished_try base-1 145 base 145
echo 'additive scenario-0 tuning' > "$CASE/root/base-1/spec"
cost "$CASE/root/base-1/a/sessions/L1-p1-r1" 100
cost "$CASE/root/base-1/a/sessions/L1-p1-r2" 45
echo 'pooled: 70/145 against 33/72, +2.4 points [-11.0, +15.8]' > "$CASE/pooled-base"
run result "$CASE/root" base-1
check "base: result of a base exits 0" code_is 0
check "base: and keeps nothing" out_has "keep: never"
check "base: it appends a base event at 145 sessions at the launch's pin" \
    ledger_has "\"event\": \"base\", \"try\": \"base-1\", \"scenario\": \"0\", \"pin\": \"$head_sha\", \"count\": 145"
check "base: and writes the base-pin itself" sh -c "[ \"\$(cat '$CASE/root/base-pin')\" = '0 $head_sha' ]"
check "base: its transcript says it is a base re-measure" grep -q '^title: "Additive tuning of .*, base re-measure of scenario 0"$' "$CASE/checkout/$(cat "$CASE/root/base-1/transcript")"
check "base: and its base row is against the screening transcript it replaces" grep -q 'probe compare .*base-re-measure-of-scenario-0\.md .*additive-screening-scenario-0\.md$' "$CASE/engine.log"
checkout=$fix/repo
run plan "$CASE/root" confirm docs-r2-v1
check "base: with the base measured at the engine's source, the confirmation plans" code_is 0
check "base: and names the measurement it read" out_has "measured: scenario 0 at $head_sha, 145 sessions, in base-1"
: > "$CASE/trees.log"
run plan "$CASE/root" base
check "base: a second base at a commit that has one exits 6" code_is 6
check "base: and names the measurement" err_has "is measured at $head_sha already"
check "base: and builds no tree" no_trees
# A screen after the re-measure reads its base row against the new base.
checkout=$CASE/checkout
finished_try docs-r2-v2 2452 "screen 2" 25
echo 'pooled: 12/25 against 70/145, +0.0 points [-1.0, +1.0]' > "$CASE/pooled-keep"
run result "$CASE/root" docs-r2-v2
check "base: a screen after the re-measure is recorded" code_is 0
check "base: and its base row is against the re-measured base, not the screening transcript" grep -q 'probe compare .*variant-2-scenario-1\.md .*base-re-measure-of-scenario-0\.md$' "$CASE/engine.log"
checkout=$fix/repo
echo 'pub fn x() { routed_again() }' > "$CASE/tree-head/engine/crates/x/src/lib.rs"
run plan "$CASE/root" confirm docs-r2-v1
check "base: a second engine change after the re-measure refuses the confirmation with 4 again" code_is 4
run plan "$CASE/root" screen 3
check "base: and a screen, whose acceptance named the old pin, is stale by today's gate" code_is 4
# The owner accepts the change at the new pin: a screen plans, and the
# confirmation still waits for a base measured at the engine's source.
run accept "$CASE/root" "https://github.com/headwater-ai/headwater/issues/1777#issuecomment-2"
run plan "$CASE/root" screen 3
check "base: an accepted engine change at the new pin lets a screen plan" code_is 0
run plan "$CASE/root" confirm docs-r2-v1
check "base: and still refuses the confirmation with 4, whatever was accepted" code_is 4
check "base: naming the route path" out_has "route: engine/crates/x/src/lib.rs (accepted by https://github.com/headwater-ai/headwater/issues/1777#issuecomment-2"
check "base: and the launch that measures it" err_has "a route change makes scenario 0 stale for a confirmation"

# A base event at another count, or of another scenario, does not back the
# pin: the confirmation reads a base at its own count.
new_case
printf '%s\n' '{"event": "best", "try": "docs-r1-best", "layer": "docs", "round": 1, "transcript": "x.md", "used": 1, "sessions": 145, "count": 145}' > "$CASE/root/ledger.jsonl"
printf '%s\n' '{"event": "screen", "try": "docs-r2-v1", "layer": "docs", "round": 2, "used": 5, "sessions": 25, "passed": 10, "graded": 25}' >> "$CASE/root/ledger.jsonl"
printf '%s\n' '{"event": "base", "try": "base-1", "scenario": "0", "pin": "2d212a269a111fddb7af5b2b75ebaf2f9ab32f29", "count": 25, "transcript": "x.md"}' >> "$CASE/root/ledger.jsonl"
printf '%s\n' '{"event": "base", "try": "base-2", "scenario": "3", "pin": "2d212a269a111fddb7af5b2b75ebaf2f9ab32f29", "count": 145, "transcript": "x.md"}' >> "$CASE/root/ledger.jsonl"
run plan "$CASE/root" confirm docs-r2-v1
check "base: a base of 25 sessions, and one of another scenario, at the pin do not back it" code_is 4
sed -i 's/"count": 25,/"count": 145,/' "$CASE/root/ledger.jsonl"
run plan "$CASE/root" confirm docs-r2-v1
check "base: and one of 145 sessions of the base scenario does" code_is 0
check "base: and it is the one named" out_has "145 sessions, in base-1: x.md"
sed -i 's/"pin": "2d212a269a111fddb7af5b2b75ebaf2f9ab32f29", "count": 145, "transcript": "x.md"}/"pin": "2d212a26", "count": 145, "transcript": "x.md"}/' "$CASE/root/ledger.jsonl"
run plan "$CASE/root" confirm docs-r2-v1
check "base: and one at another pin does not" code_is 4

# ----------------------------------------------------- rounds: order (#1838)

new_case
decl "$OPEN_HOOK" 10000
run plan "$CASE/root"
check "order: a round of hooks while docs, before it, is open exits 6" code_is 6
check "order: and names docs" err_has "the docs layer comes before hooks"
check "order: and builds nothing" no_trees
round_event() { printf '{"event": "round", "layer": "%s", "round": %s, "kept": %s}\n' "$1" "$2" "$3" >> "$CASE/root/ledger.jsonl"; }
round_event docs 1 false
run plan "$CASE/root"
check "order: one empty round of docs does not stop it" code_is 6
round_event docs 2 false
run plan "$CASE/root"
check "order: two empty rounds in a row stop docs, and hooks plans" code_is 0
check "order: as round 1 of hooks" out_has "layer: hooks open, rounds closed: -"
decl "$OPEN_DOCS" 10000
run plan "$CASE/root"
check "order: a stopped layer plans no round" code_is 6
check "order: and says it stopped" err_has "the docs layer stopped"
new_case
round_event docs 1 true
run plan "$CASE/root"
check "order: a closed round 1 does not close round 2" code_is 0
round_event docs 2 false
run plan "$CASE/root"
check "order: a round the ledger closed plans nothing more" code_is 6
round_event docs 3 false
round_event docs 4 true
decl "$OPEN_HOOK" 10000
run plan "$CASE/root"
check "order: a kept round after two empty ones does not stop docs, and its fourth round exhausts it" code_is 0
decl "$OPEN_DOCS" 10000
run plan "$CASE/root"
check "order: a layer that took its 4 rounds exits 6" code_is 6
check "order: and says so" err_has "took its 4 rounds"
new_case
sed -i 's/^    order: \[docs, hooks\]$/    order: [hooks]/' "$CASE/probe.yml"
run plan "$CASE/root"
check "order: a layer that order does not name exits 6" code_is 6

# --------------------------------------------- rounds: the ceiling (#1838)

# What is left of round 2 with no stage is 4 x 25 + 145 = 245 cents under the
# tuning ceiling, and the first best so far, 145, under the base ceiling.
# With a ceiling of 5000, a spend of 4755 is at it exactly.
ceiling_case() {
    new_case
    decl "$OPEN_DOCS" 10000 5000
    cost "$CASE/root/docs-try1/a/sessions/s1" "$1"
}
ceiling_case 4756
run plan "$CASE/root"
check "ceiling: one cent above it exits 5" code_is 5
check "ceiling: and names the ceiling" err_has "above the tuning ceiling of 5000 cents"
check "ceiling: and builds no tree" no_trees
run launch "$CASE/root" screen 1
check "ceiling: a launch one cent above it exits 5" code_is 5
check "ceiling: and invokes nothing" no_calls
ceiling_case 4755
run plan "$CASE/root"
check "ceiling: at it exactly exits 0" code_is 0
check "ceiling: and prints what is left under the tuning ceiling" out_has "left: 245 sessions of the round, 245 cents at 1 a session"
check "ceiling: and under the base ceiling" out_has "left of the bases: 145 sessions, 145 cents at 1 a session"
ceiling_case 100
run plan "$CASE/root"
check "ceiling: well below it exits 0" code_is 0
# The fourth screen leaves 25 + 145 = 170: 4830 is at the ceiling.
ceiling_case 4831
run plan "$CASE/root" screen 4
check "ceiling: the fourth screen one cent above it exits 5" code_is 5
ceiling_case 4830
run plan "$CASE/root" screen 4
check "ceiling: the fourth screen at it exits 0" code_is 0
# A stage priced above the cap of one launch would stop unfinished.
new_case
decl '{layer: docs, scenario: 1, try: 2, cap_cents: 144, plan: "https://example.org/accepted", change: none}' 10000
run plan "$CASE/root" best
check "cap: a best so far of 145 cents under a cap of 144 exits 5" code_is 5
check "cap: and says the driver would stop it" err_has "above the cap of one launch, 144"
run plan "$CASE/root" screen 1
check "cap: a screen of 25 cents under it plans" code_is 0

# ------------------------------------- rounds: the base ceiling (#1838)

# base-1 and the first best so far of a layer are charged to the base
# ceiling, so their spend does not move the check of a screen against the
# tuning ceiling. The best so far is measured, so none is left to price.
ceiling_case 4755
cost "$CASE/root/base-1/a/sessions/s1" 600
cost "$CASE/root/docs-r1-best/a/sessions/s1" 400
best_event x 145
run plan "$CASE/root" screen 1
check "split: 1000 cents of bases leave a screen at the tuning ceiling exactly" code_is 0
check "split: and the spend prints both sums" out_has "spent: 5755, of which the bases 1000 and the tries 4755"
run spent "$CASE/root"
check "split: spent prints the total" sh -c "head -1 '$CASE/out' | grep -qx 5755"
check "split: and the bases against their ceiling" out_has "bases: 1000 of the base ceiling, 1000"
check "split: and the tries against theirs" out_has "tries: 4755 of the tuning ceiling, 5000"
run status "$CASE/root"
check "split: status prints both sums too" sh -c "grep -qx 'bases: 1000 of the base ceiling, 1000' '$CASE/out' && grep -qx 'tries: 4755 of the tuning ceiling, 5000' '$CASE/out'"
cost "$CASE/root/docs-try1/a/sessions/s2" 1
run plan "$CASE/root" screen 1
check "split: one cent more of tries exits 5" code_is 5
check "split: and names the tuning ceiling" err_has "above the tuning ceiling of 5000 cents"
# A later best so far of the layer is a try.
ceiling_case 4755
cost "$CASE/root/docs-r1-best/a/sessions/s1" 1
run plan "$CASE/root" screen 1
check "split: a best so far of round 2 after one of round 1 is charged to the tuning ceiling" code_is 5
check "split: and is priced there" out_has "left: 390 sessions of the round, 390 cents at 1 a session, the best so far at 145 included"
# base-2 is charged to the tuning ceiling, and base-1 to the base ceiling.
ceiling_case 4855
run plan "$CASE/root" base
check "split: base-1 is charged to the base ceiling" code_is 0
check "split: and leaves nothing under the tuning ceiling" out_has "left of the bases: 145 sessions"
check "split: in base-1" out_has "in base-1"
check "split: and builds no tree" no_trees
mkdir -p "$CASE/root/base-1"
run plan "$CASE/root" base
check "split: base-2 at the tuning ceiling exactly plans" code_is 0
check "split: in base-2" out_has "in base-2"
cost "$CASE/root/docs-try1/a/sessions/s2" 1
run plan "$CASE/root" base
check "split: and one cent above it exits 5" code_is 5
check "split: naming the tuning ceiling" err_has "above the tuning ceiling of 5000 cents"
# The four bases of #1833 at the declared rate of 50 cents a session: one
# base and three first bests are 4 x 7250 = 29000 cents, above 22000.
four_case() {
    new_case
    rank_decl '{layer: hooks, scenario: 3, try: 1, cap_cents: 8000, plan: "https://example.org/accepted", change: none, moment: intent}' 78000 22000
    sed -i 's/session_cost_cents: 1}/session_cost_cents: 50}/' "$CASE/probe.yml"
    round_event skills 1 false
    round_event skills 2 false
    round_event docs 1 false
    round_event docs 2 false
    cost "$CASE/root/base-1/a/sessions/s1" "$1"
    cost "$CASE/root/skills-r1-best/a/sessions/s1" "$2"
    cost "$CASE/root/docs-r1-best/a/sessions/s1" "$3"
}
four_case 7250 7250 7250
run plan "$CASE/root" best
check "four: the fourth charge to the base ceiling at the declared rate, 29000, exits 5" code_is 5
check "four: and names the base ceiling" err_has "above the base ceiling of 22000 cents"
check "four: and builds no tree" no_trees
four_case 7250 7250 251
run plan "$CASE/root" best
check "four: one cent above the base ceiling exits 5" code_is 5
four_case 7250 7250 250
run plan "$CASE/root" best
check "four: at the base ceiling exactly plans" code_is 0
# A confirmation that waits for a re-measure prices it: base-2 under the
# tuning ceiling, beside the confirmation.
new_case
decl "$OPEN_DOCS" 10000 5000
printf '%s\n' '{"event": "best", "try": "docs-r1-best", "layer": "docs", "round": 1, "transcript": "x.md", "used": 1, "sessions": 145, "count": 145}' > "$CASE/root/ledger.jsonl"
printf '%s\n' '{"event": "screen", "try": "docs-r2-v1", "layer": "docs", "round": 2, "used": 5, "sessions": 25, "passed": 10, "graded": 25}' >> "$CASE/root/ledger.jsonl"
mkdir -p "$CASE/root/base-1"
cost "$CASE/root/docs-try1/a/sessions/s1" 4710
run plan "$CASE/root" confirm docs-r2-v1
check "remeasure: a confirmation with no measured base and room for base-2 exits 4" code_is 4
check "remeasure: and prices base-2 beside it" out_has "left with the re-measure: 290 sessions of the round, 290 cents, the re-measure in base-2 included"
cost "$CASE/root/docs-try1/a/sessions/s2" 1
run plan "$CASE/root" confirm docs-r2-v1
check "remeasure: and one cent above the tuning ceiling with it exits 5" code_is 5
rm -rf "$CASE/root/base-1"
run plan "$CASE/root" confirm docs-r2-v1
check "remeasure: base-1 is priced under the base ceiling instead" code_is 4
check "remeasure: and says so" out_has "left of the bases with the re-measure: 145 sessions, 145 cents, in base-1"
cost "$CASE/root/docs-r1-best/a/sessions/s1" 855
run plan "$CASE/root" confirm docs-r2-v1
check "remeasure: base-1 at the base ceiling exactly exits 4" code_is 4
cost "$CASE/root/docs-r1-best/a/sessions/s1" 856
run plan "$CASE/root" confirm docs-r2-v1
check "remeasure: and one cent above it exits 5" code_is 5
check "remeasure: naming the base ceiling" err_has "above the base ceiling of 1000 cents"

# ------------------- the fixed draw: one session per tuning probe (#1875)

# The decisive case of #1875: a screen of 22 tuning probes at `screen: 22`
# runs 22 sessions, one for each probe, on a draw of one phrasing for each
# probe that is the same for every variant of the round. Variants ranked on
# different task lists would rank the draw and not the change. The checkout
# holds the 22 probe documents: one with three phrasings, one with two, and
# twenty with none, which count as one phrasing.
draw_repo=$fix/repo-draw
git init -q "$draw_repo"
mkdir -p "$draw_repo/docs/probes"
draw_set=
i=1
while [ "$i" -le 22 ]; do
    id=HW-PROBE-p$(printf '%02d' "$i")
    draw_set="$draw_set${draw_set:+, }$id"
    {
        printf -- '---\nid: %s\n---\n\n# %s\n\n## Task\n\n' "$id" "$id"
        case $i in
            1) printf '### Phrasing 1\n\nOne of p01.\n\n### Phrasing 2\n\nTwo of p01.\n\n### Phrasing 3\n\nThree of p01.\n' ;;
            2) printf '### Phrasing 1\n\nOne of p02.\n\n### Phrasing 2\n\nTwo of p02.\n' ;;
            *) printf 'The task of %s.\n' "$id" ;;
        esac
        printf '\n## Target\n\nA link.\n'
    } > "$draw_repo/docs/probes/p$i.md"
    i=$((i + 1))
done
git -C "$draw_repo" add docs
git -C "$draw_repo" -c user.name=fixture -c user.email=fixture@example.org commit -q -m probes
git -C "$draw_repo" update-ref refs/remotes/origin/main HEAD
git -C "$draw_repo" checkout -q --detach
draw_decl() {
    sed -i "s/^  tuning: \[.*\]$/  tuning: [$draw_set]/; s/^    screen: 24$/    screen: 22/" "$CASE/probe.yml"
}
saved_checkout=$checkout
checkout=$draw_repo
new_case
draw_decl
run plan "$CASE/root" screen 1
check "draw: a screen of 22 tuning probes at screen: 22 plans" code_is 0
check "draw: at 22 sessions, one for each tuning probe" out_has "stage: screen 1, 22 sessions, 22 cents at 1 a session, in docs-r2-v1"
check "draw: and prints one draw line for each session" [ "$(grep -c '^draw: ' "$CASE/out")" = 22 ]
check "draw: in the order of the set" sh -c "grep '^draw: ' '$CASE/out' | awk '{ print \$2 }' | paste -sd, - | sed 's/,/, /g' | grep -qx '$draw_set'"
check "draw: a probe with three phrasings draws one of the three" grep -Eqx 'draw: HW-PROBE-p01 [123]' "$CASE/out"
check "draw: a probe with two phrasings draws one of the two" grep -Eqx 'draw: HW-PROBE-p02 [12]' "$CASE/out"
check "draw: a probe with no phrasing draws 1, its whole task" grep -qx 'draw: HW-PROBE-p03 1' "$CASE/out"
check "draw: the phrasing is phrasing.sh draw of the seed, the open layer, the round and the probe" \
    grep -qx "draw: HW-PROBE-p01 $(sh "$here/phrasing.sh" draw 0 docs 2 HW-PROBE-p01 1 3)" "$CASE/out"
grep '^draw: ' "$CASE/out" > "$CASE/draw-v1"
run plan "$CASE/root" screen 2
grep '^draw: ' "$CASE/out" > "$CASE/draw-v2"
check "draw: the second variant of the round plans the same 22 sessions" out_has "stage: screen 2, 22 sessions"
check "draw: decisive: on the same draw of 22 lines, byte for byte" sh -c "[ \"\$(wc -l < '$CASE/draw-v1')\" = 22 ] && cmp -s '$CASE/draw-v1' '$CASE/draw-v2'"
check "draw: and the plan stores no draw under the round root" [ ! -e "$CASE/root/docs-r2-v1" ]
run plan "$CASE/root" best
check "draw: a best so far draws nothing, because its phrasings are spread" sh -c "! grep -q '^draw: ' '$CASE/out'"
run launch "$CASE/root" screen 1
await_calls 1
check "draw: launch writes the draw in the try root, as plan printed it" cmp -s "$CASE/draw-v1" "$CASE/root/docs-r2-v1/draw"
check "draw: and the driver gets HEADWATER_CAMPAIGN_DRAW" grep -qx "draw $CASE/root/docs-r2-v1/draw" "$CASE/calls"
# The next reader is campaign.sh, which takes each job's phrasing from the
# file with `phrasing.sh pick`: it reads the phrasing the plan drew.
check "draw: and campaign.sh's pick reads the drawn phrasing from that file" \
    [ "$(sh "$here/phrasing.sh" pick HW-PROBE-p01 1 3 "$CASE/root/docs-r2-v1/draw")" = "$(awk '$2 == "HW-PROBE-p01" { print $3 }' "$CASE/draw-v1")" ]
echo 'another variant' >> "$CASE/tree-head/README.md"
run launch "$CASE/root" screen 2
await_calls 2
check "draw: decisive: the second variant launches on the same draw file, byte for byte" cmp -s "$CASE/root/docs-r2-v1/draw" "$CASE/root/docs-r2-v2/draw"
check "draw: and its driver gets its own" grep -qx "draw $CASE/root/docs-r2-v2/draw" "$CASE/calls"
run relaunch "$CASE/root" docs-r2-v1
await_calls 3
check "draw: relaunch exports the draw again" [ "$(grep -c "^draw $CASE/root/docs-r2-v1/draw\$" "$CASE/calls")" = 2 ]
# Another round draws on its own key, so the next round's draw is not this
# one carried forward: round 3 of docs gives p01 the draw of its own key.
new_case
draw_decl
sed -i 's/try: 2,/try: 3,/' "$CASE/probe.yml"
run plan "$CASE/root" screen 1
check "draw: round 3 draws phrasing.sh draw of round 3" \
    grep -qx "draw: HW-PROBE-p01 $(sh "$here/phrasing.sh" draw 0 docs 3 HW-PROBE-p01 1 3)" "$CASE/out"
checkout=$saved_checkout

# ------------- a hook variant: its moment plus row 11 (#1875, HW-DR-0112)

# A hook variant screens and confirms on the rows of its moment, plus row 11
# (`ungoverned_control`). The plan of each is a call of its own, because the
# two flags of one call intersect. The stub engine answers `--moment read`
# with b and d, and `--action ungoverned_control` with e and b, so the union
# in the order of the set is b, d and e.
OPEN_HOOK_READ='{layer: hooks, scenario: 2, try: 1, cap_cents: 3000, plan: "https://example.org/accepted", change: none, moment: read}'
hook_case() {
    new_case
    decl "$OPEN_HOOK_READ" 10000
    sed -i 's/^    order: \[docs, hooks\]$/    order: [hooks, docs]/' "$CASE/probe.yml"
    printf -- '- HW-PROBE-b (moments: read)\n- HW-PROBE-d (moments: read, write)\n' > "$CASE/plan-moment"
    printf -- '- HW-PROBE-e (action: ungoverned_control)\n- HW-PROBE-b (action: ungoverned_control)\n' > "$CASE/plan-action"
}
hook_case
run plan "$CASE/root" screen 1
check "hook: a read variant plans" code_is 0
check "hook: it selects the union of the moment and row 11, in the order of the set" \
    sh -c "[ \"\$(grep '^selected: ' '$CASE/out' | tr '\n' ' ')\" = 'selected: HW-PROBE-b selected: HW-PROBE-d selected: HW-PROBE-e ' ]"
check "hook: at the screen's repetitions, 5, over the 3 it selected" out_has "stage: screen 1, 15 sessions, 15 cents at 1 a session, in hooks-r1-v1"
check "hook: and draws for the probes it selected alone" [ "$(grep -c '^draw: HW-PROBE-[bde] ' "$CASE/out") $(grep -c '^draw: ' "$CASE/out")" = "15 15" ]
check "hook: it asks the plan for the moment in a call of its own" grep -q -- '--set tuning --moment read$' "$CASE/engine.log"
check "hook: and for row 11 in another, as ungoverned_control" grep -q -- '--set tuning --action ungoverned_control$' "$CASE/engine.log"
check "hook: never the two flags in one call, which intersect" sh -c "! grep -- '--moment' '$CASE/engine.log' | grep -q -- '--action'"
run plan "$CASE/root" best
check "hook: a best so far runs the selection at the confirmation's repetitions, 29 x 3" out_has "stage: best, 87 sessions"
run launch "$CASE/root" screen 1
await_calls 1
check "hook: launch writes each probe outside the selection as an exclude on the spec" \
    sh -c "[ \"\$(cat '$CASE/root/hooks-r1-v1/spec')\" = 'additive scenario-2 tuning HW-PROBE-a HW-PROBE-c' ]"
hook_case
: > "$CASE/plan-moment"
run plan "$CASE/root" screen 1
check "hook: a moment no probe carries is an empty part of the union, not an error" code_is 0
check "hook: and the selection is row 11 alone" sh -c "[ \"\$(grep -c '^selected: ' '$CASE/out')\" = 2 ]"
hook_case
: > "$CASE/plan-moment"
: > "$CASE/plan-action"
run plan "$CASE/root" screen 1
check "hook: a selection of no probe exits 6" code_is 6
check "hook: and launches nothing" no_calls
new_case
decl "$OPEN_HOOK" 10000
sed -i 's/^    order: \[docs, hooks\]$/    order: [hooks, docs]/' "$CASE/probe.yml"
run plan "$CASE/root" screen 1
check "hook: an intent variant selects every probe" sh -c "[ \"\$(grep -c '^selected: ' '$CASE/out')\" = 5 ]"
check "hook: without asking the plan" sh -c "! grep -q -- '--moment' '$CASE/engine.log' 2>/dev/null"
check "hook: at the screen's count" out_has "stage: screen 1, 25 sessions"
new_case
run plan "$CASE/root" screen 1
check "docs: a layer that is not hooks prints no selection" sh -c "! grep -q '^selected: ' '$CASE/out'"
# `result` reads the count of a try that left probes out as the stage's
# repetitions times the probes it ran: 29 x 3 = 87 for a confirmation that
# excludes 2 of 5. A batch of 145 jobs is then not that confirmation, and
# a best so far at 145 is not the count the keep rule reads it against.
saved_checkout=$checkout
result_case
echo "additive scenario-1 tuning HW-PROBE-d HW-PROBE-e" > "$CASE/root/docs-r2-confirm/spec"
run result "$CASE/root" docs-r2-confirm
check "subset: a confirmation that excludes 2 of 5 and planned 145 jobs exits 2" code_is 2
check "subset: and says it runs 87" err_has "a confirm runs 87 sessions, and the batch of $CASE/root/docs-r2-confirm planned 145 jobs"
result_case
finished_try docs-r2-confirm 2452 "confirm docs-r2-v1" 87
echo "additive scenario-1 tuning HW-PROBE-d HW-PROBE-e" > "$CASE/root/docs-r2-confirm/spec"
run result "$CASE/root" docs-r2-confirm
check "subset: at 87 jobs, a best so far at 145 is not the count the keep rule reads" err_has "and the keep rule reads a confirmation against a best so far at 87 alone"
checkout=$saved_checkout

# ----------------------------------------------- rounds: arguments (#1838)

new_case
run launch "$CASE/root"
check "args: launch with no stage exits 2" code_is 2
for bad in "screen" "screen 0" "screen x" "screen 01" "confirm" "confirm ../x" "final" "best 1"; do
    # shellcheck disable=SC2086
    run plan "$CASE/root" $bad
    check "args: plan $bad exits 2" code_is 2
done
check "args: and no bad stage builds a tree" no_trees
new_case
sed -i '/^    screen: 24$/d; /^    confirm: 144$/d; /^    variants: 4$/d; /^    stop_after_empty: 2$/d; /^    order: /d; /^    ceiling_cents: /d; /^    base_ceiling_cents: /d' "$CASE/probe.yml"
run plan "$CASE/root"
check "args: a tuning round with no rounds declared exits 2" code_is 2
check "args: and says it has no rounds" err_has "has no rounds to run"
for verb in relaunch canary result; do
    run "$verb" "$CASE/root"
    check "args: $verb with no try exits 2" code_is 2
done

# ------------------------------------------------------- usage.py (#1838)

# usage_session <batch> <id> <stream line> [<log line>]: one session.
usage_session() {
    mkdir -p "$1/sessions/$2"
    echo 0 > "$1/sessions/$2/status"
    printf '%s\n' "$3" > "$1/sessions/$2/raw.jsonl"
    if [ -n "${4:-}" ]; then
        mkdir -p "$1/config/$2/projects/x"
        printf '%s\n' "$4" > "$1/config/$2/projects/x/log.jsonl"
    fi
}
usage_of() { python3 "$here/usage.py" "$CASE/probe.yml" "$1" "$CASE/batch" > "$CASE/out" 2> "$CASE/err"; echo $? > "$CASE/code"; }
tool_line() { printf '{"type":"assistant","message":{"content":[{"type":"tool_use","name":"%s","input":%s}]}}' "$1" "$2"; }
new_case
rank_decl "$OPEN_SKILLS"
b=$CASE/batch
usage_session "$b" s1 "$(tool_line Skill '{"skill":"hw-x"}')"
usage_session "$b" s2 "$(tool_line Skill '{"skill":"plugin:hw-x"}')"
usage_session "$b" s3 "$(tool_line Skill '{"skill":"hw-y"}')"
usage_session "$b" s4 "$(tool_line Read '{"file_path":".claude/skills/hw-x/SKILL.md"}')"
mkdir -p "$b/sessions/s5"
usage_of skills
check "usage: skills counts a Skill call of a shipped skill, by its name after a plugin, and no other call" out_has "used 2 sessions 4"
# The text `headwater route` renders, as the intent hook prints it: the task
# echoed on the `route` line, an anchor the task names, and two pointer lines,
# the second of them to the path the task names.
routed='Headwater routed this task.\n\nroute \"Read docs/spec/03-z.md and answer.\"\n  names the anchor docs/spec/04-w.md\n  purpose rationale 3\n  docs/spec/01-x.md (One) — A summary that names docs/spec/05-v.md.\n    governs docs/spec/04-w.md\n  docs/spec/03-z.md (Three) — The task names it.\n'
# Claude Code 2.1.288, the version the campaign runs, logs the hook's output as
# `hook_success`; older logs carry it as `hook_additional_context`.
inject='{"type":"attachment","rendered":[{"content":"<system-reminder>x</system-reminder>"}],"attachment":{"type":"hook_success","hookName":"UserPromptSubmit","hookEvent":"UserPromptSubmit","content":"'"$routed"'"}}'
other='{"type":"attachment","attachment":{"type":"hook_success","hookName":"Stop","hookEvent":"Stop","content":"'"$routed"'"}}'
older='{"type":"attachment","attachment":{"type":"hook_additional_context","hookName":"UserPromptSubmit","content":"'"$routed"'"}}'
rm -rf "$b"
usage_session "$b" s1 "$(tool_line Read '{"file_path":"/work/docs/spec/01-x.md"}')" "$inject"
usage_session "$b" s2 "$(tool_line Bash '{"command":"sed -n 1,20p ./docs/spec/01-x.md"}')" "$inject"
usage_session "$b" s3 "$(tool_line Read '{"file_path":"docs/spec/02-y.md"}')" "$inject"
usage_session "$b" s4 "$(tool_line Bash '{"command":"echo docs/spec/01-x.md"}')" "$inject"
usage_session "$b" s5 "$(tool_line Read '{"file_path":"docs/spec/01-x.md"}')"
usage_session "$b" s6 "$(tool_line Read '{"file_path":"docs/spec/03-z.md"}')" "$inject"
usage_session "$b" s7 "$(tool_line Read '{"file_path":"docs/spec/04-w.md"}')" "$inject"
usage_session "$b" s8 "$(tool_line Read '{"file_path":"docs/spec/05-v.md"}')" "$inject"
usage_session "$b" s9 "$(tool_line Read '{"file_path":"docs/spec/01-x.md"}')" "$other"
usage_of hooks
check "usage: hooks counts a Read or a reader in Bash of a pointer in the UserPromptSubmit hook_success, and no path the task names, no anchor, evidence or summary path, and no other hook" out_has "used 2 sessions 9"
rm -rf "$b"
usage_session "$b" s1 "$(tool_line Read '{"file_path":"docs/spec/01-x.md"}')" "$older"
usage_session "$b" s2 "$(tool_line Read '{"file_path":"docs/spec/03-z.md"}')" "$older"
usage_of hooks
check "usage: hooks still counts a pointer in the hook_additional_context of an older log, and not the path the task names" out_has "used 1 sessions 2"
# Each of the four hooks, per hook and per session (#1875). Intent keeps the
# pointer-opened rule. Read, write and review count a hook_success with text:
# `PreToolUse:Read`, `PreToolUse:Write`, `PostToolUse:Edit` and `Stop`. A hook
# that printed nothing, or only white space, such as `touch.sh` on the write
# matcher, is no use,
# and neither is a hook the layer does not ship (`PreToolUse:Grep`).
out_line() { grep -qxF -- "$1" "$CASE/out"; }
out_first() { [ "$(head -1 "$CASE/out")" = "$1" ]; }
hook_line() { printf '{"type":"attachment","attachment":{"type":"hook_success","hookName":"%s","hookEvent":"%s","content":"%s"}}\n' "$1" "${1%%:*}" "$2"; }
rm -rf "$b"
usage_session "$b" s1 "$(tool_line Read '{"file_path":"docs/spec/01-x.md"}')" "$inject"
log=$b/config/s1/projects/x/log.jsonl
hook_line PreToolUse:Read 'docs/spec/01-x.md is governed by HW-DR-0001.' >> "$log"
hook_line PreToolUse:Write 'Use headwater new for a new document.' >> "$log"
hook_line PreToolUse:Write '' >> "$log"
hook_line PostToolUse:Edit '' >> "$log"
hook_line Stop 'headwater check found 1 error.' >> "$log"
usage_session "$b" s2 "$(tool_line Read '{"file_path":"docs/spec/02-y.md"}')" "$(hook_line PostToolUse:Edit 'The edit left a stale fold.')"
hook_line PreToolUse:Grep 'not a hook the layer ships' >> "$b/config/s2/projects/x/log.jsonl"
usage_session "$b" s3 "$(tool_line Read '{"file_path":"docs/spec/02-y.md"}')" "$(hook_line PreToolUse:Write '')"
hook_line Stop '\n ' >> "$b/config/s3/projects/x/log.jsonl"
# Claude Code 2.1.312 logs a hook's added context as a hook_additional_context
# whose content is a list of strings. A pointer line in the read hook's list
# counts for read and never for intent, though the session opened it.
listed='{"type":"attachment","rendered":["x"],"attachment":{"type":"hook_additional_context","hookName":"PreToolUse:Read","hookEvent":"PreToolUse","content":["Headwater routed this read.","  docs/spec/01-x.md (One) — A summary."]}}'
usage_session "$b" s4 "$(tool_line Read '{"file_path":"docs/spec/01-x.md"}')" "$listed"
usage_of hooks
check "usage: hooks keeps the first line on the intent rule, which try.sh rank reads" out_first "used 1 sessions 4"
check "usage: hooks counts the intent hook by the pointer it opened" out_line "hook intent used 1 sessions 4"
check "usage: hooks counts the read hook by a PreToolUse:Read with text, as a string or a list" out_line "hook read used 2 sessions 4"
check "usage: hooks counts the write hook by a Pre or PostToolUse on Write or Edit with text, and no empty one" out_line "hook write used 2 sessions 4"
check "usage: hooks counts the review hook by a Stop with text, and no empty one" out_line "hook review used 1 sessions 4"
check "usage: hooks prints the hooks each session used, in the order intent, read, write, review" out_line "session s1 intent read write review"
check "usage: hooks prints a session by the hooks it used, and no hook the layer does not ship" out_line "session s2 write"
check "usage: hooks prints a dash for a session that used no hook" out_line "session s3 -"
check "usage: hooks reads the list content of a hook_additional_context, and counts another hook's pointer for that hook and not for intent" out_line "session s4 read"
# The intent hook as Claude Code 2.1.312 logs it: a list whose first element is
# the heading and whose pointer line comes after it. A reader of the first
# element alone hides every intent use that try.sh rank reads.
intent_listed='{"type":"attachment","rendered":["x"],"attachment":{"type":"hook_additional_context","hookName":"UserPromptSubmit","hookEvent":"UserPromptSubmit","content":["Headwater routed this task.","  docs/spec/01-x.md (One) — A summary."]}}'
rm -rf "$b"
usage_session "$b" s1 "$(tool_line Read '{"file_path":"docs/spec/01-x.md"}')" "$intent_listed"
usage_of hooks
check "usage: hooks reads every element of the intent hook's list, and counts a pointer after the first" out_line "hook intent used 1 sessions 1"
check "usage: hooks names intent for a session that opened a pointer from the intent hook's list" out_line "session s1 intent"
rm -rf "$b"
usage_session "$b" s1 "$(tool_line mcp__headwater__explain '{"path":"x"}')"
usage_session "$b" s2 "$(tool_line Read '{"file_path":"mcp__x"}')"
usage_session "$b" s3 "$(tool_line mcp_headwater_explain '{"path":"x"}')"
usage_of mcp
check "usage: mcp counts a call to an MCP tool, named mcp__ and not mcp_, and nothing else" out_has "used 1 sessions 3"
usage_of docs
check "usage: a layer the round does not tune exits 2" code_is 2
rm -rf "$b"
usage_of mcp
check "usage: a batch with no sessions directory exits 2" code_is 2

# ---------------------------------------------------------------- arguments

new_case
run
check "args: no verb exits 2" code_is 2
run frobnicate "$CASE/root"
check "args: an unknown verb exits 2" code_is 2
run spent "$CASE/root" extra
check "args: spent with a try exits 2" code_is 2
run ledger "$CASE/root" extra
check "args: ledger with a try exits 2" code_is 2
run accept "$CASE/root"
check "args: accept with no ruling URL exits 2" code_is 2

# TRY_FIXTURES_KEEP keeps the cases' directories, to read a failure.
if [ -n "${TRY_FIXTURES_KEEP:-}" ]; then echo "try-fixtures: the cases are in $fix" >&2; else rm -rf "$fix"; fi
echo "try-fixtures: $passed passed, $failed failed"
[ "$failed" = 0 ]
