#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
#
# The detached driver loop of a staggered probe campaign (#1659): the canary,
# then batch A, then batch B, each run by `tools/probe/campaign.sh` in slices
# of 500 sessions, again and again until each batch exits 0, under one cap
# for both batches together. A campaign at `--parallel 4` runs for about a
# day, longer than an agent stays alive, so a detached shell runs this loop
# and a person or an agent reads its log.
#
#     sh tools/probe/campaign-drive.sh --root <dir> --pin <sha> --claude-version <v> \
#         --spec-a <file> --spec-b <file> --repetitions-b <n> --model <m> \
#         --session-cents <B> --cap-cents <C> [--parallel <P>] \
#         [--overshoot-cents <M>] [--seed <n>]
#
# Launch it from a worktree detached at the pin, with the engine built there,
# so that it outlives the shell that started it:
#
#     git worktree add --detach <tree> <pin>
#     cd <tree> && setsid nohup sh tools/probe/campaign-drive.sh ... \
#         > <root>/drive.out 2>&1 < /dev/null &
#
# Batch A runs in `<root>/a` and batch B in `<root>/b`. The log
# `<root>/drive.log`, and the markers `canary-recorded`, `canary-failed`,
# `a.done`, `b.done`, `capped` and `done`, are in `<root>`. The spec files
# live outside the checkout, because `campaign.sh` refuses a dirty one. A
# driver started again skips each phase whose marker exists, and derives
# every cap again from the files, so a restart after a kill goes on where
# the last one stopped. The plan this runs is the slice 1 note of #1659,
# "The plan for slice 2".
#
# ## Before every invocation
#
# It checks four things at the start and again before each invocation of
# `campaign.sh`, and invokes nothing when one fails:
#
# - **The stop file.** When `<root>/stop` exists it logs `stop` and exits 10.
#   Every wait sleeps in steps of 60 s at most and checks it between steps,
#   so a stop takes effect within a minute of any wait. A stop does not end
#   an invocation that is running. To end one sooner, kill its process
#   group, which `setsid` gave it: `kill -- -<pgid>`.
# - **The pin.** `HEAD` is `--pin`, the worktree is detached, the pin is on
#   `origin/main`, and the tree is clean. Otherwise it exits 4.
# - **The harness.** `<root>/bin/claude` is a symlink to
#   `$HEADWATER_DRIVE_CLAUDE_VERSIONS/<v>` (default
#   `~/.local/share/claude/versions`), and `<root>/bin` is first on `PATH`
#   for every invocation, so an update of the host's `claude` never reaches
#   the batch. A missing version exits 3. A `claude --version` that does not
#   begin with `<v> `, or a batch directory whose `claude-version` differs
#   from it, exits 4, because a batch whose harness changed half way does
#   not assemble.
# - **The token.** It reads `claudeAiOauth.expiresAt`, in milliseconds, from
#   `.credentials.json` under `$HEADWATER_PROBE_HOST_CONFIG` (default
#   `~/.claude`), which is where `probe-record.sh` copies each session's
#   credentials from. When the token expires in less than 15 minutes, it
#   waits, and reads the file again each minute, logging a line at least
#   every 10 minutes. A missing file or field exits 3. The token must
#   outlast the session (spec 15, "the recorder contract"), and a session
#   cannot refresh it, because the confinement blocks platform.claude.com.
#   **So the host must refresh the token.** Something outside this driver
#   has to run `claude` on the host, such as a session a person keeps
#   alive, because the driver never starts a session of its own. Without
#   that, the driver waits at this check and spends nothing.
#
# ## The phases
#
# 1. The canary: batch A at `--parallel 1 --max-sessions 8`. The driver
#    then checks that 8 sessions recorded with `status` 0 and that each cost
#    at most `B + M`. If not, it writes the reason to `<root>/canary-failed`
#    and exits 11. Otherwise it writes `<root>/canary-recorded` and waits
#    for `<root>/canary-passed`, which a reader writes after reading the 8
#    records for what the driver does not parse: no web call, no outside
#    path, no refused provider fetch, and the intent hook live in every arm
#    but `no-hook`. A canary that a halt cut short asks only for the
#    sessions it still owes.
# 2. Batch A at `--parallel P --max-sessions 500`, until it exits 0.
# 3. Batch B the same, with `--spec <spec-b> --repetitions <n>`.
# 4. `<root>/done`, written only when both `a.done` and `b.done` exist. A
#    driver started with `done` present invokes nothing and exits 0.
#
# Every invocation is `sh $HEADWATER_DRIVE_CAMPAIGN` (default
# `tools/probe/campaign.sh` in the current directory) with `--out --model
# --spec --seed --session-cents --cap-cents --parallel --max-sessions` and,
# on B, `--repetitions`, and nothing else. `CARGO_TARGET_DIR` passes through.
#
# ## The cap
#
# `--cap-cents C` bounds both batches together. `campaign.sh` bounds one
# batch directory, and spends past its cap by at most `--parallel` times the
# largest overshoot of one turn past `--session-cents` (see its header). So
# before every invocation the driver gives it the cap `C - spent(other) -
# p * M`, where `spent` sums every `sessions/*/cost` and
# `sessions/*/spent-before`, `other` is the other batch, `p` is that
# invocation's `--parallel` and `M` is `--overshoot-cents`. When that is at
# or below 0, the batch is capped. `M` defaults to 100 cents: check (a) of
# #1659 measured an overshoot of 14.6 cents on a first turn at B = 5, and
# nobody has measured one at B = 300, so 100 is a margin and not a
# measurement. After each invocation, a total past `C` logs the figures and
# exits 7. That is the alarm, not the mechanism.
#
# ## What each exit of campaign.sh does
#
# `campaign.sh` exits 7 both when the cap or the ceiling refused a job,
# which is expected at the end of a batch, and when a recorder failed, which
# is not. So the driver reads the files under the batch directory: a job
# that is not recorded, was started by this invocation, and is not the job
# a halt left, but has no `slice/refused/<name>`, is a recorder failure.
#
# - 0: the batch is done.
# - 9 without `slice/halt`: the bound was reached, so invoke again at once.
# - 9, or 7 with refusals only, with `slice/halt`: a halt, such as a usage
#   limit or an expired token. Sleep `300 * 2^(k-1)` s, at most 3600, where
#   `k` counts the halts in a row since one recorded a session, and invoke
#   again. The 13th such halt exits 12.
# - 7 with refusals only: invoke once more, because a refusal can be a
#   reservation for a session in flight that has since finished. A second
#   refusal-only exit 7 that recorded nothing more writes `<root>/capped`
#   and exits 8, and writes no `done`.
# - 7 with a recorder failure: log the sessions and exit 7. The driver does
#   not run them again, because a re-run spends again.
# - 4, 5, 6 and any other code: stop with that code.
#
# Each invocation logs one line, on standard output and in `drive.log`:
#
#     <time> <canary|a|b> exit=<n> recorded=<r>/<total> new=<d> cents=<spent(dir)> all=<spent(a)+spent(b)>/<C> [halt|bound|capped|refused-only]
#
# `HEADWATER_DRIVE_NOW` and `HEADWATER_DRIVE_SLEEP` name the commands that
# print the epoch seconds and sleep, `date +%s` and `sleep` by default, so
# the fixtures (`campaign-drive-fixtures.sh`) need no real time.
#
# It needs `git`, `jq` and `awk`.
#
# ## Exit status
#
#   0   both batches exited 0, and `done` is written
#   1   `campaign.sh` exited 1
#   2   a usage error
#   3   a tool, the harness version or the token is missing, or campaign.sh
#       exited 3
#   4   the pin or the harness is not the one given, or campaign.sh exited 4
#   5   campaign.sh refused the plan
#   6   campaign.sh refused the output directory
#   7   a recorder failed, or both batches spent past `--cap-cents`
#   8   a batch is capped: `capped` is written, and `done` is not
#   10  `<root>/stop` exists
#   11  the canary failed: `canary-failed` holds the reason
#   12  12 halts in a row recorded nothing, and the 13th stopped the driver

