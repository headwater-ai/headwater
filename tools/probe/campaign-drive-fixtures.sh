#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
#
# The fixtures of tools/probe/campaign-drive.sh, the detached driver loop of a
# staggered probe campaign (#1659).
#
#     sh tools/probe/campaign-drive-fixtures.sh
#
# No case spends, calls a model or reaches the network. Each case runs the
# driver against a stub `campaign.sh` that writes the layout the real one
# leaves (`jobs`, `sessions/<name>/{status,cost,spent-before,started}`,
# `slice/{halt,started/<name>,refused/<name>}` and `claude-version`), a stub
# clock whose sleep adds to a file and never waits, a directory of fake
# `claude` versions, a credentials file the case writes, and a throwaway git
# repository detached at the pin. A `claude` on `PATH` and every fake version
# log any call other than `--version`, and the last case asserts that log is
# empty. Each case runs under `timeout 60`, so a driver that loops shows as a
# failure and not as a hang.
#
# The decisive case is the first: two batches together never spend past
# `--cap-cents`, because the cap each invocation is given is the cap less
# what the other batch spent less `--parallel` times the overshoot margin.
#
# Exit status: 0 when every case passes, 1 when one fails, 3 when `jq`, `git`
# or `timeout` is missing.

set -u

for tool in jq git awk timeout; do
    command -v "$tool" >/dev/null 2>&1 || { echo "campaign-drive-fixtures: $tool is missing, so no case runs." >&2; exit 3; }
done

here=$(cd "$(dirname "$0")" && pwd)
driver=${HEADWATER_DRIVE_UNDER_TEST:-$here/campaign-drive.sh}
fix=$(mktemp -d "${TMPDIR:-/tmp}/campaign-drive-fixtures.XXXXXX")
passed=0
failed=0

ok() { passed=$((passed + 1)); }
bad() { failed=$((failed + 1)); echo "FAIL: $1" >&2; }
check() {
    # check <description> <command...>
    desc=$1
    shift
    if "$@"; then ok; else bad "$desc"; fi
}

# ---------------------------------------------------------------- the stubs

mkdir -p "$fix/versions" "$fix/decoy"
for v in 2.1.287 2.1.288; do
    cat > "$fix/versions/$v" <<EOF
#!/bin/sh
if [ "\${1:-}" = --version ]; then echo "$v (Claude Code)"; exit 0; fi
echo "versions/$v \$*" >> "$fix/paid.log"
exit 1
EOF
    chmod +x "$fix/versions/$v"
done
cat > "$fix/decoy/claude" <<EOF
#!/bin/sh
if [ "\${1:-}" = --version ]; then echo "9.9.9 (Claude Code)"; exit 0; fi
echo "decoy \$*" >> "$fix/paid.log"
exit 1
EOF
chmod +x "$fix/decoy/claude"

# The clock: `now` prints the epoch seconds in $CASE/clock, and `sleep` adds
# to it, logs the step, and runs the one-shot hook $CASE/on-sleep.
cat > "$fix/now.sh" <<'EOF'
cat "$CASE/clock"
EOF
cat > "$fix/sleep.sh" <<'EOF'
t=$(cat "$CASE/clock")
echo $((t + $1)) > "$CASE/clock"
echo "sleep $1" >> "$CASE/sleeps"
echo "sleep $1" >> "$CASE/events"
if [ -f "$CASE/on-sleep" ]; then
    mv "$CASE/on-sleep" "$CASE/on-sleep.ran"
    sh "$CASE/on-sleep.ran"
fi
EOF

