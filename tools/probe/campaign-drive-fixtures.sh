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
# Section 7a holds the saturation stop of phase B (#1875) against a stub of
# its seam, and 7b holds the seam, `tools/probe/cell-passes.sh`, against a
# stub engine and a stub `campaign.sh --assemble`. Its decisive case is the
# first of 7a: of four cells, 24 of 24 and 0 of 24 stop and 13 of 24 and 23
# of 23 graded run on.
#
# Exit status: 0 when every case passes, 1 when one fails, 3 when `jq`, `git`,
# `timeout` or `python3` is missing.

set -u

for tool in jq git awk timeout python3; do
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
# A `claude` of the pinned version, for a case that runs the stub by hand.
mkdir -p "$fix/vbin"
ln -s "$fix/versions/2.1.288" "$fix/vbin/claude"

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

# The poll: the step between two reads of the usage gate while campaign.sh
# runs (#1876). It moves no clock and logs no sleep, so the cases that count
# the sleeps of a wait read only those, and it runs the one-shot hook
# $CASE/on-poll.
cat > "$fix/poll.sh" <<'EOF'
sleep 0.01
if [ -f "$CASE/on-poll" ]; then
    mv "$CASE/on-poll" "$CASE/on-poll.ran"
    sh "$CASE/on-poll.ran"
fi
EOF

# The campaign. Each invocation reads the next line of $CASE/scenario (the
# last line repeats once they run out), which is either
#     exit=<n> [record=<k>] [cents=<c>] [halt=y] [skip=<k>] [refuse=<k>] [fail=<k>] [provider=<k>] [waithalt]
# or
#     auto cents=<c>
# A manual line walks the jobs not yet recorded, in order: it leaves `skip`
# of them unstarted, records `record` at `cents` each, halts the next one
# when `halt=y`, refuses the next `refuse`, fails the next `fail`, and exits
# with `exit`. An `auto` line honors the cap it was given as campaign.sh
# does at --parallel 1: it records a job at `cents` while spent plus the
# session budget is at most the cap and the bound allows, refuses every job
# after that, and exits 7 on a refusal, 9 on the bound and 0 when all are
# recorded. A session a manual line records gets a `record.md` stating that
# the proxy refused `provider` provider fetches (default 0), in the words
# probe-record.sh prints. With `waithalt`, the invocation writes
# $CASE/stub-ready once its slice is fresh, and walks its jobs only after
# `<out>/slice/halt` exists, polling for it in real time for at most 20 s:
# the jobs in flight when the driver halts it still finish (#1876).
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
code= record=0 cents=0 halt=n skip=0 refuse=0 fail=0 auto=n provider=0 unrec=0 nomark=n waithalt=n
for word in $line; do
    case $word in
        auto) auto=y ;;
        waithalt) waithalt=y ;;
        unrec=*) unrec=${word#unrec=} ;;
        nomark) nomark=y ;;
        exit=*) code=${word#exit=} ;;
        record=*) record=${word#record=} ;;
        cents=*) cents=${word#cents=} ;;
        halt=*) halt=${word#halt=} ;;
        skip=*) skip=${word#skip=} ;;
        refuse=*) refuse=${word#refuse=} ;;
        fail=*) fail=${word#fail=} ;;
        provider=*) provider=${word#provider=} ;;
    esac
done
printf 'argv %s cap=%s max=%s\n' "$out" "$cap" "$max" >> "$CASE/calls.short"
if [ "$waithalt" = y ]; then
    : > "$CASE/stub-ready"
    tries=0
    while [ ! -f "$out/slice/halt" ] && [ "$tries" -lt 2000 ]; do
        sleep 0.01
        tries=$((tries + 1))
    done
fi
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
    # campaign.sh never draws a job `--unrecorded` marked (#1659).
    if [ -f "$out/unrecorded/$name" ]; then
        echo "skip-unrecorded $name" >> "$CASE/events"
        continue
    fi
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
        if [ "$provider" = 0 ]; then said="no request"; else said="$provider request"; fi
        echo "The proxy refused a request to the provider API that asked the provider to fetch from another host for the session. It refused $said of that kind, and no request whose head it does not forward." > "$out/sessions/$name/record.md"
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
    elif [ "$unrec" -gt 0 ]; then
        # A recorder exit 5 that a person accepted with `--unrecorded`, or,
        # with `nomark`, one nobody accepted.
        start "$name"
        echo 5 > "$out/sessions/$name/status"
        echo "$cents" > "$out/sessions/$name/cost"
        if [ "$nomark" = n ]; then
            mkdir -p "$out/unrecorded"
            echo "2026-10-04T00:00:00Z probe-transform: the artifact x could not be copied" > "$out/unrecorded/$name"
        fi
        unrec=$((unrec - 1))
    fi
done < "$out/jobs"
if [ "$auto" = y ]; then
    [ "$refused" = 1 ] && exit 7
    [ "$left" = 1 ] && exit 9
    exit 0
fi
exit "$code"
EOF

# The seam of the saturation stop (#1875), `tools/probe/cell-passes.sh`.
# `counts <batch>` logs the call to $CASE/cells.calls and prints
# $CASE/cells, the graded counts of batch A as
# `<arm> <set> <action> <probe> <satisfied> <graded>`, or exits 4 when
# $CASE/cells-fail exists. `probes <tier> <arm> <round> [<exclude>...]`
# prints each row of $CASE/probes, `<arm> <set> <probe> <action>`, whose
# arm is the line's and whose probe the line does not exclude, and nothing
# for a line of another tier, as the helper does.
cat > "$fix/cells-stub.sh" <<'EOF'
set -u
verb=$1
shift
case $verb in
    counts)
        echo "counts $*" >> "$CASE/cells.calls"
        [ -f "$CASE/cells-fail" ] && { echo "cell-passes: the grade refused the transcript" >&2; exit 4; }
        cat "$CASE/cells"
        ;;
    probes)
        [ "$1" = additive ] || exit 0
        [ -f "$CASE/probes-fail" ] && { echo "cell-passes: the plan of scenario $2 was refused" >&2; exit 4; }
        arm=$2
        shift 3
        awk -v arm="$arm" -v out="$*" '
            BEGIN { n = split(out, o, " "); for (i = 1; i <= n; i++) gone[o[i]] = 1 }
            $1 == arm && !($3 in gone)
        ' "$CASE/probes"
        ;;
esac
EOF

# ---------------------------------------------------------------- a case