set -u

usage() {
    echo "usage: sh tools/probe/campaign-drive.sh --root <dir> --pin <sha> --claude-version <v> --spec-a <file> --spec-b <file> --repetitions-b <n> --model <m> --session-cents <B> --cap-cents <C> [--parallel <P>] [--overshoot-cents <M>] [--seed <n>]" >&2
    exit 2
}

root= pin= version= spec_a= spec_b= reps_b= model= budget= cap= parallel=4 margin=100 seed=0
while [ $# -gt 0 ]; do
    [ $# -ge 2 ] || usage
    case $1 in
        --root) root=$2 ;;
        --pin) pin=$2 ;;
        --claude-version) version=$2 ;;
        --spec-a) spec_a=$2 ;;
        --spec-b) spec_b=$2 ;;
        --repetitions-b) reps_b=$2 ;;
        --model) model=$2 ;;
        --session-cents) budget=$2 ;;
        --cap-cents) cap=$2 ;;
        --parallel) parallel=$2 ;;
        --overshoot-cents) margin=$2 ;;
        --seed) seed=$2 ;;
        *) usage ;;
    esac
    shift 2
done
for value in "$root" "$pin" "$version" "$spec_a" "$spec_b" "$model"; do
    [ -n "$value" ] || usage
done
positive() {
    case $1 in '' | *[!0-9]* | 0*) return 1 ;; esac
}
for value in "$reps_b" "$budget" "$cap" "$parallel" "$margin"; do
    positive "$value" || usage