# The campaign. Each invocation reads the next line of $CASE/scenario (the
# last line repeats once they run out), which is either
#     exit=<n> [record=<k>] [cents=<c>] [halt=y] [skip=<k>] [refuse=<k>] [fail=<k>]
# or
#     auto cents=<c>
# A manual line walks the jobs not yet recorded, in order: it leaves `skip`
# of them unstarted, records `record` at `cents` each, halts the next one
# when `halt=y`, refuses the next `refuse`, fails the next `fail`, and exits
# with `exit`. An `auto` line honors the cap it was given as campaign.sh
# does at --parallel 1: it records a job at `cents` while spent plus the
# session budget is at most the cap and the bound allows, refuses every job
# after that, and exits 7 on a refusal, 9 on the bound and 0 when all are
# recorded.
cat > "$fix/campaign-stub.sh" <<'EOF'
set -u
printf 'argv %s\n' "$*" >> "$CASE/calls.argv"
out= spec= cap= max= budget=
while [ $# -gt 0 ]; do
    case $1 in
        --out) out=$2; shift 2 ;;
        --spec) spec=$2; shift 2 ;;
        --cap-cents) cap=$2; shift 2 ;;
        --max-sessions) max=$2; shift 2 ;;
        --session-cents) budget=$2; shift 2 ;;
        *) shift ;;
    esac
done
ver=$(claude --version 2>/dev/null | head -1)
echo "call $out" >> "$CASE/events"
n=$(cat "$CASE/scenario.n" 2>/dev/null || echo 0)
n=$((n + 1))
echo "$n" > "$CASE/scenario.n"
lines=$(wc -l < "$CASE/scenario")
[ "$n" -gt "$lines" ] && n=$lines
line=$(sed -n "${n}p" "$CASE/scenario")
mkdir -p "$out/sessions"
if [ ! -f "$out/jobs" ]; then
    jobs=$(sed -n 's/^jobs //p' "$spec")
    i=1
    while [ "$i" -le "$jobs" ]; do
        printf '%s-%03d 1 campaign present probe-x\n' "$(basename "$out")" "$i" >> "$out/jobs"
        i=$((i + 1))
    done