# new_case: a fresh case directory with a repository detached at the pin,
# a clock at a fixed time, credentials that outlive it by 30 days, and two
# specs of 20 and 10 jobs. Sets CASE, PIN and the driver's defaults.
new_case() {
    CASE=$(mktemp -d "$fix/case.XXXXXX")
    export CASE
    echo 1790000000 > "$CASE/clock"
    : > "$CASE/events"
    mkdir -p "$CASE/host" "$CASE/repo" "$CASE/root"
    write_token $((1790000000 + 30 * 86400))
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
    B=60 C=100000 M=20 P=2 VERSION=2.1.288 RAMP=
    # The usage gate of #1876, at the values of `additive-launch.env`. The
    # snapshot directory is the case's own, so the host's real snapshots
    # never decide a case.
    mkdir -p "$CASE/usage"
    U_STOP=85 U_LATE=100 U_HOURS=24
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
        if [ -n "$RAMP" ]; then set -- --ramp "$RAMP" "$@"; else set -- --parallel "$P" "$@"; fi
        # A_ONLY runs the driver with no batch B at all (#1775).
        [ -n "${A_ONLY:-}" ] || set -- --spec-b "$CASE/spec-b" --repetitions-b 118 "$@"
        HEADWATER_DRIVE_CAMPAIGN=$fix/campaign-stub.sh \
        HEADWATER_DRIVE_CELLS=$fix/cells-stub.sh \
        HEADWATER_DRIVE_NOW="sh $fix/now.sh" \
        HEADWATER_DRIVE_SLEEP="sh $fix/sleep.sh" \
        HEADWATER_DRIVE_CLAUDE_VERSIONS=$fix/versions \
        HEADWATER_PROBE_HOST_CONFIG=$CASE/host \
        HEADWATER_USAGE_DIR=$CASE/usage \
        HEADWATER_DRIVE_USAGE_STOP=$U_STOP \
        HEADWATER_DRIVE_USAGE_LATE=$U_LATE \
        HEADWATER_DRIVE_USAGE_LATE_HOURS=$U_HOURS \
        HEADWATER_DRIVE_POLL="sh $fix/poll.sh" \
        PATH="$fix/decoy:$PATH" \
            timeout 60 sh "$driver" --root "$CASE/root" --pin "$PIN" --claude-version "$VERSION" \
            --spec-a "$CASE/spec-a" --model fixture-model \
            --session-cents "$B" --cap-cents "$C" --overshoot-cents "$M" "$@"
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

# ------------------------------- the restart: an earlier attempt is counted
#
# The screening round of the additive campaign is approved at 70000 cents
# over every attempt (#1775). A restart on a new pin or a moved harness
# needs a new batch directory, and the old one is moved aside under the
# same root. The cap the new batch is given is 70000 less what the earlier
# attempt spent, its runs again included, less p x M.

new_case
C=70000 B=60 M=20 P=2
only_a
mkdir -p "$CASE/root/a.attempt-1/sessions/s1" "$CASE/root/a.attempt-1/sessions/s2"
echo 30000 > "$CASE/root/a.attempt-1/sessions/s1/cost"
echo 400 > "$CASE/root/a.attempt-1/sessions/s2/cost"
echo 100 > "$CASE/root/a.attempt-1/sessions/s2/spent-before"
printf 'auto cents=50\n' > "$CASE/scenario"
drive
check "restart: batch A is given C - the earlier attempt's 30500 - 2 x M = 39460" \
    grep -q "/root/a cap=39460 max=500\$" "$CASE/calls.short"
check "restart: the driver finishes" code_is 0
check "restart: every attempt together is at most C" [ $(($(spent_of a) + 30500)) -le "$C" ]
# An earlier attempt that spent the whole approval leaves nothing to give.
new_case
C=70000 B=60 M=20 P=2
only_a
mkdir -p "$CASE/root/a.attempt-1/sessions/s1"
echo 69990 > "$CASE/root/a.attempt-1/sessions/s1/cost"
printf 'auto cents=50\n' > "$CASE/scenario"
drive
check "restart: an earlier attempt that left less than p x M caps the new batch with 8" code_is 8
check "restart: and the new batch is not invoked" calls_are 0
check "restart: the capped file states what every attempt spent" grep -q "^spent-all: 69990\$" "$CASE/root/capped"
# An attempt moved aside under a name that starts with a dot is still a
# directory of the root, and a shell glob of `*` does not list it (verify
# round 1 of #1775).
new_case
C=70000 B=60 M=20 P=2
only_a
mkdir -p "$CASE/root/.a-attempt-1/sessions/s1" "$CASE/root/..a-attempt-2/sessions/s1"
echo 30000 > "$CASE/root/.a-attempt-1/sessions/s1/cost"
echo 500 > "$CASE/root/..a-attempt-2/sessions/s1/cost"
printf 'auto cents=50\n' > "$CASE/scenario"
drive
check "restart: an attempt under a dot-named directory is counted too (70000 - 30500 - 40 = 39460)" \
    grep -q "/root/a cap=39460 max=500\$" "$CASE/calls.short"

# ------------------------------------------- batch A alone, with no batch B

new_case
only_a
rm -f "$CASE/root/b.done"
printf 'auto cents=50\n' > "$CASE/scenario"
A_ONLY=1 drive
check "a-only: a driver with no --spec-b runs batch A and exits 0" code_is 0
check "a-only: it writes done after a.done, with no b.done" [ -f "$CASE/root/done" -a ! -f "$CASE/root/b.done" ]
check "a-only: it never invokes batch B" [ "$(grep -c '/root/b ' "$CASE/calls.short")" = 0 ]
new_case
A_ONLY=1 drive --repetitions-b 118
check "a-only: --repetitions-b without --spec-b is a usage error" code_is 2
# The saturation stop (#1875) stops cells of batch B, so it needs one.
new_case
A_ONLY=1 drive --saturate-at 24
check "usage: --saturate-at without --spec-b is a usage error" code_is 2
check "usage: and invokes nothing" calls_are 0
for value in 0 024 -1 x ''; do
    new_case
    drive --saturate-at "$value"
    check "usage: --saturate-at '$value' is a usage error" code_is 2
    check "usage: --saturate-at '$value' invokes nothing" calls_are 0
done

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
# The two hooks (#1821) log each run, with the markers they found, so the
# case shows when each ran.
printf 'echo "$1 $(ls "$CASE/root" | tr "\\n" " ")" >> "$CASE/hooks.log"\n' > "$CASE/hook.sh"
drive --on-canary "sh $CASE/hook.sh canary" --on-done "sh $CASE/hook.sh done"
check "order: exits 0" code_is 0
check "hooks: --on-canary runs once" [ "$(grep -c '^canary ' "$CASE/hooks.log")" = 1 ]
check "hooks: --on-canary runs after canary-recorded is written, before a.done" sh -c "grep '^canary ' '$CASE/hooks.log' | grep -q ' canary-recorded ' && ! grep '^canary ' '$CASE/hooks.log' | grep -q ' a.done '"
check "hooks: --on-done runs once" [ "$(grep -c '^done ' "$CASE/hooks.log")" = 1 ]
check "hooks: --on-done runs after done is written, and last" sh -c "tail -1 '$CASE/hooks.log' | grep -q '^done .* done '"
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
drive --on-canary "sh $CASE/hook.sh canary" --on-done "sh $CASE/hook.sh done"
check "order: started again with done present, exits 0" code_is 0
check "order: started again with done present, invokes nothing" calls_are 6
check "hooks: a driver started with done present runs neither hook" [ "$(wc -l < "$CASE/hooks.log")" = 2 ]

# A reader on --on-canary that writes canary-passed lets the driver go on
# with nobody awake: the hook runs before the wait, so the wait never logs.
new_case
A_ONLY=1
printf 'exit=9 record=8 cents=50\nexit=0 record=12 cents=50\n' > "$CASE/scenario"
A_ONLY=1 drive --on-canary ": > $CASE/root/canary-passed"
check "hooks: an --on-canary that writes canary-passed lets the driver finish" code_is 0
check "hooks: and the driver never waits for a reader" sh -c "! grep -q 'wait canary-passed' '$CASE/root/drive.log'"
A_ONLY=

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

# A slice that ends on the bound ends the run of halts.
new_case
only_a
: > "$CASE/scenario"
for i in 1 2 3 4 5 6 7; do echo 'exit=9 halt=y' >> "$CASE/scenario"; done
echo 'exit=9 record=1 cents=10' >> "$CASE/scenario"
for i in 1 2 3 4 5 6 7; do echo 'exit=9 halt=y' >> "$CASE/scenario"; done
echo 'exit=0 record=20 cents=10' >> "$CASE/scenario"
drive
check "halt: 7 halts, a bound and 7 halts do not reach the limit" code_is 0
check "halt: the halt after a bound counts from 1" sh -c "grep 'sleeping' '$CASE/out' | sed -n 8p | grep -q 'halt 1; sleeping 300 s'"

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

# ------------------------------------------------- 3a. the usage gate
#
# The owner's ruling on #1868: a run stops launching sessions at 85 percent
# of the 7-day window, unless the reset is under 24 hours away, in which case
# it runs on to 100 percent (#1876). The driver reads the freshest figure of
# the snapshot directory and of the last `rate_limit_event` of its own
# transcripts, before each invocation and while one runs.

# snap <percent> <resets_at> [<last_activity>]: one snapshot in the case's
# usage directory, in the shape the host's status line writes.
snap() {
    usnap=$((${usnap:-0} + 1))
    printf '{"session_id":"s%s","last_activity":%s,"seven_day":{"used_percentage":%s,"resets_at":"%s"}}\n' \
        "$usnap" "${3:-$(($(cat "$CASE/clock") - 60))}" "$1" "$2" > "$CASE/usage/s$usnap.json"
}
# transcript <mtime> <line>: the raw.jsonl of a session of batch A, with one
# rate limit line between two others, dated by its mtime.
transcript() {
    mkdir -p "$CASE/root/a/sessions/a-001"
    printf '{"type":"assistant"}\n%s\n{"type":"result"}\n' "$2" > "$CASE/root/a/sessions/a-001/raw.jsonl"
    touch -d "@$1" "$CASE/root/a/sessions/a-001/raw.jsonl"
}
# window <utilization> <resets_at>: a rate limit line of the harness, whose
# 7-day window is under `unifiedWindows`.
window() {
    printf '{"type":"rate_limit_event","rate_limit_info":{"rateLimitType":"five_hour","utilization":0.1,"resetsAt":%s,"unifiedWindows":{"five_hour":{"utilization":0.1,"resetsAt":%s},"seven_day":{"utilization":%s,"resetsAt":%s}}}}' \
        "$2" "$2" "$1" "$2"
}
t0=1790000000
h48=$((t0 + 48 * 3600))
h12=$((t0 + 12 * 3600))
recorded_a() { grep -lx 0 "$CASE/root/a/sessions"/*/status 2>/dev/null | wc -l | tr -d ' '; }

# The decisive case: the figure passes 85 percent while an invocation runs.
# The driver halts the invocation, so campaign.sh starts nothing more, and
# exits 13 with no halt backoff. A relaunch below the threshold then records
# every job once.
new_case
only_a
printf 'exit=9 record=3 cents=10 waithalt\nexit=0 record=20 cents=10\n' > "$CASE/scenario"
cat > "$CASE/on-poll" <<EOF
tries=0
while [ ! -f "\$CASE/stub-ready" ] && [ "\$tries" -lt 2000 ]; do sleep 0.01; tries=\$((tries + 1)); done
printf '{"last_activity":%s,"seven_day":{"used_percentage":86,"resets_at":"%s"}}\n' $t0 $h48 > "\$CASE/usage/late.json"
EOF
drive
check "usage: a figure past 85% during an invocation exits 13" code_is 13
check "usage: and writes the halt of the slice" has a/slice/halt
check "usage: and writes <root>/usage-gated" has usage-gated
check "usage: and logs one line naming 86% and the reset" sh -c "grep -c 'usage-gated: 7-day at 86% (from .*late.json), threshold 85%, resets 2026-09-23T14:13:20Z' '$CASE/out' | grep -qx 1"
check "usage: with one invocation" calls_are 1
check "usage: and no halt backoff" sh -c "! grep -q 'halt 1; sleeping' '$CASE/out'"
check "usage: the jobs in flight record" [ "$(recorded_a)" = 3 ]
snap 50 "$h48" "$((t0 + 1))"
drive
check "usage: a relaunch below the threshold finishes" code_is 0
check "usage: and removes usage-gated" hasnt usage-gated
check "usage: and invokes once more" calls_are 2
check "usage: every job is recorded" [ "$(recorded_a)" = 20 ]
check "usage: and none twice" sh -c "! find '$CASE/root/a/sessions' -name spent-before | grep -q ."

# The thresholds, each read before the invocation.
usage_case() {
    # usage_case <description> <expected exit> <expected calls>
    drive
    check "usage: $1 exits $2" code_is "$2"
    check "usage: $1 invokes $3 times" calls_are "$3"
}
new_case
only_a
echo 'exit=0 record=20 cents=10' > "$CASE/scenario"
snap 84 "$h48"
usage_case "84% with 48 h to the reset" 0 1
new_case
only_a
echo 'exit=0 record=20 cents=10' > "$CASE/scenario"
snap 85 "$h48"
usage_case "85% with 48 h to the reset" 13 0
check "usage: and the marker holds the figure" grep -q '^percent: 85$' "$CASE/root/usage-gated"
check "usage: and the threshold" grep -q '^threshold: 85$' "$CASE/root/usage-gated"
check "usage: and the reset" grep -q "^resets-at: $h48\$" "$CASE/root/usage-gated"
new_case
only_a
echo 'exit=0 record=20 cents=10' > "$CASE/scenario"
snap 86 "$h12"
usage_case "86% with 12 h to the reset, the late threshold," 0 1
new_case
only_a
echo 'exit=0 record=20 cents=10' > "$CASE/scenario"
snap 99.9 "$h12"
usage_case "99.9% with 12 h to the reset" 0 1
new_case
only_a
echo 'exit=0 record=20 cents=10' > "$CASE/scenario"
snap 100 "$h12"
usage_case "100% with 12 h to the reset" 13 0
check "usage: and names the late threshold" out_has "threshold 100%"
new_case
only_a
echo 'exit=0 record=20 cents=10' > "$CASE/scenario"
snap 90 "$((t0 + 24 * 3600))"
usage_case "90% with exactly 24 h to the reset" 13 0
new_case
only_a
echo 'exit=0 record=20 cents=10' > "$CASE/scenario"
snap 50 "$h48" "$((t0 - 600))"
transcript "$((t0 - 60))" "$(window 0.90 "$h48")"
usage_case "a transcript at 0.90 newer than a snapshot at 50" 13 0
check "usage: and the source names the transcript" out_has "usage-gated: 7-day at 90% (from .*a-001/raw.jsonl)"
new_case
only_a
echo 'exit=0 record=20 cents=10' > "$CASE/scenario"
snap 50 "$h48" "$((t0 - 60))"
transcript "$((t0 - 600))" "$(window 0.90 "$h48")"
usage_case "a transcript at 0.90 older than a snapshot at 50" 0 1
new_case
only_a
echo 'exit=0 record=20 cents=10' > "$CASE/scenario"
transcript "$((t0 - 60))" "{\"type\":\"rate_limit_event\",\"rate_limit_info\":{\"rateLimitType\":\"seven_day\",\"utilization\":0.95,\"resetsAt\":$h48}}"
usage_case "a top-level seven_day line at 0.95" 13 0
new_case
only_a
echo 'exit=0 record=20 cents=10' > "$CASE/scenario"
transcript "$((t0 - 60))" "{\"type\":\"rate_limit_event\",\"rate_limit_info\":{\"rateLimitType\":\"five_hour\",\"utilization\":0.99,\"resetsAt\":$h48}}"
usage_case "a top-level five_hour line at 0.99, which is no 7-day figure," 0 1
new_case
only_a
echo 'exit=0 record=20 cents=10' > "$CASE/scenario"
transcript "$((t0 - 60))" '{"type":"rate_limit_event","rate_limit":{"utilization":0.88}}'
usage_case "the older rate_limit shape, which is no figure," 0 1
check "usage: and it reads as unknown" out_has "usage unknown"
new_case
only_a
echo 'exit=0 record=20 cents=10' > "$CASE/scenario"
# At 100 percent, so that a figure read as live would trip even the late
# threshold.
snap 100 "$((t0 - 1))"
usage_case "a snapshot whose reset is past" 0 1
check "usage: and logs nothing gated" sh -c "! grep -q 'usage-gated' '$CASE/out'"
new_case
only_a
echo 'exit=0 record=20 cents=10' > "$CASE/scenario"
transcript "$((t0 - 60))" "$(window 1.0 "$((t0 - 1))")"
usage_case "a transcript whose reset is past" 0 1
check "usage: and logs nothing gated" sh -c "! grep -q 'usage-gated' '$CASE/out'"
new_case
only_a
echo 'exit=0 record=20 cents=10' > "$CASE/scenario"
usage_case "no figure" 0 1
check "usage: and logs one usage unknown line" sh -c "grep -c 'usage unknown' '$CASE/out' | grep -qx 1"
new_case
only_a
echo 'exit=0 record=20 cents=10' > "$CASE/scenario"
snap 99 "$h48"
U_STOP= U_LATE= U_HOURS=
usage_case "the three keys unset" 0 1
check "usage: and logs one usage gate off line" sh -c "grep -c 'usage gate off' '$CASE/out' | grep -qx 1"
new_case
only_a
echo 'exit=0 record=20 cents=10' > "$CASE/scenario"
U_STOP=eighty
usage_case "a threshold that is not a whole number" 2 0
new_case
only_a
echo 'exit=0 record=20 cents=10' > "$CASE/scenario"
snap 99 "$h48"
: > "$CASE/root/usage-gated"
snap 10 "$h48" "$t0"
usage_case "a stale usage-gated below the threshold" 0 1
check "usage: and the driver removes it" hasnt usage-gated

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
echo two > "$CASE/repo/file"
g2 commit -q -a -m two
g2 update-ref refs/remotes/origin/main HEAD
drive
check "pin: a HEAD past a pin that is on origin/main exits 4" code_is 4
check "pin: a HEAD past the pin invokes nothing" calls_are 0
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
g2 update-ref refs/remotes/origin/tune/fixture-v1 HEAD
drive
check "pin: a pin on a branch under origin/tune/ exits 0" code_is 0
check "pin: a pin on a branch under origin/tune/ invokes the batch" [ "$(calls)" -gt 0 ]
pin_case
echo two > "$CASE/repo/file"
g2 commit -q -a -m two
g2 checkout -q --detach
g2 update-ref refs/remotes/origin/other/v1 HEAD
PIN=$(g2 rev-parse HEAD)
drive
check "pin: a pin on a pushed branch outside origin/tune/ exits 4" code_is 4
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
check "stops: within one step of 60 s, not at the end of the 300 s backoff" [ "$(wc -l < "$CASE/sleeps")" = 1 ]
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
# A status 5 that `campaign.sh --unrecorded` marked is a counted non-record,
# not a recorder failure (#1659). The slice it ended in is a clean slice, and
# the job is not drawn again.
new_case
only_a
printf 'exit=7 record=1 cents=10 unrec=1 cents=10\nexit=0 record=1 cents=10\n' > "$CASE/scenario"
drive
check "stops: a marked status-5 job does not exit 7" sh -c "[ \"\$(cat '$CASE/code')\" != 7 ]"
check "stops: and the batch finishes with 0" code_is 0
check "stops: and is not drawn again" sh -c "grep -q '^skip-unrecorded a-002' '$CASE/events'"
check "stops: and keeps its status 5" sh -c "[ \"\$(cat '$CASE/root/a/sessions/a-002/status')\" = 5 ]"
check "stops: and is not named as a failed recorder" sh -c "! grep -q 'a recorder failed' '$CASE/out'"
# The same status 5 with no marker is still a recorder failure.
new_case
only_a
printf 'exit=7 record=1 cents=10 unrec=1 nomark cents=10\n' > "$CASE/scenario"
drive
check "stops: an unmarked status-5 job still exits 7" code_is 7
check "stops: after one invocation, as a recorder failure" calls_are 1
check "stops: and names that session and nothing else" out_has "a recorder failed for a-002;"

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

# ------------------------------------------------- the ramp

# caps_hold: every invocation in calls.argv was given C less what the other
# batch had spent less its own --parallel times M. The stub's spend of the
# other batch is fixed while a batch runs here, so the expected figure is
# read from the files at the end.
caps_hold() {
    awk -v c="$C" -v m="$M" -v a="$(spent_of a)" '
        {
            for (i = 1; i < NF; i++) {
                if ($i == "--out") out = $(i + 1)
                if ($i == "--parallel") p = $(i + 1)
                if ($i == "--cap-cents") cap = $(i + 1)
            }
            other = (out ~ /\/b$/) ? a : 0
            if (cap != c - other - p * m) { print "bad: " $0; bad++ }
        }
        END { exit bad > 0 }
    ' "$CASE/calls.argv"
}
parallels_of() {
    grep -- "--out $CASE/root/$1 " "$CASE/calls.argv" | sed 's/.*--parallel \([0-9]*\).*/\1/' | tr '\n' ' '
}

new_case
RAMP=2,3,4
echo "jobs 10" > "$CASE/spec-a"
echo "jobs 60" > "$CASE/spec-b"
: > "$CASE/root/canary-passed"
cat > "$CASE/scenario" <<'EOF'
exit=9 record=8 cents=50
exit=0 record=2 cents=50
exit=9 record=4 cents=50
exit=9 record=4 cents=50
exit=9 record=4 cents=50
exit=9 record=2 cents=50 halt=y
exit=9 record=4 cents=70
exit=9 record=4 cents=50 provider=1
exit=7 record=2 cents=50 refuse=1
exit=9 record=4 cents=50
exit=0 record=60 cents=50
EOF
drive
check "ramp: the batch finishes" code_is 0
check "ramp: the canary runs at --parallel 1" [ "$(parallels_of a)" = "1 2 " ]
check "ramp: B rises on a clean slice, stops at the last step, lowers on a halt, holds on a cost outside the canary's range, a provider refusal and a cap refusal, and rises again (saw: $(parallels_of b))" [ "$(parallels_of b)" = "2 3 4 4 3 3 3 3 4 " ]
check "ramp: every invocation's cap is C - spent(other) - its own parallel x M" caps_hold
check "ramp: both batches together within the cap" [ $(($(spent_of a) + $(spent_of b))) -le "$C" ]
check "ramp: each change is logged" sh -c "grep -c 'ramp b parallel' '$CASE/out' | grep -qx 4"
check "ramp: the canary range is written" grep -qx '50 50' "$CASE/root/canary-range"

new_case
only_a
RAMP=2,3
printf 'exit=9 halt=y\nexit=0 record=20 cents=10\n' > "$CASE/scenario"
drive
check "ramp: a halt at the first step holds it there" [ "$(parallels_of a)" = "2 2 " ]

new_case
only_a
RAMP=2,3
printf 'exit=9 record=4 cents=10\nexit=0 record=20 cents=10\n' > "$CASE/scenario"
drive
check "ramp: with no canary range the ramp holds (fails closed)" [ "$(parallels_of a)" = "2 2 " ]

for bad_ramp in 4,3 4,,6 0,2 4, two; do
    new_case
    RAMP=$bad_ramp
    drive
    check "ramp: --ramp $bad_ramp exits 2" code_is 2
done
new_case
RAMP=2,4
drive --parallel 3
check "ramp: --ramp with --parallel exits 2" code_is 2
check "ramp: and invokes nothing" calls_are 0

# ------------------------------------------------- the edges (verify round 1)

# A recorder failure beside a halt is a recorder failure, and is not run
# again: the order of the exit-7 tests is the point.
new_case
only_a
printf 'exit=7 halt=y fail=1 cents=10\nexit=0 record=20 cents=10\n' > "$CASE/scenario"
drive
check "edges: exit 7 with a halt and a failed job exits 7" code_is 7
check "edges: a failed job beside a halt is not run again" calls_are 1

# A refusal-only exit after a bound gets its own re-try: the re-try count
# is reset by every other outcome.
new_case
only_a
printf 'exit=7 record=2 cents=10 refuse=1\nexit=9 record=2 cents=10\nexit=7 refuse=1\nexit=0 record=20 cents=10\n' > "$CASE/scenario"
drive
check "edges: a refusal after a bound is re-tried and the batch finishes" code_is 0
check "edges: four invocations" calls_are 4

# A total exactly at the cap is not an overspend.
new_case
only_a
C=1000
echo 'exit=0 record=20 cents=50' > "$CASE/scenario"
drive
check "edges: spent equal to --cap-cents exits 0" code_is 0

# A cap that leaves exactly 0 is capped, with no invocation.
new_case
C=1000
: > "$CASE/root/canary-recorded"
: > "$CASE/root/canary-passed"
: > "$CASE/root/a.done"
mkdir -p "$CASE/root/a/sessions/x"
echo 960 > "$CASE/root/a/sessions/x/cost"
drive
check "edges: a cap of exactly 0 exits 8" code_is 8
check "edges: a cap of exactly 0 invokes nothing" calls_are 0

# A restart that finds the 8 canary sessions recorded, and no
# canary-recorded, invokes no canary.
new_case
: > "$CASE/root/canary-passed"
printf 'exit=9 record=8 cents=50\nexit=0 record=20 cents=50\nexit=0 record=10 cents=40\n' > "$CASE/scenario"
(
    cd "$CASE/repo" && PATH="$fix/vbin:$PATH" sh "$fix/campaign-stub.sh" --out "$CASE/root/a" --spec "$CASE/spec-a" \
        --cap-cents 99980 --max-sessions 8 --session-cents 60
) > /dev/null 2>&1
drive
check "edges: a restart with 8 canary sessions recorded exits 0" code_is 0
check "edges: and never asks for --max-sessions 0" sh -c "! grep -q 'max=0\$' '$CASE/calls.short'"
check "edges: three calls in all, the hand canary, A and B" calls_are 3
check "edges: and writes canary-recorded" has canary-recorded

# A slice that costs less than the canary's cheapest session holds the ramp.
new_case
RAMP=2,3
: > "$CASE/root/canary-passed"
: > "$CASE/root/b.done"
printf 'exit=9 record=8 cents=50\nexit=9 record=2 cents=40\nexit=0 record=20 cents=50\n' > "$CASE/scenario"
drive
check "edges: a slice below the canary range holds the ramp (saw: $(parallels_of a))" [ "$(parallels_of a)" = "1 2 2 " ]

# Batch B started on another version exits 4 before any spend.
new_case
: > "$CASE/root/canary-recorded"
: > "$CASE/root/canary-passed"
: > "$CASE/root/a.done"
mkdir -p "$CASE/root/b"
echo '2.1.287 (Claude Code)' > "$CASE/root/b/claude-version"
echo 'exit=0 record=10 cents=10' > "$CASE/scenario"
drive
check "edges: a b/claude-version of another version exits 4" code_is 4
check "edges: and invokes nothing" calls_are 0

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

# ------------------------------------------------- 7a. the saturation stop
#
# HW-DR-0112, phase B: "A cell that saturates stops at 24 sessions. A cell
# saturates when all its sessions pass or all fail at 24." A cell is one
# row (the probe's `action`) in one scenario. Batch A ran 24 sessions of
# each cell, and batch B buys the other 48 only for a cell whose result is
# not already fixed. The decisive case has four cells: 24 of 24 and 0 of 24
# stop, and 13 of 24 and 23 of 23 graded (one session refused) run on.

# saturation_case: batch A done, batch B not, a spec-b of a scenario-1 line,
# a scenario-6 line and the stub's `jobs` line, and two rows of two probes.
saturation_case() {
    new_case
    : > "$CASE/root/canary-recorded"
    : > "$CASE/root/canary-passed"
    : > "$CASE/root/a.done"
    printf 'additive scenario-1 final\nadditive scenario-6 final\njobs 10\n' > "$CASE/spec-b"
    : > "$CASE/probes"
    for arm in scenario-0 scenario-1 scenario-6; do
        printf '%s held-out P1 row_a\n%s held-out P2 row_a\n%s held-out P3 row_b\n%s held-out P4 row_b\n' \
            "$arm" "$arm" "$arm" "$arm" >> "$CASE/probes"
    done
    printf 'exit=0 record=10 cents=10\n' > "$CASE/scenario"
}
b_args() { grep -- "--out $CASE/root/b " "$CASE/calls.argv"; }

saturation_case
cat > "$CASE/cells" <<'EOF'
scenario-0 held-out row_a P1 12 12
scenario-0 held-out row_a P2 12 12
scenario-1 held-out row_a P1 12 12
scenario-1 held-out row_a P2 12 12
scenario-1 held-out row_b P3 7 12
scenario-1 held-out row_b P4 6 12
scenario-6 held-out row_a P1 0 12
scenario-6 held-out row_a P2 0 12
scenario-6 held-out row_b P3 12 12
scenario-6 held-out row_b P4 11 11
EOF
# The count phase B passes, as `additive-launch.env` declares it.
TRY_SATURATE_AT=
. "$here/additive-launch.env"
check "saturation: additive-launch.env declares the 24 of HW-DR-0112" [ "$TRY_SATURATE_AT" = 24 ]
drive --saturate-at "$TRY_SATURATE_AT"
check "saturation: the driver finishes batch B and exits 0" code_is 0
check "saturation: it writes b.done and done" sh -c "[ -f '$CASE/root/b.done' ] && [ -f '$CASE/root/done' ]"
check "saturation: batch B runs once, from the derived spec <root>/b.spec" sh -c "[ \"\$(grep -c -- '--out $CASE/root/b ' '$CASE/calls.argv')\" = 1 ] && grep -q -- '--spec $CASE/root/b.spec ' '$CASE/calls.argv'"
check "saturation: batch B keeps --repetitions-b" sh -c "grep -- '--out $CASE/root/b ' '$CASE/calls.argv' | grep -q -- '--repetitions 118'"
check "saturation: row_a stopped in both scenarios, so P1 and P2 are excludes on both lines (saw: $(tr '\n' '|' < "$CASE/root/b.spec" 2>/dev/null))" \
    sh -c "[ \"\$(cat '$CASE/root/b.spec')\" = \"\$(printf 'additive scenario-1 final P1 P2\nadditive scenario-6 final P1 P2\njobs 10')\" ]"
check "saturation: the ledger states row_a/scenario-1 stopped at 24 of 24" grep -qx 'stopped row_a/scenario-1 at 24: 24 of 24' "$CASE/root/saturated"
check "saturation: the ledger states row_a/scenario-6 stopped at 0 of 24" grep -qx 'stopped row_a/scenario-6 at 24: 0 of 24' "$CASE/root/saturated"
check "saturation: the ledger states row_b/scenario-1 runs at 13 of 24" grep -qx 'runs row_b/scenario-1: 13 of 24' "$CASE/root/saturated"
check "saturation: a cell of 23 of 23 graded has not reached 24, and runs on" grep -qx 'runs row_b/scenario-6: 23 of 23' "$CASE/root/saturated"
check "saturation: the ledger has a line for each of the four cells, and none for scenario 0, which spec-b does not run" [ "$(wc -l < "$CASE/root/saturated")" = 4 ]
check "saturation: drive.log states each stop" grep -q 'stopped row_a/scenario-6 at 24: 0 of 24' "$CASE/root/drive.log"
check "saturation: batch A is graded once" [ "$(wc -l < "$CASE/cells.calls")" = 1 ]
check "saturation: the grade reads batch A's directory" grep -qx "counts $CASE/root/a" "$CASE/cells.calls"
check "saturation: the spec-b the person wrote is not changed" sh -c "[ \"\$(cat '$CASE/spec-b')\" = \"\$(printf 'additive scenario-1 final\nadditive scenario-6 final\njobs 10')\" ]"

# A restart after the counts were taken grades nothing again, and batch B
# gets the same spec, even when the plan would now select other probes,
# because the driver writes the derived spec once.
cp "$CASE/root/b.spec" "$CASE/b.spec.first"
cp "$CASE/probes" "$CASE/probes.first"
printf 'scenario-1 held-out P9 row_a\n' >> "$CASE/probes"
rm -f "$CASE/root/b.done" "$CASE/root/done"
drive --saturate-at 24
cp "$CASE/probes.first" "$CASE/probes"
check "saturation restart: exits 0" code_is 0
check "saturation restart: batch A is not graded again" [ "$(wc -l < "$CASE/cells.calls")" = 1 ]
check "saturation restart: batch B gets the same spec" cmp -s "$CASE/root/b.spec" "$CASE/b.spec.first"
check "saturation restart: batch B runs from it again" [ "$(grep -c -- "--spec $CASE/root/b.spec " "$CASE/calls.argv")" = 2 ]
# A driver killed after the count and before it wrote the spec derives it
# again from <root>/cells.a, with no new grade.
rm -f "$CASE/root/b.spec" "$CASE/root/b.done" "$CASE/root/done"
drive --saturate-at 24
check "saturation restart: a spec lost after the count is derived again from cells.a, with no grade" \
    sh -c "[ \"\$(wc -l < '$CASE/cells.calls')\" = 1 ] && cmp -s '$CASE/root/b.spec' '$CASE/b.spec.first'"

# Every cell stopped: no batch B invocation, and the log says why.
saturation_case
printf 'additive scenario-1 final\nadditive scenario-6 final P4\n' > "$CASE/spec-b"
cat > "$CASE/cells" <<'EOF'
scenario-1 held-out row_a P1 12 12
scenario-1 held-out row_a P2 12 12
scenario-1 held-out row_b P3 0 12
scenario-1 held-out row_b P4 0 12
scenario-6 held-out row_a P1 0 12
scenario-6 held-out row_a P2 0 12
scenario-6 held-out row_b P3 12 12
scenario-6 held-out row_b P4 12 12
EOF
drive --saturate-at 24
check "saturation all: exits 0" code_is 0
check "saturation all: batch B is never invoked" [ -z "$(b_args)" ]
check "saturation all: b.done and done are written" sh -c "[ -f '$CASE/root/b.done' ] && [ -f '$CASE/root/done' ]"
check "saturation all: the log says batch B has no line left" grep -q 'batch B has no line left' "$CASE/root/drive.log"
check "saturation all: the derived spec holds no line" [ ! -s "$CASE/root/b.spec" ]
check "saturation all: a line that already excludes P4 is dropped once the rest of its probes stopped, and its cell counts batch A's P4" grep -qx 'stopped row_b/scenario-6 at 24: 24 of 24' "$CASE/root/saturated"

# Every cell stopped, in a spec-b that carries comments and a blank line:
# the comments are copied into the derived spec, and a spec of nothing but
# comments is still a batch B with no line, which the driver never runs.
saturation_case
printf '# phase B, batch B\nadditive scenario-1 final\n\n# the second scenario\nadditive scenario-6 final\n' > "$CASE/spec-b"
cat > "$CASE/cells" <<'EOF'
scenario-1 held-out row_a P1 12 12
scenario-1 held-out row_a P2 12 12
scenario-1 held-out row_b P3 0 12
scenario-1 held-out row_b P4 0 12
scenario-6 held-out row_a P1 0 12
scenario-6 held-out row_a P2 0 12
scenario-6 held-out row_b P3 12 12
scenario-6 held-out row_b P4 12 12
EOF
drive --saturate-at 24
check "saturation all, commented: exits 0" code_is 0
check "saturation all, commented: batch B is never invoked, though the derived spec keeps its comments (saw: $(tr '\n' '|' < "$CASE/root/b.spec" 2>/dev/null))" \
    sh -c "[ -z \"\$(grep -- '--out $CASE/root/b ' '$CASE/calls.argv')\" ] && grep -q '^# phase B, batch B\$' '$CASE/root/b.spec'"
check "saturation all, commented: the log says batch B has no line left" grep -q 'batch B has no line left' "$CASE/root/drive.log"

# A line that keeps a probe its author excluded: the stopped row's probes
# are added after that exclude, which stays, and the row that runs on stays
# on the line.
saturation_case
printf 'additive scenario-1 final P5\nadditive scenario-6 final P5\njobs 10\n' > "$CASE/spec-b"
printf 'scenario-1 held-out P5 row_c\nscenario-6 held-out P5 row_c\n' >> "$CASE/probes"
cat > "$CASE/cells" <<'EOF'
scenario-1 held-out row_a P1 12 12
scenario-1 held-out row_a P2 12 12
scenario-1 held-out row_b P3 7 12
scenario-1 held-out row_b P4 6 12
scenario-1 held-out row_c P5 12 12
scenario-6 held-out row_a P1 3 12
scenario-6 held-out row_a P2 4 12
scenario-6 held-out row_b P3 0 12
scenario-6 held-out row_b P4 0 12
scenario-6 held-out row_c P5 0 12
EOF
drive --saturate-at 24
check "saturation exclude: exits 0" code_is 0
check "saturation exclude: each kept line keeps its own exclude P5 and adds only the stopped row's probes (saw: $(tr '\n' '|' < "$CASE/root/b.spec" 2>/dev/null))" \
    sh -c "[ \"\$(cat '$CASE/root/b.spec')\" = \"\$(printf 'additive scenario-1 final P5 P1 P2\nadditive scenario-6 final P5 P3 P4\njobs 10')\" ]"
check "saturation exclude: batch B runs from that spec" grep -q -- "--spec $CASE/root/b.spec " "$CASE/calls.argv"

# Scenario 0 is the anchor, pooled at 145 sessions, and no stop applies to
# it: a spec-b line of scenario 0 runs whole whatever its counts.
saturation_case
printf 'additive scenario-0 final\njobs 10\n' > "$CASE/spec-b"
cat > "$CASE/cells" <<'EOF'
scenario-0 held-out row_a P1 12 12
scenario-0 held-out row_a P2 12 12
scenario-0 held-out row_b P3 0 12
scenario-0 held-out row_b P4 0 12
EOF
drive --saturate-at 24
check "saturation anchor: a scenario-0 line keeps every probe" sh -c "[ \"\$(head -1 '$CASE/root/b.spec')\" = 'additive scenario-0 final' ]"
check "saturation anchor: the ledger says no stop applies to scenario 0" grep -q '^runs row_a/scenario-0: 24 of 24, the anchor' "$CASE/root/saturated"

# A cell graded past the stop count (a row of three probes) is not at the
# count, and runs on, even when none of its sessions passed.
saturation_case
printf 'additive scenario-1 final\njobs 10\n' > "$CASE/spec-b"
printf 'scenario-1 held-out P5 row_b\n' >> "$CASE/probes"
cat > "$CASE/cells" <<'EOF'
scenario-1 held-out row_a P1 12 12
scenario-1 held-out row_a P2 12 12
scenario-1 held-out row_b P3 0 12
scenario-1 held-out row_b P4 0 12
scenario-1 held-out row_b P5 0 12
EOF
drive --saturate-at 24
check "saturation count: a cell of 0 of 36 is not at 24, and runs on" grep -qx 'runs row_b/scenario-1: 0 of 36' "$CASE/root/saturated"
check "saturation count: so only row_a is excluded" sh -c "[ \"\$(head -1 '$CASE/root/b.spec')\" = 'additive scenario-1 final P1 P2' ]"

# A count that fails stops the driver before it spends on batch B.
saturation_case
: > "$CASE/cells"
: > "$CASE/cells-fail"
drive --saturate-at 24
check "saturation fail: a failed count exits 14" code_is 14
check "saturation fail: and batch B is never invoked" [ -z "$(b_args)" ]
check "saturation fail: and no cells.a or b.spec is left to read as a count" sh -c "[ ! -f '$CASE/root/cells.a' ] && [ ! -f '$CASE/root/b.spec' ]"
check "saturation fail: the driver stops at the failed count, and goes on to no step after it" sh -c "grep -q 'the saturation count of batch A failed' '$CASE/root/drive.log' && ! grep -q 'graded batch A once' '$CASE/root/drive.log'"

# A plan of a spec-b line that fails stops the driver too, with no derived
# spec and no ledger to read as a decision.
saturation_case
cp "$CASE/probes" "$CASE/cells.src"
awk '{ print $1, $2, $4, $3, 6, 12 }' "$CASE/cells.src" > "$CASE/cells"
: > "$CASE/probes-fail"
drive --saturate-at 24
check "saturation plan fail: a failed plan exits 14" code_is 14
check "saturation plan fail: and batch B is never invoked" [ -z "$(b_args)" ]
check "saturation plan fail: and no b.spec or saturated is left" sh -c "[ ! -f '$CASE/root/b.spec' ] && [ ! -f '$CASE/root/saturated' ]"
check "saturation plan fail: the log names the line whose plan failed" grep -q 'the plan of line 1 of .* failed, so batch B does not start' "$CASE/root/drive.log"

# Without --saturate-at, batch B runs the spec the person wrote, and the
# seam is never called.
saturation_case
: > "$CASE/cells"
drive
check "saturation off: batch B runs spec-b itself" grep -q -- "--spec $CASE/spec-b " "$CASE/calls.argv"
check "saturation off: and nothing is graded" [ ! -f "$CASE/cells.calls" ]

# ------------------------------------------------- 7b. the seam behind the stop
#
# `tools/probe/cell-passes.sh`, against a stub engine and a stub
# `campaign.sh --assemble`, in a checkout that holds `declared.py` and a
# declaration of one round. The pass counts are the grader's, as
# `probe compare` states them, and never a recorder status.

cells_case() {
    CASE=$(mktemp -d "$fix/cells.XXXXXX")
    export CASE
    mkdir -p "$CASE/co/tools/probe" "$CASE/co/.headwater" "$CASE/batch"
    cp "$here/declared.py" "$CASE/co/tools/probe/declared.py"
    cp "$here/cell-passes.sh" "$CASE/co/tools/probe/cell-passes.sh"
    printf 'sets:\n  held-out: [HW-PROBE-p1, HW-PROBE-p2, HW-PROBE-p3]\nrounds:\n  final:\n    set: held-out\n    rates: [0.9, 0.8]\n    tries: 1\n' \
        > "$CASE/co/.headwater/probe.yml"
    printf 'HW-PROBE-p1 row_a\nHW-PROBE-p2 row_b\nHW-PROBE-p3 row_b\n' > "$CASE/plan"
    printf '1 additive scenario-1 final HW-PROBE-p3\n2 campaign present sufficiency\n' > "$CASE/batch/lines"
    : > "$CASE/engine.calls"
}
cat > "$fix/engine-stub.sh" <<'EOF'
#!/bin/sh
echo "engine $*" >> "$CASE/engine.calls"
[ "$1" = probe ] || exit 1
case $2 in
    plan)
        if [ -f "$CASE/plan-refuses" ]; then
            echo "This run is refused: the set names a probe the corpus does not declare."
            exit 0
        fi
        gone=$(printf '%s\n' "$*" | tr ' ' '\n' | awk 'x { print; x = 0 } $0 == "--exclude" { x = 1 }' | tr '\n' ' ')
        printf 'A additive probe run.\n\n## The selection\n\n'
        awk -v gone="$gone" '
            BEGIN { n = split(gone, o, " "); for (i = 1; i <= n; i++) out[o[i]] = 1 }
            !($1 in out) { printf "- %s (docs/probes/x.md)\n    category: sufficiency\n    action: %s\n", $1, $2 }
        ' "$CASE/plan"
        printf '\n## The read set\n\n- docs/probes/x.md (probe) sha256:0\n'
        ;;
    compare)
        shift 2
        while [ "$1" = --root ]; do shift 2; done
        # The counts are one transcript compared with itself, so the stub
        # grades nothing unless both arguments name the same readable file.
        if [ "$#" != 2 ] || [ "$1" != "$2" ] || [ ! -f "$1" ] || [ ! -r "$1" ]; then
            echo "compare stub: expected one transcript twice, got: $*" >&2
            exit 2
        fi
        cp "$1" "$CASE/transcript.seen"
        cat "$CASE/compare"
        ;;