done
case $seed in '' | *[!0-9]*) usage ;; esac
for spec in "$spec_a" "$spec_b"; do
    [ -f "$spec" ] || { echo "campaign-drive: no spec file $spec." >&2; exit 2; }
done
for tool in git jq awk; do
    command -v "$tool" >/dev/null 2>&1 || { echo "campaign-drive: $tool is missing." >&2; exit 3; }
done

campaign=${HEADWATER_DRIVE_CAMPAIGN:-$PWD/tools/probe/campaign.sh}
versions=${HEADWATER_DRIVE_CLAUDE_VERSIONS:-$HOME/.local/share/claude/versions}
credentials=${HEADWATER_PROBE_HOST_CONFIG:-$HOME/.claude}/.credentials.json
now_cmd=${HEADWATER_DRIVE_NOW:-date +%s}
sleep_cmd=${HEADWATER_DRIVE_SLEEP:-sleep}
mkdir -p "$root" || exit 2
root=$(cd "$root" && pwd)

now() { $now_cmd; }
stamp() { date -u -d "@$(now)" +%Y-%m-%dT%H:%M:%SZ; }
say() { printf '%s %s\n' "$(stamp)" "$1" | tee -a "$root/drive.log"; }

# ------------------------------------------------------------ the files

# spent <dir>: every cost and every earlier attempt's cost under the batch.
spent() {
    find "$1/sessions" -mindepth 2 -maxdepth 2 -type f \( -name cost -o -name spent-before \) -exec cat {} + 2>/dev/null \
        | awk '{ s += $1 } END { print s + 0 }'
}

# classify <dir>: one pass over the jobs of the batch, printing
# `<recorded> <total> <refused> <failed> <halt> [<failed names>...]`. A job
# is recorded at `status` 0, refused with `slice/refused/<name>`, left when
# this invocation did not start it or when it is the job a halt left, and
# failed otherwise. `slice/started/<name>` matters: a job a halt left with
# `status` 10 in an earlier invocation, and this one did not start, still
# carries that status.
classify() {
    [ -f "$1/jobs" ] || { echo "0 0 0 0 0"; return; }
    awk -v d="$1" '
        function exists(p,   line, r) { r = (getline line < p); close(p); return r >= 0 }
        function first(p,   line) { line = ""; if ((getline line < p) <= 0) line = ""; close(p); return line }
        BEGIN { halt = exists(d "/slice/halt") }
        NF {
            name = $1; total++
            status = first(d "/sessions/" name "/status")
            if (status == "0") recorded++
            else if (exists(d "/slice/refused/" name)) refused++
            else if (!exists(d "/slice/started/" name)) left++
            else if (status == "10" && halt) left++
            else { failed++; names = names " " name }
        }
        END { printf "%d %d %d %d %d%s\n", recorded, total, refused, failed, halt, names }
    ' "$1/jobs"
}

recorded() { set -- $(classify "$1"); echo "$1"; }

# ------------------------------------------------------------ the waits

stop_check() {
    if [ -f "$root/stop" ]; then
        say "stop"
        exit 10
    fi
}

