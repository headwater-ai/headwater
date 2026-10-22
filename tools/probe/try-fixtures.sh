#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
#
# The fixtures of tools/probe/try.sh, the ten verbs of one try of the
# additive campaign's tuning round (#1821, #1822).
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
        for control in "$@"; do :; done
        echo "HW-PROBE-a: 9/18 against 9/18, +0.0 points [-30.0, +30.0]"
        case $control in
            */additive-screening-scenario-0.md) cat "$CASE/pooled-base" ;;
            *) cat "$CASE/pooled-keep" ;;
        esac
        ;;
esac
EOF
chmod +x "$fix/engine-stub.sh"

OPEN_DOCS='{layer: docs, scenario: 1, try: 2, cap_cents: 3000, plan: "https://example.org/accepted", change: none}'
OPEN_HOOK='{layer: hook, scenario: 2, try: 1, cap_cents: 3000, plan: "https://example.org/accepted", change: none}'

# decl <open> <approved_cents>: the declaration of the case.
decl() {
    cat > "$CASE/probe.yml" <<EOF
layers:
  docs: {}
  hook: {}
sets:
  tuning: [HW-PROBE-a, HW-PROBE-b]
scenarios:
  0: {layers: []}
  1: {base: 0, layers: [docs]}
  2: {base: 0, layers: [hook]}
rounds:
  tuning:
    set: tuning
    rates: [0.625, 0.375]
    tries: 3
    approved_cents: $2
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
run launch "$CASE/root"
check "decisive: launch at approval + 1 exits 5" code_is 5
check "decisive: it prints what was spent" out_has "spent: 7001"
check "decisive: it prints the cap" out_has "cap: 3000"
check "decisive: it prints the approval" out_has "approved: 10000"
check "decisive: the driver is not invoked" no_calls
check "decisive: no tree is built" no_trees
check "decisive: no try root is made" [ ! -e "$CASE/root/docs-try2" ]
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
check "base: and the try may launch" out_has "plan: a try of docs on scenario 1 may launch"

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
run launch "$CASE/root"
await_calls 1
check "launch: exits 0" code_is 0
check "launch: the driver is invoked once" [ "$(calls_count)" = 1 ]
check "launch: the try root is <layer>-try<n>, with the open layer's cap" grep -q -- "--root $CASE/root/docs-try2 .* --cap-cents 3000 " "$CASE/calls"
check "launch: with the constants of additive-launch.env" grep -q -- "--claude-version 2.1.288 --spec-a $CASE/root/docs-try2/spec --model claude-sonnet-5 --session-cents 300 --cap-cents 3000 --parallel 4 --overshoot-cents 100 --seed 0 " "$CASE/calls"
check "launch: with --on-canary running canary on the try" grep -q -- "--on-canary sh '.*/try.sh' canary '$CASE/root' 'docs-try2'\$" "$CASE/calls"
check "launch: the host facts of <root>/env reach the driver" grep -qx "env versions=$CASE/versions target=$CASE/target tmp=$CASE/tmp pwd=.*" "$CASE/calls"
check "launch: the spec is one line, outside the checkout" sh -c "[ \"\$(cat '$CASE/root/docs-try2/spec')\" = 'additive scenario-1 tuning' ]"
check "launch: driver.argv, driver.pid and launched are written" sh -c "[ -s '$CASE/root/docs-try2/driver.argv' ] && [ -s '$CASE/root/docs-try2/driver.pid' ] && [ -s '$CASE/root/docs-try2/launched' ]"
run launch "$CASE/root"
check "launch: a try root that exists exits 6" code_is 6
check "launch: and invokes nothing more" [ "$(calls_count)" = 1 ]

new_case
rm "$CASE/root/env"
run launch "$CASE/root"
check "launch: no <root>/env exits 2" code_is 2
check "launch: no env invokes nothing" no_calls
new_case
echo 'PATH=/tmp/evil' >> "$CASE/root/env"
run launch "$CASE/root"
check "launch: an env line of another key exits 2" code_is 2
check "launch: an env of another key invokes nothing" no_calls
new_case
decl none 10000
run launch "$CASE/root"
check "launch: no open layer exits 5" code_is 5
check "launch: no open layer invokes nothing" no_calls
new_case
echo 'pub fn y() {}' >> "$CASE/tree-head/engine/crates/x/src/lib.rs"
run launch "$CASE/root"
check "launch: a stale base exits 4" code_is 4
check "launch: a stale base invokes nothing" no_calls
check "launch: a stale base makes no try root" [ ! -e "$CASE/root/docs-try2" ]

# The driver's pin check refuses each of these checkouts, so a launch that
# started it would report a driver that exits at once.
for state in branch dirty ahead; do
    new_case
    checkout=$fix/repo-$state
    run launch "$CASE/root"
    checkout=$fix/repo
    check "launch: a checkout that is $state exits 6" code_is 6
    check "launch: a checkout that is $state invokes nothing" no_calls
    check "launch: a checkout that is $state makes no try root" [ ! -e "$CASE/root/docs-try2" ]
done
check "launch: and the last one names origin/main" err_has "is not on origin/main"
new_case
run launch "$CASE/root"
await_calls 1
check "launch: the driver runs from the checkout, pinned at its HEAD" sh -c "grep -qx 'env .* pwd=$fix/repo' '$CASE/calls' && grep -q -- '--pin $(git -C "$fix/repo" rev-parse HEAD) ' '$CASE/calls'"

# ---------------------------------------------------------------- relaunch

new_case
run launch "$CASE/root"
await_calls 1
run relaunch "$CASE/root"
await_calls 2
check "relaunch: exits 0" code_is 0
check "relaunch: invokes the driver again with the same arguments" sh -c "[ \"\$(grep -c '^argv ' '$CASE/calls')\" = 2 ] && [ \"\$(grep '^argv ' '$CASE/calls' | sort -u | wc -l)\" = 1 ]"
mkdir -p "$CASE/root/docs-try2"
for marker in done capped canary-failed stop; do
    : > "$CASE/root/docs-try2/$marker"
    run relaunch "$CASE/root"
    check "relaunch: $marker present exits 6" code_is 6
    rm "$CASE/root/docs-try2/$marker"
done
check "relaunch: no refused relaunch invokes the driver" [ "$(calls_count)" = 2 ]
check "relaunch: no relaunch writes a.done" [ ! -e "$CASE/root/docs-try2/a.done" ]
run relaunch "$CASE/root" docs-try9
check "relaunch: a try that does not exist exits 6" code_is 6
new_case
checkout=$fix/repo-moves
run launch "$CASE/root"
await_calls 1
git -C "$fix/repo-moves" -c user.name=fixture -c user.email=fixture@example.org commit -q --allow-empty -m moved
run relaunch "$CASE/root"
checkout=$fix/repo
check "relaunch: a checkout whose HEAD left the pin of driver.argv exits 6" code_is 6
check "relaunch: and names the pin" err_has "not the pin"
check "relaunch: and invokes the driver no second time" [ "$(calls_count)" = 1 ]
new_case
: > "$CASE/driver-sleeps"
run launch "$CASE/root"
await_calls 1
run relaunch "$CASE/root"
check "relaunch: a driver that runs exits 6" code_is 6
run wait "$CASE/root"
check "wait: a driver that runs and needs nobody exits 1" code_is 1
kill "$(cat "$CASE/root/docs-try2/driver.pid")" 2>/dev/null
check "relaunch: a running driver is not invoked again" [ "$(calls_count)" = 1 ]

# ------------------------------------------------------------------ canary

# record <job> [<sed expression>]: a session of the canary under docs-try2,
# with the sentences of a clean record.
record() {
    s=$CASE/root/docs-try2/a/sessions/$1
    mkdir -p "$s"
    echo "$1 1 additive scenario-1 tuning HW-PROBE-a" >> "$CASE/root/docs-try2/a/jobs"
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
    mkdir -p "$CASE/root/docs-try2/a" "$CASE/root/hook-try1/a"
    echo "additive scenario-1 tuning" > "$CASE/root/docs-try2/spec"
    : > "$CASE/root/docs-try2/canary-recorded"
    for j in 1 2 3 4 5 6 7 8; do record "L1-p1-r$j"; done
    # A job of the jobs list that has not started is not a record.
    echo "L1-p2-r1 1 additive scenario-1 tuning HW-PROBE-b" >> "$CASE/root/docs-try2/a/jobs"
}
rec() { echo "$CASE/root/docs-try2/a/sessions/$1"; }

canary_case
run canary "$CASE/root"
check "canary: eight clean records write canary-passed" [ -f "$CASE/root/docs-try2/canary-passed" ]
check "canary: and exit 0" code_is 0
check "canary: and write no canary-needs-reader" [ ! -e "$CASE/root/docs-try2/canary-needs-reader" ]

canary_case
sed -i 's/named 0 paths outside/named 1 path outside/' "$(rec L1-p1-r3)/record.md"
run canary "$CASE/root"
check "canary: one record naming 1 path outside exits 7" code_is 7
check "canary: and writes no canary-passed" [ ! -e "$CASE/root/docs-try2/canary-passed" ]
check "canary: canary-needs-reader names that session and its sentence" grep -qx "L1-p1-r3: outside: The session named 1 path outside its workspace" "$CASE/root/docs-try2/canary-needs-reader"
check "canary: and no other session" [ "$(wc -l < "$CASE/root/docs-try2/canary-needs-reader")" = 1 ]

canary_case
sed -i 's/made 0 calls to a web tool/made 1 call to a web tool/' "$(rec L1-p1-r5)/record.md"
run canary "$CASE/root"
check "canary: one record with 1 call to a web tool exits 7" code_is 7
check "canary: and names it" grep -qx "L1-p1-r5: web: The session made 1 call to a web tool" "$CASE/root/docs-try2/canary-needs-reader"

canary_case
sed -i 's/made 0 calls to a web tool/made 2 calls to a web tool/; s/named 0 paths outside/named 3 paths outside/' "$(rec L1-p1-r6)/record.md"
run canary "$CASE/root"
check "canary: the plural of each is offending too" sh -c "grep -q 'L1-p1-r6: web: The session made 2 calls' '$CASE/root/docs-try2/canary-needs-reader' && grep -q 'L1-p1-r6: outside: The session named 3 paths' '$CASE/root/docs-try2/canary-needs-reader'"

canary_case
sed -i 's/^The session made 0 calls to a web tool\./The calls this session made to a web tool were not counted, because its stream did not parse as JSON./; s/^The session named 0 paths outside.*/The paths outside its workspace this session named were not counted, because its stream did not parse as JSON./' "$(rec L1-p1-r2)/record.md"
run canary "$CASE/root"
check "canary: a not-counted sentence is offending and never a pass" code_is 7
check "canary: and names both sentences" sh -c "grep -q 'L1-p1-r2: web: The calls this session made to a web tool were not counted' '$CASE/root/docs-try2/canary-needs-reader' && grep -q 'L1-p1-r2: outside: The paths outside its workspace this session named were not counted' '$CASE/root/docs-try2/canary-needs-reader'"

canary_case
sed -i 's/It refused no request of that kind/It refused 1 request of that kind/' "$(rec L1-p1-r1)/record.md"
run canary "$CASE/root"
check "canary: a refused provider fetch exits 7" code_is 7
check "canary: and names it" grep -q "L1-p1-r1: provider: It refused 1 request of that kind" "$CASE/root/docs-try2/canary-needs-reader"

canary_case
sed -i 's/The intent hook was not live in this session\./The intent hook was live in this session./' "$(rec L1-p1-r4)/record.md"
run canary "$CASE/root"
check "canary: a live hook in a scenario without the hook layer exits 7" code_is 7
check "canary: and names the hook state" grep -q "L1-p1-r4: hook: the scenario declares it not live" "$CASE/root/docs-try2/canary-needs-reader"

canary_case
echo "additive scenario-2 tuning" > "$CASE/root/docs-try2/spec"
run canary "$CASE/root"
check "canary: a hook that was not live in a scenario that holds the hook layer exits 7" code_is 7
check "canary: on every record" [ "$(grep -c ': hook: the scenario declares it live' "$CASE/root/docs-try2/canary-needs-reader")" = 8 ]
canary_case
echo "additive scenario-2 tuning" > "$CASE/root/docs-try2/spec"
for j in 1 2 3 4 5 6 7 8; do
    sed -i 's/The intent hook was not live in this session\./The intent hook was live in this session./' "$(rec "L1-p1-r$j")/record.md"
done
run canary "$CASE/root"
check "canary: a live hook in every record of a hook scenario passes" code_is 0

canary_case
echo 400 > "$(rec L1-p1-r7)/cost"
run canary "$CASE/root"
check "canary: a cost of exactly 300 + 100 passes" code_is 0
canary_case
echo 401 > "$(rec L1-p1-r7)/cost"
run canary "$CASE/root"
check "canary: a cost of 401 exits 7" code_is 7
check "canary: and names it" grep -qx "L1-p1-r7: cost 401 above 400" "$CASE/root/docs-try2/canary-needs-reader"

canary_case
echo 5 > "$(rec L1-p1-r8)/status"
run canary "$CASE/root"
check "canary: a status of 5 exits 7" code_is 7
check "canary: and names it" grep -qx "L1-p1-r8: status 5" "$CASE/root/docs-try2/canary-needs-reader"

canary_case
rm -r "$(rec L1-p1-r8)"
run canary "$CASE/root"
check "canary: seven records exit 7" code_is 7
check "canary: and say so" grep -qx "7 records of the canary, not 8" "$CASE/root/docs-try2/canary-needs-reader"

canary_case
rm "$CASE/root/docs-try2/canary-recorded"
run canary "$CASE/root"
check "canary: before canary-recorded exits 6" code_is 6
check "canary: and writes nothing" sh -c "[ ! -e '$CASE/root/docs-try2/canary-passed' ] && [ ! -e '$CASE/root/docs-try2/canary-needs-reader' ]"

canary_case
sed -i 's/named 0 paths outside/named 1 path outside/' "$(rec L1-p1-r3)/record.md"
: > "$CASE/root/docs-try2/canary-passed"
run canary "$CASE/root"
check "canary: a canary-passed a person wrote stands" code_is 0
check "canary: and is not overwritten" [ ! -s "$CASE/root/docs-try2/canary-passed" ]

# ------------------------------------------------------ result: the keep rule

# screening <file> <tier> <round> <arm>: a committed transcript of the case's
# checkout, with the front matter `result` selects on.
screening() {
    mkdir -p "$checkout/docs/probe-runs"
    printf -- '---\nid: HW-RUN-%s\ntier: %s\nround: %s\narm: %s\ntitle: "x"\n---\n\n# x\n' \
        "$1" "$2" "$3" "$4" > "$checkout/docs/probe-runs/$1.md"
}
# finished_try <try> [<all>]: a try root whose driver wrote `done`, two
# sessions of 1200 and 1252 cents, and a drive.out whose last `all=` line is
# <all>, 2452 when it is not given.
finished_try() {
    t=$CASE/root/$1
    mkdir -p "$t/a"
    echo "additive scenario-1 tuning" > "$t/spec"
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
    finished_try docs-try2
    echo 'pooled: 11/72 against 11/72, +0.0 points [-12.8, +12.8]' > "$CASE/pooled-base"
}
ledger_has() { grep -qF -- "$1" "$CASE/root/ledger.jsonl"; }
result_md_has() { grep -qxF -- "$1" "$CASE/root/docs-try2/result.md"; }

# Each row is the pooled keep row that `headwater probe compare` prints for
# that try against a best of 39 of 72, computed with the Newcombe interval of
# grade.rs, then the verdict and the direction. The rule is the owner's of
# 2026-10-20 on #1777: kept only when the whole interval sits above zero. The
# rule the issue first wrote, "strictly above the best so far", keeps 40.
while IFS='|' read -r row_try row_line row_keep row_direction; do
    result_case
    echo "$row_line" > "$CASE/pooled-keep"
    run result "$CASE/root"
    check "result $row_try: exits 0, kept or not" code_is 0
    check "result $row_try: prints keep: $row_keep" out_has "keep: $row_keep"
    check "result $row_try: prints direction: $row_direction" out_has "direction: $row_direction"
    check "result $row_try: result.md holds keep: $row_keep" result_md_has "keep: $row_keep"
    check "result $row_try: result.md holds the keep row verbatim" grep -qF -- "$row_line" "$CASE/root/docs-try2/result.md"
    if [ "$row_keep" = kept ]; then row_kept=true; else row_kept=false; fi
    check "result $row_try: the ledger records kept $row_kept" ledger_has "\"kept\": $row_kept"
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
run result "$CASE/root"
check "result: the keep row compares the new transcript against the scenario-1 screening transcript" grep -q 'probe compare .*docs/probe-runs/additive-tuning-of-.*-docs-try-2-scenario-1\.md .*/docs/probe-runs/additive-screening-scenario-1\.md$' "$CASE/engine.log"
check "result: the base row compares it against the scenario-0 screening transcript" grep -q 'probe compare .*docs/probe-runs/additive-tuning-of-.*-docs-try-2-scenario-1\.md .*/docs/probe-runs/additive-screening-scenario-0\.md$' "$CASE/engine.log"
check "result: result.md holds the base row verbatim" result_md_has "base row: pooled: 11/72 against 11/72, +0.0 points [-12.8, +12.8]"
check "result: result.md holds the spend" result_md_has "spent: 2452"
check "result: result.md states the rule and cites it" grep -qF 'owner, 2026-10-20, #1777' "$CASE/root/docs-try2/result.md"
check "result: the transcript is written with the facets tier, arm and round" grep -q '^new probe_transcript .*--facet tier=additive --facet arm=scenario-1 --facet round=tuning' "$CASE/engine.log"
result_transcript=$(ls "$checkout"/docs/probe-runs/additive-tuning-of-*-docs-try-2-scenario-1.md 2>/dev/null)
check "result: the transcript is in the checkout" [ -f "$result_transcript" ]
check "result: its Run identity holds the assembled identity block" grep -qx 'cost_cents: 2452' "$result_transcript"
check "result: its Events hold the assembled events block" grep -qx '  session: "L1-additive-scenario-1-p1-r1"' "$result_transcript"
check "result: no TODO is left in the two sections" sh -c "! grep -q TODO '$result_transcript'"
check "result: the ledger records the result event" ledger_has '"event": "result"'
check "result: with its layer and scenario" sh -c "grep -qF '\"layer\": \"docs\"' '$CASE/root/ledger.jsonl' && grep -qF '\"scenario\": \"1\"' '$CASE/root/ledger.jsonl'"
check "result: and its direction" ledger_has '"direction": "up"'
check "result: and the spend" ledger_has '"spent": 2452'
run result "$CASE/root"
check "result: a try that has a result.md exits 6" code_is 6
check "result: and appends no second event" [ "$(grep -c '"event": "result"' "$CASE/root/ledger.jsonl")" = 1 ]

# The best so far is the latest kept try of the layer, and a try that is not
# kept never becomes it.
finished_try docs-try3
echo 'pooled: 40/72 against 51/72, -15.3 points [-30.1, +0.6]' > "$CASE/pooled-keep"
run result "$CASE/root" docs-try3
check "result: try 3 exits 0" code_is 0
check "result: its keep row is against try 2, which was kept" grep -q 'probe compare .*-docs-try-3-scenario-1\.md .*-docs-try-2-scenario-1\.md$' "$CASE/engine.log"
finished_try docs-try4
run result "$CASE/root" docs-try4
check "result: try 4's keep row is against try 2 still, because try 3 was not kept" grep -q 'probe compare .*-docs-try-4-scenario-1\.md .*-docs-try-2-scenario-1\.md$' "$CASE/engine.log"

# A kept try of another layer is not the best so far of this one.
result_case
printf '%s\n' '{"event": "result", "try": "hook-try1", "layer": "hook", "scenario": "2", "transcript": "docs/probe-runs/additive-screening-scenario-10.md", "kept": true}' > "$CASE/root/ledger.jsonl"
echo 'pooled: 51/72 against 39/72, +16.7 points [+0.9, +31.4]' > "$CASE/pooled-keep"
run result "$CASE/root"
check "result: a kept try of the hook layer is not the best so far of docs" grep -q 'probe compare .*-docs-try-2-scenario-1\.md .*/additive-screening-scenario-1\.md$' "$CASE/engine.log"

# The spend: the session files against the driver's last `all=` line.
result_case
rm -r "$CASE/root/docs-try2"
finished_try docs-try2 2451
echo 'pooled: 51/72 against 39/72, +16.7 points [+0.9, +31.4]' > "$CASE/pooled-keep"
run result "$CASE/root"
check "result: session files of 2452 against all=2451 exit 2" code_is 2
check "result: and name both figures" sh -c "grep -q 2452 '$CASE/err' && grep -q 2451 '$CASE/err'"
check "result: and write no result.md" [ ! -e "$CASE/root/docs-try2/result.md" ]
check "result: and assemble nothing" sh -c "! grep -q assemble '$CASE/trees.log' 2>/dev/null"
result_case
sed -i '/all=/d' "$CASE/root/docs-try2/drive.out"
run result "$CASE/root"
check "result: a drive.out with no all= line exits 2" code_is 2

# The keep row has no interval when a side graded no session.
result_case
echo 'pooled: 0/0 against 39/72, no difference: the treated transcript graded no session' > "$CASE/pooled-keep"
run result "$CASE/root"
check "result: a keep row with no interval exits 2" code_is 2
check "result: and says the keep row has no interval" err_has "the keep row has no interval"
check "result: and writes no result.md" [ ! -e "$CASE/root/docs-try2/result.md" ]
check "result: and records no result event" sh -c "! grep -q '\"event\": \"result\"' '$CASE/root/ledger.jsonl' 2>/dev/null"

# The best so far must be one transcript.
result_case
screening additive-screening-scenario-1-again additive screening scenario-1
echo 'pooled: 51/72 against 39/72, +16.7 points [+0.9, +31.4]' > "$CASE/pooled-keep"
run result "$CASE/root"
check "result: two screening transcripts of the open scenario exit 2" code_is 2
check "result: and name both" sh -c "grep -q additive-screening-scenario-1.md '$CASE/err' && grep -q additive-screening-scenario-1-again.md '$CASE/err'"
result_case
rm "$checkout/docs/probe-runs/additive-screening-scenario-1.md"
run result "$CASE/root"
check "result: no screening transcript of the open scenario exits 2" code_is 2

result_case
: > "$CASE/assemble-fails"
run result "$CASE/root"
check "result: an assembly that refuses exits 2" code_is 2
check "result: and writes no transcript" sh -c "! grep -q '^new ' '$CASE/engine.log' 2>/dev/null"

# A try with no `done`: a halt, and no verdict.
result_case
rm "$CASE/root/docs-try2/done"
: > "$CASE/root/docs-try2/capped"
echo 'pooled: 51/72 against 39/72, +16.7 points [+0.9, +31.4]' > "$CASE/pooled-keep"
run result "$CASE/root"
check "result: a try with no done exits 7" code_is 7
check "result: and appends a halt event" ledger_has '"event": "halt"'
check "result: with the spend" ledger_has '"spent": 2452'
check "result: and the markers" ledger_has '"markers": "capped"'
check "result: and prints no keep: line" sh -c "! grep -q '^keep:' '$CASE/out'"
check "result: and writes no result.md" [ ! -e "$CASE/root/docs-try2/result.md" ]
check "result: and writes no transcript" sh -c "! grep -q '^new ' '$CASE/engine.log' 2>/dev/null"
checkout=$fix/repo

# ------------------------------------------------------------------ ledger

result_case
run ledger "$CASE/root"
check "ledger: no ledger.jsonl prints no events" out_has "no events"
check "ledger: and exits 0" code_is 0
echo 'pooled: 51/72 against 39/72, +16.7 points [+0.9, +31.4]' > "$CASE/pooled-keep"
run result "$CASE/root"
finished_try docs-try3
rm "$CASE/root/docs-try3/done"
run result "$CASE/root" docs-try3
run ledger "$CASE/root"
check "ledger: two events print two lines and the total" [ "$(grep -c '^[a-z]*: ' "$CASE/out")" = 3 ]
check "ledger: the first is the result" out_has "result: try=docs-try2"
check "ledger: the second is the halt" out_has "halt: try=docs-try3"
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
mkdir -p "$CASE/root/docs-try2"
for marker in done canary-failed capped stop; do
    : > "$CASE/root/docs-try2/$marker"
    run wait "$CASE/root"
    check "wait: $marker exits 0" code_is 0
    check "wait: and prints $marker" sh -c "[ \"\$(cat '$CASE/out')\" = '$marker' ]"
    rm "$CASE/root/docs-try2/$marker"
done
: > "$CASE/root/docs-try2/canary-recorded"
run wait "$CASE/root"
check "wait: canary-recorded without canary-passed exits 0" code_is 0
check "wait: and prints canary-recorded" out_has canary-recorded
: > "$CASE/root/docs-try2/canary-needs-reader"
run wait "$CASE/root"
check "wait: canary-needs-reader is printed before canary-recorded" sh -c "[ \"\$(cat '$CASE/out')\" = canary-needs-reader ]"
: > "$CASE/root/docs-try2/canary-passed"
run wait "$CASE/root"
check "wait: a canary that passed and a driver that is gone print driver-exited" sh -c "[ \"\$(cat '$CASE/out')\" = driver-exited ]"
echo 'TRY_FLOOR_TRY_KB=999999999999' >> "$CASE/root/env"
run wait "$CASE/root"
check "wait: below the disk floor exits 0" code_is 0
check "wait: and prints disk-floor" out_has disk-floor
check "wait: and writes stop" [ -s "$CASE/root/docs-try2/stop" ]
new_case
decl none 10000
run wait "$CASE/root"
check "wait: with no open layer and no try named exits 2" code_is 2
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

# rank_decl <open>: a declaration with the skills layer and the six fields of
# the rounds, at one cent a session, so a cap of 3000 holds a confirmation.
rank_decl() {
    cat > "$CASE/probe.yml" <<EOF
layers:
  skills: {paths: [.claude/skills/hw-x]}
  docs: {}
  hook: {}
sets:
  tuning: [HW-PROBE-a, HW-PROBE-b, HW-PROBE-c, HW-PROBE-d, HW-PROBE-e]
scenarios:
  0: {layers: []}
  1: {base: 0, layers: [skills]}
  2: {base: 0, layers: [docs]}
  3: {base: 0, layers: [hook]}
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
    order: [skills, docs, hook]
    ceiling_cents: ${2:-100000}
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
    rank_decl "$OPEN_SKILLS" "${1:-100000}"
    checkout=$CASE/checkout
    screening additive-screening-scenario-0 additive screening scenario-0
    screening additive-screening-scenario-1 additive screening scenario-1
    # The best so far, at the confirmation's count: 3 of 145 call the skill.
    skill_batch skills-r1-best best 145 3
    echo 'pooled: 80/145 against 78/145, +1.4 points [-10.0, +12.8]' > "$CASE/pooled-base"
    run result "$CASE/root" skills-r1-best
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
screened 1 2 25
run launch "$CASE/root" confirm skills-r1-v1
check "rank: a top variant with no gain in use exits 6" code_is 6
check "rank: and closes the round with nothing kept" ledger_has '"reason": "no-gain-in-use"'
run plan "$CASE/root" screen 2
check "rank: a closed round screens no more variants" code_is 6
check "rank: and says it is closed" err_has "round 1 of the skills layer is closed"
checkout=$fix/repo

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