esac
EOF
chmod +x "$fix/engine-stub.sh"
cat > "$fix/assemble-stub.sh" <<'EOF'
echo "campaign $*" >> "$CASE/campaign.calls"
out=$2
mkdir -p "$out/assembled"
printf 'model: fixture\ntier: additive\narm: scenario-1\n' > "$out/assembled/L1-additive-scenario-1-final.identity.yaml"
printf -- '- probe: HW-PROBE-p1\n  session: s1\n' > "$out/assembled/L1-additive-scenario-1-final.events.yaml"
exit "$(cat "$CASE/assemble-exit" 2>/dev/null || echo 0)"
EOF
cells() {
    (
        cd "$CASE/co" || exit 99
        HEADWATER_CELLS_ENGINE=$fix/engine-stub.sh HEADWATER_CELLS_CAMPAIGN=$fix/assemble-stub.sh \
            timeout 60 sh tools/probe/cell-passes.sh "$@"
    ) > "$CASE/out" 2> "$CASE/err"
    echo $? > "$CASE/code"
}

cells_case
printf 'HW-PROBE-p1: 12/12 against 12/12, +0.0 points [-24.2, +24.2]\nHW-PROBE-p2: no difference: the treated transcript graded no session\npooled: 12/12 against 12/12, +0.0 points [-24.2, +24.2]\n' > "$CASE/compare"
cells counts "$CASE/batch"
check "cells: counts exits 0" code_is 0
check "cells: each probe the line selects gets the grader's satisfied over graded (saw: $(tr '\n' '|' < "$CASE/out"))" \
    sh -c "[ \"\$(cat '$CASE/out')\" = \"\$(printf 'scenario-1 held-out row_a HW-PROBE-p1 12 12\nscenario-1 held-out row_b HW-PROBE-p2 0 0')\" ]"