# pause <seconds>: sleep in steps of 60 s at most, checking the stop file.
pause() {
    left=$1
    while [ "$left" -gt 0 ]; do
        stop_check
        step=$left
        [ "$step" -gt 60 ] && step=60
        $sleep_cmd "$step"
        left=$((left - step))
    done
    stop_check
}

token_wait() {
    logged=
    while :; do
        stop_check
        [ -f "$credentials" ] || { echo "campaign-drive: no credentials file $credentials." >&2; exit 3; }
        expires=$(jq -r '.claudeAiOauth.expiresAt // empty' "$credentials" 2>/dev/null)
        positive "$expires" || { echo "campaign-drive: $credentials states no claudeAiOauth.expiresAt." >&2; exit 3; }
        t=$(now)
        remaining=$((expires - t * 1000))
        [ "$remaining" -ge 900000 ] && return 0
        if [ -z "$logged" ] || [ $((t - logged)) -ge 600 ]; then
            say "wait token expires in $((remaining / 60000)) min; waiting for the host to refresh it"
            logged=$t
        fi
        pause 60
    done
}

canary_wait() {
    logged=
    while [ ! -f "$root/canary-passed" ]; do
        t=$(now)
        if [ -z "$logged" ] || [ $((t - logged)) -ge 600 ]; then
            say "wait canary-passed: read the 8 records under $root/a, then write $root/canary-passed"
            logged=$t
        fi
        pause 60
    done
}

# ------------------------------------------------------------ the checks

pin_check() {
    head=$(git rev-parse HEAD 2>/dev/null)
    if [ "$head" != "$pin" ]; then
        echo "campaign-drive: HEAD is ${head:-unknown}, not the pin $pin." >&2
        exit 4
    fi
    if git symbolic-ref -q HEAD >/dev/null 2>&1; then
        echo "campaign-drive: the worktree is on a branch; detach it at the pin." >&2
        exit 4
    fi
    if ! git merge-base --is-ancestor "$pin" origin/main 2>/dev/null; then
        echo "campaign-drive: the pin $pin is not on origin/main." >&2
        exit 4
    fi
    if [ -n "$(git status --porcelain 2>/dev/null)" ]; then
        echo "campaign-drive: the tree is dirty." >&2
        exit 4
    fi
}

harness_check() {
    if [ ! -e "$versions/$version" ]; then
        echo "campaign-drive: no claude $version in $versions." >&2
        exit 3
    fi
    mkdir -p "$root/bin"
    ln -sfn "$versions/$version" "$root/bin/claude"
    seen=$(claude --version 2>/dev/null | head -1)
    case $seen in
        "$version "*) ;;
        *) echo "campaign-drive: $root/bin/claude reports '$seen', not $version." >&2; exit 4 ;;
    esac
    for batch in a b; do
        if [ -f "$root/$batch/claude-version" ] && [ "$(cat "$root/$batch/claude-version")" != "$seen" ]; then
            echo "campaign-drive: $root/$batch was started with $(cat "$root/$batch/claude-version"), not $seen." >&2
            exit 4
        fi
    done
}

checks() {
    stop_check
    pin_check
    harness_check
    token_wait
}

write_capped() {
    set -- $(classify "$root/a")
    ra=$1 ta=$2
    set -- $(classify "$root/b")
    {
        echo "spent-a: $(spent "$root/a")"
        echo "spent-b: $(spent "$root/b")"
        echo "cap-cents: $cap"
        echo "recorded: $ra of $ta in a, $1 of $2 in b"
    } > "$root/capped"
}

# ------------------------------------------------------------ one batch