fi
[ -f "$out/claude-version" ] || printf '%s\n' "$ver" > "$out/claude-version"
rm -rf "$out/slice"
mkdir -p "$out/slice/started" "$out/slice/refused"
code= record=0 cents=0 halt=n skip=0 refuse=0 fail=0 auto=n
for word in $line; do
    case $word in
        auto) auto=y ;;
        exit=*) code=${word#exit=} ;;
        record=*) record=${word#record=} ;;
        cents=*) cents=${word#cents=} ;;
        halt=*) halt=${word#halt=} ;;
        skip=*) skip=${word#skip=} ;;
        refuse=*) refuse=${word#refuse=} ;;
        fail=*) fail=${word#fail=} ;;
    esac
done
printf 'argv %s cap=%s max=%s\n' "$out" "$cap" "$max" >> "$CASE/calls.short"
spent() {
    find "$out/sessions" -mindepth 2 -maxdepth 2 -type f \( -name cost -o -name spent-before \) -exec cat {} + 2>/dev/null \
        | awk '{ s += $1 } END { print s + 0 }'
}
start() {
    d=$out/sessions/$1
    mkdir -p "$d"
    : > "$out/slice/started/$1"
    if [ -f "$d/cost" ]; then
        b=$(cat "$d/cost")
        [ -f "$d/spent-before" ] && b=$((b + $(cat "$d/spent-before")))
        echo "$b" > "$d/spent-before"
    fi
    rm -f "$d/status" "$d/cost"
    : > "$d/started"
}
refused=0 left=0 started=0
while read -r name rest; do
    [ "$(cat "$out/sessions/$name/status" 2>/dev/null)" = 0 ] && continue
    if [ "$auto" = y ]; then
        if [ "$started" -ge "$max" ]; then
            left=1
        elif [ $(($(spent) + budget)) -gt "$cap" ]; then
            : > "$out/slice/refused/$name"
            refused=1
        else
            start "$name"
            echo 0 > "$out/sessions/$name/status"
            echo "$cents" > "$out/sessions/$name/cost"
            started=$((started + 1))
        fi
        continue
    fi
    if [ "$skip" -gt 0 ]; then
        skip=$((skip - 1))
    elif [ "$record" -gt 0 ]; then
        start "$name"
        echo 0 > "$out/sessions/$name/status"
        echo "$cents" > "$out/sessions/$name/cost"
        record=$((record - 1))
    elif [ "$halt" = y ]; then
        start "$name"
        echo 10 > "$out/sessions/$name/status"
        : > "$out/slice/halt"
        halt=n
    elif [ "$refuse" -gt 0 ]; then
        : > "$out/slice/refused/$name"
        refuse=$((refuse - 1))
    elif [ "$fail" -gt 0 ]; then
        start "$name"
        echo 2 > "$out/sessions/$name/status"
        echo "$cents" > "$out/sessions/$name/cost"
        fail=$((fail - 1))
    fi
done < "$out/jobs"
if [ "$auto" = y ]; then
    [ "$refused" = 1 ] && exit 7
    [ "$left" = 1 ] && exit 9
    exit 0
fi
exit "$code"
EOF

# ---------------------------------------------------------------- a case

# new_case: a fresh case directory with a repository detached at the pin,
# a clock at a fixed time, credentials that outlive it by 8 h, and two
# specs of 20 and 10 jobs. Sets CASE, PIN and the driver's defaults.
new_case() {
    CASE=$(mktemp -d "$fix/case.XXXXXX")
    export CASE
    echo 1790000000 > "$CASE/clock"
    : > "$CASE/events"
    mkdir -p "$CASE/host" "$CASE/repo" "$CASE/root"
    write_token $((1790000000 + 8 * 3600))
    echo "jobs 20" > "$CASE/spec-a"
    echo "jobs 10" > "$CASE/spec-b"
    g() { git -C "$CASE/repo" -c user.name=fixture -c user.email=fixture@example.invalid -c core.hooksPath=/dev/null "$@"; }
    g init -q -b main
    echo one > "$CASE/repo/file"
    g add file
    g commit -q -m one
    g update-ref refs/remotes/origin/main HEAD
    g checkout -q --detach
    PIN=$(g rev-parse HEAD)
    B=60 C=100000 M=20 P=2 VERSION=2.1.288
}

write_token() {
    printf '{"claudeAiOauth":{"expiresAt":%s000,"subscriptionType":"max"}}\n' "$1" > "$CASE/host/.credentials.json"
}

# Skip the canary and batch B, so a case reads batch A alone.
only_a() {
    : > "$CASE/root/canary-recorded"
    : > "$CASE/root/canary-passed"
    : > "$CASE/root/b.done"
}

drive() {
    (
        cd "$CASE/repo" || exit 99
        HEADWATER_DRIVE_CAMPAIGN=$fix/campaign-stub.sh \
        HEADWATER_DRIVE_NOW="sh $fix/now.sh" \
        HEADWATER_DRIVE_SLEEP="sh $fix/sleep.sh" \
        HEADWATER_DRIVE_CLAUDE_VERSIONS=$fix/versions \
        HEADWATER_PROBE_HOST_CONFIG=$CASE/host \
        PATH="$fix/decoy:$PATH" \
            timeout 60 sh "$driver" --root "$CASE/root" --pin "$PIN" --claude-version "$VERSION" \
            --spec-a "$CASE/spec-a" --spec-b "$CASE/spec-b" --repetitions-b 118 --model fixture-model \
            --session-cents "$B" --cap-cents "$C" --parallel "$P" --overshoot-cents "$M" "$@"
    ) > "$CASE/out" 2> "$CASE/err"
    echo $? > "$CASE/code"
}

code_is() { [ "$(cat "$CASE/code")" = "$1" ]; }
calls() {
    if [ -f "$CASE/calls.short" ]; then awk '/^argv / { n++ } END { print n + 0 }' "$CASE/calls.short"; else echo 0; fi
}
calls_are() { [ "$(calls)" = "$1" ]; }
has() { [ -f "$CASE/root/$1" ]; }
hasnt() { [ ! -f "$CASE/root/$1" ]; }
out_has() { grep -q -- "$1" "$CASE/out"; }
spent_of() {
    find "$CASE/root/$1/sessions" -mindepth 2 -maxdepth 2 -type f \( -name cost -o -name spent-before \) -exec cat {} + 2>/dev/null \
        | awk '{ s += $1 } END { print s + 0 }'
}

# ------------------------------------------- the decisive case: the shared cap

new_case
C=1000 B=60 M=20 P=2
printf 'jobs 10\n' > "$CASE/spec-a"
printf 'jobs 30\n' > "$CASE/spec-b"
: > "$CASE/root/canary-passed"
printf 'auto cents=70\nauto cents=90\nauto cents=50\n' > "$CASE/scenario"
drive
a=$(spent_of a)
b=$(spent_of b)
check "cap: the driver exits 8 when both batches are capped" code_is 8
check "cap: <root>/capped is written" has capped
check "cap: no <root>/done is written" hasnt done
check "cap: spent(a) + spent(b) is at most --cap-cents ($a + $b > $C)" [ $((a + b)) -le "$C" ]
check "cap: the canary is given C - spent(b) - 1 x M" grep -q "/root/a cap=980 max=8\$" "$CASE/calls.short"
check "cap: batch A is given C - spent(b) - 2 x M" grep -q "/root/a cap=960 max=500\$" "$CASE/calls.short"
bcaps=$(grep '/root/b ' "$CASE/calls.short" | sed 's/.* cap=\([0-9-]*\) .*/\1/' | sort -u)
check "cap: every B invocation is given C - spent(a) - 2 x M, $((C - a - 2 * M)) (saw: $bcaps)" [ "$bcaps" = "$((C - a - 2 * M))" ]
check "cap: B is invoked twice, the refusal and one re-try" [ "$(grep -c '/root/b ' "$CASE/calls.short")" = 2 ]
check "cap: the capped file states both spends and C" grep -q "^cap-cents: $C\$" "$CASE/root/capped"

# ------------------------------------------------- 0. arguments

new_case
drive --parallel 0
check "args: a --parallel of 0 exits 2" code_is 2
new_case
drive --cap-cents ten
check "args: a --cap-cents that is not a number exits 2" code_is 2
new_case
(
    cd "$CASE/repo" && timeout 60 sh "$driver" --root "$CASE/root" --pin "$PIN"
) > "$CASE/out" 2> "$CASE/err"
echo $? > "$CASE/code"
check "args: a missing argument exits 2" code_is 2
check "args: no usage error calls the campaign" calls_are 0

# ------------------------------------------------- 1. order and done

new_case
: > "$CASE/root/canary-passed"
cat > "$CASE/scenario" <<'EOF'
exit=9 record=8 cents=50
exit=9 record=4 cents=50
exit=9 record=4 cents=50
exit=0 record=4 cents=50
exit=9 record=5 cents=40
exit=0 record=5 cents=40
EOF
drive
check "order: exits 0" code_is 0
check "order: six invocations" calls_are 6
check "order: the canary is first, at --parallel 1 --max-sessions 8" sh -c "grep '^argv' '$CASE/calls.short' | head -1 | grep -q '/root/a cap=99980 max=8\$'"
check "order: the canary argv carries --parallel 1" sh -c "grep -c -- '--parallel 1 --max-sessions 8' '$CASE/calls.argv' | grep -qx 1"
check "order: A and B run at --parallel 2 --max-sessions 500" sh -c "grep -c -- '--parallel 2 --max-sessions 500' '$CASE/calls.argv' | grep -qx 5"
check "order: --repetitions 118 on B alone" sh -c "grep -- '--repetitions 118' '$CASE/calls.argv' | grep -c -- '/root/b ' | grep -qx 2 && ! grep -- '/root/a ' '$CASE/calls.argv' | grep -q -- '--repetitions'"
check "order: B runs after A" sh -c "grep '^argv' '$CASE/calls.short' | sed -n 5,6p | grep -c '/root/b ' | grep -qx 2"
check "order: done, a.done and b.done are written" sh -c "[ -f '$CASE/root/done' ] && [ -f '$CASE/root/a.done' ] && [ -f '$CASE/root/b.done' ]"
check "order: one line per invocation, in the stated format" sh -c "grep -cE '^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z (canary|a|b) exit=[0-9]+ recorded=[0-9]+/[0-9]+ new=[0-9]+ cents=[0-9]+ all=[0-9]+/[0-9]+( (halt|bound|capped|refused-only))?\$' '$CASE/out' | grep -qx 6"
check "order: the log file holds the same lines" sh -c "grep -c ' exit=' '$CASE/root/drive.log' | grep -qx 6"
check "order: the last A line reads 20/20 and the files' 1000 cents" out_has " a exit=0 recorded=20/20 new=4 cents=$(spent_of a) all=$(spent_of a)/100000\$"
check "order: the last B line reads 10/10 and both batches' cents" out_has " b exit=0 recorded=10/10 new=5 cents=$(spent_of b) all=$(($(spent_of a) + $(spent_of b)))/100000\$"
check "order: a bound exit is tagged bound" out_has " a exit=9 recorded=12/20 new=4 cents=600 all=600/100000 bound\$"
drive
check "order: started again with done present, exits 0" code_is 0
check "order: started again with done present, invokes nothing" calls_are 6

# ------------------------------------------------- 2. halt backoff

new_case
only_a
printf 'exit=9 halt=y\nexit=9 halt=y\nexit=9 halt=y\nexit=0 record=20 cents=10\n' > "$CASE/scenario"
drive
check "halt: exits 0 after three halts" code_is 0
check "halt: the first halt sleeps 300" out_has "halt 1; sleeping 300 s"
check "halt: the second halt sleeps 600" out_has "halt 2; sleeping 600 s"
check "halt: the third halt sleeps 1200" out_has "halt 3; sleeping 1200 s"
check "halt: the clock moved 2100 s" [ "$(cat "$CASE/clock")" = $((1790000000 + 2100)) ]
check "halt: no single sleep is longer than 60 s" sh -c "! awk '\$2 > 60' '$CASE/sleeps' | grep -q ."
check "halt: an invocation that halts is tagged halt" out_has " a exit=9 recorded=0/20 new=0 cents=0 all=0/100000 halt\$"
new_case
only_a
echo 'exit=9 halt=y' > "$CASE/scenario"
drive
check "halt: 13 halts with no progress exit 12" code_is 12
check "halt: 13 invocations, not 14" calls_are 13
check "halt: the largest sleep is 3600" sh -c "sed -n 's/.*sleeping \([0-9]*\) s.*/\1/p' '$CASE/out' | sort -n | tail -1 | grep -qx 3600"
check "halt: twelve halts slept" sh -c "grep -c 'sleeping' '$CASE/out' | grep -qx 12"

# A halt that recorded a session starts the count again.
new_case
only_a
printf 'exit=9 halt=y\nexit=9 halt=y\nexit=9 record=1 cents=10 halt=y\nexit=0 record=20 cents=10\n' > "$CASE/scenario"
drive
check "halt: a halt with progress counts from 1 again" sh -c "grep 'sleeping' '$CASE/out' | sed -n 3p | grep -q 'halt 1; sleeping 300 s'"

# A halt beside refusals is a halt.
new_case
only_a
printf 'exit=7 halt=y refuse=2\nexit=0 record=20 cents=10\n' > "$CASE/scenario"
drive
check "halt: exit 7 with a halt and refusals takes the halt row" out_has "halt 1; sleeping 300 s"
check "halt: and then finishes" code_is 0

# ------------------------------------------------- 3. token

new_case
only_a
write_token $((1790000000 + 600))
echo 'exit=0 record=20 cents=10' > "$CASE/scenario"
cat > "$CASE/on-sleep" <<'EOF'
printf '{"claudeAiOauth":{"expiresAt":%s000}}\n' $(($(cat "$CASE/clock") + 8 * 3600)) > "$CASE/host/.credentials.json"
EOF
drive
check "token: exits 0 once the host refreshed the token" code_is 0
check "token: the wait slept before the first invocation" sh -c "head -1 '$CASE/events' | grep -q '^sleep '"
check "token: one wait line is logged" sh -c "grep -c 'wait token expires in 10 min; waiting for the host to refresh it' '$CASE/out' | grep -qx 1"
check "token: one invocation" calls_are 1
new_case
only_a
echo 'exit=0 record=20' > "$CASE/scenario"
rm "$CASE/host/.credentials.json"
drive
check "token: a missing credentials file exits 3" code_is 3
check "token: and invokes nothing" calls_are 0
new_case
only_a
echo 'exit=0 record=20' > "$CASE/scenario"
echo '{"claudeAiOauth":{}}' > "$CASE/host/.credentials.json"
drive
check "token: a missing expiresAt exits 3" code_is 3
# A long wait logs every 10 minutes, not every minute.
new_case
only_a
write_token $((1790000000 + 60))
echo 'exit=0 record=20 cents=10' > "$CASE/scenario"
cat > "$CASE/on-sleep" <<'EOF'
if [ "$(wc -l < "$CASE/sleeps")" -lt 25 ]; then
    cp "$CASE/on-sleep.ran" "$CASE/on-sleep"
else
    echo '{"claudeAiOauth":{"expiresAt":99999999999000}}' > "$CASE/host/.credentials.json"
fi
EOF
drive
check "token: a 25-minute wait logs 3 lines" sh -c "grep -c 'wait token' '$CASE/out' | grep -qx 3"

# ------------------------------------------------- 4. the pinned harness

new_case
only_a
echo 'exit=0 record=20 cents=10' > "$CASE/scenario"
drive
check "harness: the campaign sees the pinned version, not the decoy on PATH" sh -c "grep -qx '2.1.288 (Claude Code)' '$CASE/root/a/claude-version'"
check "harness: <root>/bin/claude names the version" [ "$(readlink "$CASE/root/bin/claude")" = "$fix/versions/2.1.288" ]
new_case
only_a
VERSION=2.1.999
echo 'exit=0 record=20' > "$CASE/scenario"
drive
check "harness: a version absent from the versions directory exits 3" code_is 3
check "harness: and invokes nothing" calls_are 0
new_case
only_a
mkdir -p "$CASE/root/a"
echo '2.1.287 (Claude Code)' > "$CASE/root/a/claude-version"
echo 'exit=0 record=20' > "$CASE/scenario"
drive
check "harness: a batch started on another version exits 4" code_is 4
check "harness: and invokes nothing" calls_are 0

# ------------------------------------------------- 5. the pin

pin_case() {
    new_case
    only_a
    echo 'exit=0 record=20' > "$CASE/scenario"
}
g2() { git -C "$CASE/repo" -c user.name=fixture -c user.email=fixture@example.invalid -c core.hooksPath=/dev/null "$@"; }
pin_case
PIN=0000000000000000000000000000000000000000
drive
check "pin: HEAD other than --pin exits 4" code_is 4
check "pin: HEAD other than --pin invokes nothing" calls_are 0
pin_case
g2 checkout -q main
drive
check "pin: an attached branch exits 4" code_is 4
check "pin: an attached branch invokes nothing" calls_are 0
pin_case
echo two > "$CASE/repo/file"
g2 commit -q -a -m two
g2 checkout -q --detach
PIN=$(g2 rev-parse HEAD)
drive
check "pin: a pin not on origin/main exits 4" code_is 4
check "pin: a pin not on origin/main invokes nothing" calls_are 0
pin_case
: > "$CASE/repo/untracked"
drive
check "pin: a dirty tree exits 4" code_is 4
check "pin: a dirty tree invokes nothing" calls_are 0
# The pin is checked again before every invocation, not only at the start.
pin_case
printf 'exit=9 record=1 cents=10\nexit=0 record=20 cents=10\n' > "$CASE/scenario"
printf '#!/bin/sh\n: > "$CASE/repo/late"\nexec sh %s "$@"\n' "$fix/campaign-stub.sh" > "$CASE/dirtying-stub.sh"
(
    cd "$CASE/repo" || exit 99
    HEADWATER_DRIVE_CAMPAIGN=$CASE/dirtying-stub.sh HEADWATER_DRIVE_NOW="sh $fix/now.sh" HEADWATER_DRIVE_SLEEP="sh $fix/sleep.sh" \
    HEADWATER_DRIVE_CLAUDE_VERSIONS=$fix/versions HEADWATER_PROBE_HOST_CONFIG=$CASE/host PATH="$fix/decoy:$PATH" \
        timeout 60 sh "$driver" --root "$CASE/root" --pin "$PIN" --claude-version 2.1.288 --spec-a "$CASE/spec-a" \
        --spec-b "$CASE/spec-b" --repetitions-b 118 --model m --session-cents 60 --cap-cents 100000
) > "$CASE/out" 2> "$CASE/err"
echo $? > "$CASE/code"
check "pin: a tree dirtied during a batch exits 4 before the next invocation" code_is 4
check "pin: after one invocation" calls_are 1

# ------------------------------------------------- 6. stops

for stop in 4 5 6 1; do
    new_case
    only_a
    echo "exit=$stop" > "$CASE/scenario"
    drive
    check "stops: a campaign exit of $stop stops with $stop" code_is "$stop"
    check "stops: a campaign exit of $stop writes no done" hasnt done
    check "stops: a campaign exit of $stop invokes once" calls_are 1
done
new_case
only_a
echo 'exit=3' > "$CASE/scenario"
drive
check "stops: another campaign exit stops with that code" code_is 3
new_case
only_a
printf 'exit=9 halt=y\nexit=0 record=20\n' > "$CASE/scenario"
echo ': > "$CASE/root/stop"' > "$CASE/on-sleep"
drive
check "stops: a stop written during a halt backoff exits 10" code_is 10
check "stops: before the next invocation" calls_are 1
check "stops: and logs stop" sh -c "grep -qE 'Z stop\$' '$CASE/out'"
new_case
only_a
: > "$CASE/root/stop"
echo 'exit=0 record=20' > "$CASE/scenario"
drive
check "stops: a stop present at the start exits 10 with no invocation" sh -c "[ \"\$(cat '$CASE/code')\" = 10 ] && [ ! -f '$CASE/calls.short' ]"
new_case
only_a
echo 'exit=7 record=2 cents=10 fail=1 cents=10' > "$CASE/scenario"
drive
check "stops: a recorder failure exits 7" code_is 7
check "stops: after one invocation" calls_are 1
check "stops: and names the session" out_has "a-003"
check "stops: and writes no capped" hasnt capped
# A recorder failure among refusals is still a recorder failure.
new_case
only_a
echo 'exit=7 refuse=2 fail=1 cents=10' > "$CASE/scenario"
drive
check "stops: a failure among refusals exits 7" code_is 7
check "stops: a failure among refusals is not re-tried" calls_are 1

# A refusal-only exit 7 re-tries once, and goes on when the re-try records.
new_case
only_a
printf 'exit=7 record=1 cents=10 refuse=3\nexit=7 record=2 cents=10 refuse=3\nexit=0 record=20 cents=10\n' > "$CASE/scenario"
drive
check "refused: a re-try that records goes on to the end" code_is 0
check "refused: three invocations" calls_are 3
check "refused: an invocation refused alone is tagged refused-only" out_has " a exit=7 recorded=1/20 new=1 cents=10 all=10/100000 refused-only\$"
new_case
only_a
printf 'exit=7 refuse=3\n' > "$CASE/scenario"
drive
check "refused: a re-try with no new session exits 8" code_is 8
check "refused: after two invocations" calls_are 2
check "refused: the second line is tagged capped" out_has " a exit=7 recorded=0/20 new=0 cents=0 all=0/100000 capped\$"
check "refused: capped states the counts" grep -q '^recorded: 0 of 20 in a, 0 of 0 in b$' "$CASE/root/capped"

# A job a halt left with status 10 and the next invocation left unstarted is
# not a failure of that invocation: the driver reads slice/started.
new_case
only_a
printf 'exit=9 halt=y\nexit=7 skip=1 refuse=2\nexit=0 record=20 cents=10\n' > "$CASE/scenario"
drive
check "refused: a job halted earlier and unstarted now is not a recorder failure" code_is 0

# The cap left is 0 or less: nothing is invoked.
new_case
only_a
C=30
echo 'exit=0 record=20' > "$CASE/scenario"
drive
check "cap: a cap of C - spent - P x M at or below 0 exits 8" code_is 8
check "cap: and invokes nothing" calls_are 0
check "cap: and writes capped" has capped

# The alarm: a campaign that spends past the cap stops the driver.
new_case
only_a
C=1000
echo 'exit=9 record=5 cents=500' > "$CASE/scenario"
drive
check "cap: a total past --cap-cents exits 7" code_is 7
check "cap: and logs the figures" out_has "spent 2500 of 1000"

# ------------------------------------------------- 7. canary gate

new_case
echo 'exit=9 record=8 cents=90' > "$CASE/scenario"
drive
check "canary: a session above B + M exits 11" code_is 11
check "canary: and writes canary-failed with the reason" grep -q 'cost 90' "$CASE/root/canary-failed"
check "canary: and writes no canary-recorded" hasnt canary-recorded
drive
check "canary: started again with canary-failed present, exits 11 with no invocation" sh -c "[ \"\$(cat '$CASE/code')\" = 11 ] && [ \"\$(grep -c '^argv' '$CASE/calls.short')\" = 1 ]"
new_case
: > "$CASE/root/b.done"
printf 'exit=9 record=8 cents=50\nexit=0 record=12 cents=50\n' > "$CASE/scenario"
echo ': > "$CASE/root/canary-passed"' > "$CASE/on-sleep"
drive
check "canary: a good canary writes canary-recorded" has canary-recorded
check "canary: waits for canary-passed and then runs A" code_is 0
check "canary: logs the wait" out_has "wait canary-passed"
check "canary: A runs after the wait" sh -c "sed -n 2,3p '$CASE/events' | tr '\n' ' ' | grep -q '^sleep 60 call '"
check "canary: two invocations" calls_are 2
new_case
: > "$CASE/root/b.done"
printf 'exit=9 record=5 cents=50 halt=y\nexit=9 record=3 cents=50\nexit=0 record=12 cents=50\n' > "$CASE/scenario"
: > "$CASE/root/canary-passed"
drive
check "canary: a halted canary asks for the 3 sessions it still owes" grep -q '/root/a cap=99980 max=3$' "$CASE/calls.short"
check "canary: and then passes" code_is 0
new_case
only_a
echo 'exit=0 record=20 cents=10' > "$CASE/scenario"
drive
check "canary: started again after canary-recorded and canary-passed, runs no canary" sh -c "! grep -q 'max=8\$' '$CASE/calls.short' && grep -q 'max=500\$' '$CASE/calls.short'"

# ------------------------------------------------- 8. no paid session

check "no case called claude with anything but --version" sh -c "[ ! -s '$fix/paid.log' ]"

rm -rf "$fix"
echo "campaign-drive-fixtures: $passed passed, $failed failed"
[ "$failed" = 0 ]