check "cells: a probe the line excludes is not counted" sh -c "! grep -q p3 '$CASE/out'"
check "cells: the plan is asked with the line's exclude" grep -q -- '--set held-out --exclude HW-PROBE-p3' "$CASE/engine.calls"
check "cells: a line of another tier is neither graded nor planned" sh -c "[ \"\$(grep -c 'probe compare' '$CASE/engine.calls')\" = 1 ] && ! grep -q 'arm present' '$CASE/engine.calls'"
check "cells: the batch is assembled once, by campaign.sh --assemble" grep -qx "campaign --out $CASE/batch --assemble" "$CASE/campaign.calls"
check "cells: the grade reads a transcript of the identity and events blocks" \
    sh -c "grep -q '^## Run identity\$' '$CASE/transcript.seen' && grep -q '^## Events\$' '$CASE/transcript.seen' && grep -q '^arm: scenario-1\$' '$CASE/transcript.seen' && grep -q '^- probe: HW-PROBE-p1\$' '$CASE/transcript.seen'"
check "cells: the transcript is written in the batch, outside the checkout" [ -f "$CASE/batch/assembled/L1-additive-scenario-1-final.transcript.md" ]
# A session the grader refused is in no denominator, and an oracle's
# failure is the grader's not satisfied: the counts are the engine's own.
cells_case
printf 'HW-PROBE-p1: 0/11 against 0/11, +0.0 points [-25.9, +25.9]\nHW-PROBE-p2: 7/12 against 7/12, +0.0 points [-37.0, +37.0]\npooled: 7/23 against 7/23, +0.0 points [-25.0, +25.0]\n' > "$CASE/compare"
cells counts "$CASE/batch"
check "cells: the counts are compare's, a refused session out of the denominator (saw: $(tr '\n' '|' < "$CASE/out"))" \
    sh -c "[ \"\$(cat '$CASE/out')\" = \"\$(printf 'scenario-1 held-out row_a HW-PROBE-p1 0 11\nscenario-1 held-out row_b HW-PROBE-p2 7 12')\" ]"