# run_batch <phase> <dir> <spec> <parallel> <max-sessions> [<repetitions>]
# invokes campaign.sh until the batch exits 0 (the canary: until it holds
# 8 recorded sessions), and returns, or exits with the driver's code.
run_batch() {
    phase=$1 dir=$2 spec=$3 p=$4 max=$5 reps=${6:-}
    other=b
    [ "$dir" = b ] && other=a
    halts=0
    retried=0
    while :; do
        checks
        given=$((cap - $(spent "$root/$other") - p * margin))
        if [ "$given" -le 0 ]; then
            write_capped
            say "$phase capped: $cap less $(spent "$root/$other") spent by $other less $p x $margin leaves $given"
            exit 8
        fi
        before=$(recorded "$root/$dir")
        sessions=$max
        if [ "$phase" = canary ]; then
            [ "$before" -ge 8 ] && return 0
            sessions=$((8 - before))
        fi
        sh "$campaign" --out "$root/$dir" --model "$model" --spec "$spec" --seed "$seed" \
            --session-cents "$budget" --cap-cents "$given" --parallel "$p" --max-sessions "$sessions" \
            ${reps:+--repetitions "$reps"} >&2
        code=$?
        set -- $(classify "$root/$dir")
        rec=$1 total=$2 failed=$4 halt=$5
        shift 5
        names=$*
        new=$((rec - before))
        mine=$(spent "$root/$dir")
        all=$(($(spent "$root/a") + $(spent "$root/b")))
        tag=
        action=
        case $code in
            0) action=done ;;
            9) if [ "$halt" = 1 ]; then action=halt; else action=bound; fi ;;
            7)
                if [ "$failed" -gt 0 ]; then
                    action=failed
                elif [ "$halt" = 1 ]; then
                    action=halt
                elif [ "$retried" = 1 ] && [ "$new" = 0 ]; then
                    action=capped
                else
                    action=refused-only
                fi
                ;;
            *) action=stop ;;
        esac
        case $action in halt | bound | capped | refused-only) tag=" $action" ;; esac
        say "$phase exit=$code recorded=$rec/$total new=$new cents=$mine all=$all/$cap$tag"
        if [ "$all" -gt "$cap" ]; then
            say "overspend: spent $all of $cap cents (a $(spent "$root/a"), b $(spent "$root/b"))"
            exit 7
        fi
        [ "$action" = refused-only ] || retried=0
        case $action in
            done) return 0 ;;
            bound)
                halts=0
                [ "$phase" = canary ] && [ "$rec" -ge 8 ] && return 0
                ;;
            halt)
                if [ "$new" -gt 0 ]; then halts=1; else halts=$((halts + 1)); fi
                if [ "$halts" -gt 12 ]; then
                    say "$phase: $halts halts in a row recorded nothing; stopping"
                    exit 12
                fi
                wait=$((300 * (1 << (halts - 1))))
                [ "$wait" -gt 3600 ] && wait=3600
                say "halt $halts; sleeping $wait s"
                pause "$wait"
                ;;
            refused-only) retried=1 ;;
            capped)
                write_capped
                say "$phase capped: a re-try after a refusal recorded nothing"
                exit 8
                ;;
            failed)
                say "$phase: a recorder failed for$names; read their record.err, nothing is run again"
                exit 7
                ;;
            stop) exit "$code" ;;
        esac
    done
}

# ------------------------------------------------------------ the phases

if [ -f "$root/done" ]; then
    say "done is present; nothing to run"
    exit 0
fi
if [ -f "$root/canary-failed" ]; then
    say "canary-failed is present: $(head -1 "$root/canary-failed")"
    exit 11
fi
PATH=$root/bin:$PATH
export PATH
checks

if [ ! -f "$root/canary-recorded" ]; then
    run_batch canary a "$spec_a" 1 8
    reason=$(awk -v d="$root/a" -v most=$((budget + margin)) '
        function first(p,   line) { line = ""; if ((getline line < p) <= 0) line = ""; close(p); return line }
        NF {
            if (first(d "/sessions/" $1 "/status") != "0") next
            n++
            cost = first(d "/sessions/" $1 "/cost")
            if (cost == "" || cost + 0 > most) bad = bad " " $1 " cost " (cost == "" ? "none" : cost) " above " most ";"
        }
        END {
            if (n != 8) printf "%d sessions recorded with status 0, not 8;", n
            printf "%s", bad
        }
    ' "$root/a/jobs")
    if [ -n "$reason" ]; then
        printf '%s\n' "$reason" > "$root/canary-failed"
        say "canary failed: $reason"
        exit 11
    fi
    : > "$root/canary-recorded"
    say "canary recorded"
fi
canary_wait
if [ ! -f "$root/a.done" ]; then
    run_batch a a "$spec_a" "$parallel" 500
    : > "$root/a.done"
fi
if [ ! -f "$root/b.done" ]; then
    run_batch b b "$spec_b" "$parallel" 500 "$reps_b"
    : > "$root/b.done"
fi
if [ -f "$root/a.done" ] && [ -f "$root/b.done" ]; then
    : > "$root/done"
    say "done"
fi
exit 0
