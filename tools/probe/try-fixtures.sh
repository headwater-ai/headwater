#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
#
# The fixtures of tools/probe/try.sh, the seven verbs of one try of the
# additive campaign's tuning round (#1821).
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
echo "env versions=${HEADWATER_DRIVE_CLAUDE_VERSIONS:-} target=${CARGO_TARGET_DIR:-} tmp=${TMPDIR:-} pwd=$PWD" >> "$TRY_CALLS"
if [ -f "$TRY_SLEEP" ]; then sleep 30; fi
exit 0
EOF
cat > "$fix/campaign-stub.sh" <<'EOF'
#!/bin/sh
n= out= at=
while [ $# -gt 0 ]; do
    case $1 in
        --scenario-tree) n=$2 ;;
        --out) out=$2 ;;
        --at) at=$2 ;;
        *) echo "stub: unknown $1" >&2; exit 2 ;;
    esac
    shift 2
done
echo "tree $n $at" >> "$CASE/trees.log"
[ -f "$CASE/tree-fails" ] && { echo "stub: the tree did not build" >&2; exit 3; }
if [ "$at" = HEAD ]; then src=$CASE/tree-head; else src=$CASE/tree-pin; fi
mkdir -p "$out/trees"
cp -a "$src" "$out/trees/additive-scenario-$n"
EOF

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

# The layout of the decisive case: two earlier try roots, one dot-named.
two_roots() {
    cost "$CASE/root/docs-try1/a/sessions/s1" 3000
    cost "$CASE/root/docs-try1/a/sessions/s2" 1000
    cost "$CASE/root/.docs-try1-moved/a/sessions/s1" 2000 "$1"
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
check "spent: the sum of both try roots, the dot-named one and its spent-before included, is 7001" out_has 7001
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
check "status: one line for the first try, with what it spent and its markers" out_has "docs-try1 spent=4000 driver=none markers=canary-passed done"
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

# ---------------------------------------------------------------- arguments

new_case
run
check "args: no verb exits 2" code_is 2
run frobnicate "$CASE/root"
check "args: an unknown verb exits 2" code_is 2
run spent "$CASE/root" extra
check "args: spent with a try exits 2" code_is 2

# TRY_FIXTURES_KEEP keeps the cases' directories, to read a failure.
if [ -n "${TRY_FIXTURES_KEEP:-}" ]; then echo "try-fixtures: the cases are in $fix" >&2; else rm -rf "$fix"; fi
echo "try-fixtures: $passed passed, $failed failed"
[ "$failed" = 0 ]