# A transcript the intake refuses is not a count of zero.
cells_case
printf 'The treated transcript reached no grader, so nothing was compared: no Run identity block\n' > "$CASE/compare"
cells counts "$CASE/batch"
check "cells: a refused transcript exits 4" code_is 4
check "cells: and prints no count" [ ! -s "$CASE/out" ]
# An assembly that fails is not graded.
cells_case
echo 7 > "$CASE/assemble-exit"
: > "$CASE/compare"
cells counts "$CASE/batch"
check "cells: an assembly that exits 7 exits 4" code_is 4
check "cells: and grades nothing" sh -c "! grep -q 'probe compare' '$CASE/engine.calls'"
# probes: the selection of one spec line, with its action.
cells_case
cells probes additive scenario-6 final HW-PROBE-p2
check "cells: probes prints each probe the line selects with its action (saw: $(tr '\n' '|' < "$CASE/out"))" \
    sh -c "[ \"\$(cat '$CASE/out')\" = \"\$(printf 'scenario-6 held-out HW-PROBE-p1 row_a\nscenario-6 held-out HW-PROBE-p3 row_b')\" ]"
check "cells: probes asks the plan for the round's set in the line's arm" grep -q -- 'probe plan --root .* --tier additive --arm scenario-6 --set held-out --exclude HW-PROBE-p2' "$CASE/engine.calls"
cells_case
cells probes campaign present sufficiency
check "cells: probes prints nothing for a line of another tier" sh -c "[ \"\$(cat '$CASE/code')\" = 0 ] && [ ! -s '$CASE/out' ] && [ ! -s '$CASE/engine.calls' ]"
cells_case
: > "$CASE/plan-refuses"
cells probes additive scenario-1 final
check "cells: a plan that selects nothing exits 4" code_is 4
cells_case
cells probes additive scenario-1 screening
check "cells: a round the declaration does not hold exits 4" code_is 4
cells_case
(cd "$CASE/co" && HEADWATER_CELLS_ENGINE=$CASE/none sh tools/probe/cell-passes.sh probes additive scenario-1 final) > /dev/null 2>&1
check "cells: no engine exits 3" [ $? = 3 ]

# ------------------------------------------------- 8. no paid session

check "no case called claude with anything but --version" sh -c "[ ! -s '$fix/paid.log' ]"

[ -n "${HEADWATER_DRIVE_FIXTURES_KEEP:-}" ] || rm -rf "$fix"
echo "campaign-drive-fixtures: $passed passed, $failed failed"
[ "$failed" = 0 ]
