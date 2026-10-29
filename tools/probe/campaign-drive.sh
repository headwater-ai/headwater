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
#         --spec-a <file> [--spec-b <file> --repetitions-b <n> [--saturate-at <n>]] --model <m> \
#         --session-cents <B> --cap-cents <C> [--parallel <P> | --ramp <P1,P2,...>] \
#         [--overshoot-cents <M>] [--seed <n>] [--on-canary <cmd>] [--on-done <cmd>]
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
# It checks five things at the start and again before each invocation of
# `campaign.sh`, and invokes nothing when one fails:
#
# - **The stop file.** When `<root>/stop` exists it logs `stop` and exits 10.
#   Every wait sleeps in steps of 60 s at most and checks it between steps,
#   so a stop takes effect within a minute of any wait. A stop does not end
#   an invocation that is running. To end one sooner, kill its process
#   group, which `setsid` gave it: `kill -- -<pgid>`.
# - **The pin.** `HEAD` is `--pin`, the worktree is detached, the pin is on
#   `origin/main` or on a branch under `origin/tune/`, and the tree is clean.
#   Otherwise it exits 4. A variant of the tuning round is screened before it
#   merges, so its commit is on its pushed `tune/` branch and not yet on
#   `origin/main` (#1777). Either way the pin is a commit the remote holds.
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
# - **The usage gate** (#1876). It reads the freshest figure of the 7-day
#   window of the subscription from two sources, and takes the newer:
#   the snapshot of `$HEADWATER_USAGE_DIR` (default
#   `~/.cache/claude-usage/sessions`) with the highest `last_activity`, read
#   at `.seven_day.used_percentage` and `.seven_day.resets_at`; and the last
#   `rate_limit_event` of the newest `<root>/*/sessions/*/raw.jsonl` that
#   holds one, dated by the file's mtime, read at
#   `.rate_limit_info.unifiedWindows.seven_day`, or at `.rate_limit_info`
#   where its `rateLimitType` is `seven_day`, with `utilization` a fraction.
#   A figure whose reset is past belongs to a closed window and is no
#   figure. The threshold is `HEADWATER_DRIVE_USAGE_STOP` percent, and
#   `HEADWATER_DRIVE_USAGE_LATE` percent when the reset is less than
#   `HEADWATER_DRIVE_USAGE_LATE_HOURS` hours away. `tools/probe/try.sh` sets
#   all three from `additive-launch.env`, and the driver holds no default:
#   with any of them unset it logs `usage gate off` once and gates nothing.
#   At or above the threshold, it writes `<root>/usage-gated`, logs one
#   line `usage-gated: 7-day at <p>% (from <source>), threshold <t>%,
#   resets <time>`, and exits 13. Below it, it removes `<root>/usage-gated`.
#   With no figure it logs `usage unknown`, at most every 10 minutes, and
#   goes on: the harness's own limit still stops a session at 100 percent.
#   The gate acts per session, not only per invocation, because one
#   invocation can run a whole confirm of 145 sessions. So while
#   `campaign.sh` runs, the driver reads the gate again every 60 s
#   (`HEADWATER_DRIVE_POLL`, default the sleep command). When it trips, the
#   driver writes `<dir>/slice/halt`, which `campaign.sh` reads before it
#   starts each job, lets the jobs in flight finish and record, and exits 13
#   with no halt backoff. `tools/probe/campaign.sh` is not changed, because
#   it is a base invalidator of `.headwater/probe.yml`.
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
#
#    `--on-canary <cmd>` runs `sh -c <cmd>` once, in the foreground, right
#    after the driver writes `canary-recorded` and before it waits, so that
#    a reader such as `sh tools/probe/try.sh canary` can write
#    `canary-passed` with nobody awake (#1821). The driver logs its exit
#    status and goes on to the wait whatever the status is, because the
#    wait reads `canary-passed` and nothing else. A driver started again
#    with `canary-recorded` present does not run it again.
# 2. Batch A at `--parallel P --max-sessions 500`, until it exits 0.
# 3. Batch B the same, with `--spec <spec-b> --repetitions <n>`.
# 4. `<root>/done`, written only when `a.done` exists, and `b.done` where
#    the driver runs batch B. A driver started with `done` present invokes
#    nothing and exits 0. `--on-done <cmd>` runs `sh -c <cmd>` once, right
#    after the driver writes `done`, and the driver logs its status and
#    exits 0. A driver started with `done` present runs neither hook.
#
# Every invocation is `sh $HEADWATER_DRIVE_CAMPAIGN` (default
# `tools/probe/campaign.sh` in the current directory) with `--out --model
# --spec --seed --session-cents --cap-cents --parallel --max-sessions` and,
# on B, `--repetitions`, and nothing else. `CARGO_TARGET_DIR` passes through.
#
# ## The cap
#
# `--cap-cents C` bounds every batch directory under `<root>` together.
# `campaign.sh` bounds one batch directory, and spends past its cap by at
# most `--parallel` times the largest overshoot of one turn past
# `--session-cents` (see its header). So before every invocation the driver
# gives it the cap `C - spent(other) - p * M`, where `spent` sums every
# `sessions/*/cost` and `sessions/*/spent-before`, `other` is every other
# directory of `<root>` that holds `sessions/`, `p` is that invocation's
# `--parallel` and `M` is `--overshoot-cents`. When that is at or below 0,
# the batch is capped.
#
# **A restart does not reset the count (#1775).** A batch that must start
# again in a new directory, on a new pin or a harness that moved, keeps the
# same `<root>`. Move the old batch directory aside under it, such as
# `<root>/a` to `<root>/a.attempt-1`, and start the driver again. Its
# `other` then counts what the earlier attempt spent, so every attempt
# together spends at most `C`, past it only by the overshoot margin. The
# owner ruled this for the screening round of the additive campaign, at a
# `C` of 70000. A batch directory outside `<root>` is not counted.
#
# Batch B is optional: without `--spec-b` and `--repetitions-b`, the driver
# runs the canary and batch A alone, and writes `done` after `a.done`. The
# screening round runs this way. `M` defaults to 100 cents: check (a) of
# #1659 measured an overshoot of 14.6 cents on a first turn at B = 5, and
# nobody has measured one at B = 300, so 100 is a margin and not a
# measurement. After each invocation, a total past `C` logs the figures and
# exits 7. That is the alarm, not the mechanism.
#
# ## The ramp
#
# `--parallel <P>` (default 4) is the parallel count of every invocation of
# batch A and batch B. `--ramp <P1,P2,...>` gives steps in its place, each
# above the one before, and each batch starts at the first. After a slice
# that ended on the bound, recorded at least one session, had no halt, no
# refused job, and for every session it recorded a cost inside the range of
# the canary's 8 costs (`<root>/canary-range`) and a `record.md` stating
# that the proxy refused no provider fetch, the next slice runs one step
# higher, never past the last. A halt lowers it one step, never below the
# first. Anything else holds it. The cap margin is always `p * M` for the
# `p` of the invocation it is computed for, so a higher step leaves a larger
# margin. The provider count is a sentence of `record.md`, not a field, so a
# record that words it otherwise holds the ramp rather than raising it. A
# driver started again starts each batch at the first step. The canary runs
# at `--parallel 1` whatever the ramp says.
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
#   not run them again, because a re-run spends again. A job that
#   `campaign.sh --unrecorded` marked with `<dir>/unrecorded/<name>` is not a
#   recorder failure: a person accepted it as a counted non-record (#1659),
#   so the driver counts it as neither failed nor left, and a status 5 with
#   no marker still exits 7.
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
#   0   every batch it runs exited 0, and `done` is written
#   1   `campaign.sh` exited 1
#   2   a usage error
#   3   a tool, the harness version or the token is missing, or campaign.sh
#       exited 3
#   4   the pin or the harness is not the one given, or campaign.sh exited 4
#   5   campaign.sh exited 5: it refused the plan, a raise of
#       `--repetitions`, or a line of a round nobody approved
#   6   campaign.sh refused the output directory
#   7   a recorder failed on a session no `<dir>/unrecorded/<name>` marks, or
#       the batch directories under `<root>` spent past `--cap-cents`
#   8   a batch is capped: `capped` is written, and `done` is not
#   10  `<root>/stop` exists
#   11  the canary failed: `canary-failed` holds the reason
#   12  12 halts in a row recorded nothing, and the 13th stopped the driver
#   13  the usage gate tripped: `usage-gated` holds the figure, and
#       `tools/probe/try.sh relaunch` goes on below the threshold

set -u

usage() {
    echo "usage: sh tools/probe/campaign-drive.sh --root <dir> --pin <sha> --claude-version <v> --spec-a <file> [--spec-b <file> --repetitions-b <n> [--saturate-at <n>]] --model <m> --session-cents <B> --cap-cents <C> [--parallel <P> | --ramp <P1,P2,...>] [--overshoot-cents <M>] [--seed <n>] [--on-canary <cmd>] [--on-done <cmd>]" >&2
    exit 2
}

root= pin= version= spec_a= spec_b= reps_b= saturate= saturate_given= model= budget= cap= parallel=4 parallel_given= ramp= margin=100 seed=0 on_canary= on_done=
while [ $# -gt 0 ]; do
    [ $# -ge 2 ] || usage
    case $1 in
        --root) root=$2 ;;
        --pin) pin=$2 ;;
        --claude-version) version=$2 ;;
        --spec-a) spec_a=$2 ;;
        --spec-b) spec_b=$2 ;;
        --repetitions-b) reps_b=$2 ;;
        --saturate-at) saturate=$2 saturate_given=1 ;;
        --model) model=$2 ;;
        --session-cents) budget=$2 ;;
        --cap-cents) cap=$2 ;;
        --parallel) parallel=$2 parallel_given=1 ;;
        --ramp) ramp=$2 ;;
        --overshoot-cents) margin=$2 ;;
        --seed) seed=$2 ;;
        --on-canary) on_canary=$2 ;;
        --on-done) on_done=$2 ;;
        *) usage ;;
    esac
    shift 2
done
for value in "$root" "$pin" "$version" "$spec_a" "$model"; do
    [ -n "$value" ] || usage
done
positive() {
    case $1 in '' | *[!0-9]* | 0*) return 1 ;; esac
}
for value in "$budget" "$cap" "$parallel" "$margin"; do
    positive "$value" || usage
done
# Batch B is optional (#1775): `--spec-b` and `--repetitions-b` come
# together or not at all, and without them the driver runs batch A alone.
if [ -n "$spec_b" ] || [ -n "$reps_b" ]; then
    [ -n "$spec_b" ] || usage
    positive "$reps_b" || usage
fi
# The saturation stop (#1875) stops cells of batch B, so it needs one, and
# the driver holds no default for the count.
if [ -n "$saturate_given" ]; then
    [ -n "$spec_b" ] || usage
    positive "$saturate" || usage
fi
case $seed in '' | *[!0-9]*) usage ;; esac
# The ramp: steps of --parallel, each above the one before.
steps=0
if [ -n "$ramp" ]; then
    if [ -n "$parallel_given" ]; then
        echo "campaign-drive: give --parallel or --ramp, not both." >&2
        exit 2
    fi
    case $ramp in ,* | *, | *,,*) usage ;; esac
    last=0
    for value in $(printf '%s' "$ramp" | tr ',' ' '); do
        positive "$value" || usage
        if [ "$value" -le "$last" ]; then
            echo "campaign-drive: each step of --ramp must be above the one before." >&2
            exit 2
        fi
        last=$value
        steps=$((steps + 1))
    done
    [ "$steps" -gt 0 ] || usage
    parallel=$(printf '%s' "$ramp" | cut -d, -f1)
fi
for spec in "$spec_a" ${spec_b:+"$spec_b"}; do
    [ -f "$spec" ] || { echo "campaign-drive: no spec file $spec." >&2; exit 2; }
done
for tool in git jq awk; do
    command -v "$tool" >/dev/null 2>&1 || { echo "campaign-drive: $tool is missing." >&2; exit 3; }
done

campaign=${HEADWATER_DRIVE_CAMPAIGN:-$PWD/tools/probe/campaign.sh}
cells=${HEADWATER_DRIVE_CELLS:-$PWD/tools/probe/cell-passes.sh}
versions=${HEADWATER_DRIVE_CLAUDE_VERSIONS:-$HOME/.local/share/claude/versions}
credentials=${HEADWATER_PROBE_HOST_CONFIG:-$HOME/.claude}/.credentials.json
now_cmd=${HEADWATER_DRIVE_NOW:-date +%s}
sleep_cmd=${HEADWATER_DRIVE_SLEEP:-sleep}
poll_cmd=${HEADWATER_DRIVE_POLL:-$sleep_cmd}
usage_dir=${HEADWATER_USAGE_DIR:-$HOME/.cache/claude-usage/sessions}
usage_stop=${HEADWATER_DRIVE_USAGE_STOP:-}
usage_late=${HEADWATER_DRIVE_USAGE_LATE:-}
usage_hours=${HEADWATER_DRIVE_USAGE_LATE_HOURS:-}
usage_on=
if [ -n "$usage_stop" ] && [ -n "$usage_late" ] && [ -n "$usage_hours" ]; then
    for value in "$usage_stop" "$usage_late" "$usage_hours"; do
        positive "$value" || { echo "campaign-drive: a usage threshold is not a whole number: $usage_stop $usage_late $usage_hours." >&2; exit 2; }
    done
    usage_on=1
fi
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

# spent_all: what every batch directory under `<root>` spent, `a` and `b`
# and every earlier attempt moved aside beside them (#1775). A batch
# directory is a directory of `<root>` that holds `sessions/`, whatever its
# name: `*` lists no name that starts with a dot, so the two other patterns
# list those, less `.` and `..` (verify round 1 of #1775).
spent_all() {
    spent_all_sum=0
    for spent_all_dir in "$root"/*/ "$root"/.[!.]*/ "$root"/..?*/; do
        [ -d "${spent_all_dir}sessions" ] || continue
        spent_all_sum=$((spent_all_sum + $(spent "${spent_all_dir%/}")))
    done
    echo "$spent_all_sum"
}

# classify <dir>: one pass over the jobs of the batch, printing
# `<recorded> <total> <refused> <failed> <halt> <unrecorded> [<failed
# names>...]`. A job is recorded at `status` 0, unrecorded with
# `unrecorded/<name>`, refused with `slice/refused/<name>`, left when this
# invocation did not start it or when it is the job a halt left, and failed
# otherwise. `slice/started/<name>` matters: a job a halt left with `status`
# 10 in an earlier invocation, and this one did not start, still carries that
# status. An unrecorded job is a spent session a person accepted with
# `campaign.sh --unrecorded` as a counted non-record (#1659). It is neither
# failed nor left, and `campaign.sh` never draws it again.
classify() {
    [ -f "$1/jobs" ] || { echo "0 0 0 0 0 0"; return; }
    awk -v d="$1" '
        function exists(p,   line, r) { r = (getline line < p); close(p); return r >= 0 }
        function first(p,   line) { line = ""; if ((getline line < p) <= 0) line = ""; close(p); return line }
        BEGIN { halt = exists(d "/slice/halt") }
        NF {
            name = $1; total++
            status = first(d "/sessions/" name "/status")
            if (status == "0") recorded++
            else if (exists(d "/unrecorded/" name)) unrecorded++
            else if (exists(d "/slice/refused/" name)) refused++
            else if (!exists(d "/slice/started/" name)) left++
            else if (status == "10" && halt) left++
            else { failed++; names = names " " name }
        }
        END { printf "%d %d %d %d %d %d%s\n", recorded, total, refused, failed, halt, unrecorded, names }
    ' "$1/jobs"
}

recorded() { set -- $(classify "$1"); echo "$1"; }

# clean <dir>: whether the slice just run lets the ramp rise. It needs a
# slice that recorded at least one session, and for every session it
# recorded a cost inside the canary's range (`<root>/canary-range`) and a
# record that states the proxy refused no provider fetch. That count is a
# sentence of `record.md` and not a field, so a record that does not state
# it in the words `probe-record.sh` prints today holds the ramp: the test
# fails closed, and a reworded record costs a rise, never a cap.
clean() {
    [ -s "$root/canary-range" ] || return 1
    [ -f "$1/jobs" ] || return 1
    set -- "$1" $(cat "$root/canary-range")
    awk -v d="$1" -v lo="$2" -v hi="$3" '
        function exists(p,   line, r) { r = (getline line < p); close(p); return r >= 0 }
        function first(p,   line) { line = ""; if ((getline line < p) <= 0) line = ""; close(p); return line }
        function quiet(p,   line, ok) {
            ok = 0
            while ((getline line < p) > 0)
                if (line ~ /It refused (no request|0 requests?) of that kind/) ok = 1
            close(p)
            return ok
        }
        NF {
            name = $1
            if (!exists(d "/slice/started/" name)) next
            # A job `--unrecorded` marked has status 5, so it is skipped here.
            if (first(d "/sessions/" name "/status") != "0") next
            n++
            cost = first(d "/sessions/" name "/cost")
            if (cost == "" || cost + 0 < lo + 0 || cost + 0 > hi + 0) bad++
            else if (!quiet(d "/sessions/" name "/record.md")) bad++
        }
        END { exit !(n > 0 && bad == 0) }
    ' "$1/jobs"
}

# ramp_to <step>: set the step and the parallel count it names.
ramp_to() {
    step=$1
    p=$(printf '%s' "$ramp" | cut -d, -f"$step")
}

# ------------------------------------------------------------ the waits

stop_check() {
    if [ -f "$root/stop" ]; then
        say "stop"
        exit 10
    fi
}

# pause <seconds>: sleep in steps of 60 s at most, checking the stop file.
# The shell has no local variables, so every function of this script names
# its own: a `step` here once overwrote the ramp's.
pause() {
    pause_left=$1
    while [ "$pause_left" -gt 0 ]; do
        stop_check
        pause_chunk=$pause_left
        [ "$pause_chunk" -gt 60 ] && pause_chunk=60
        $sleep_cmd "$pause_chunk"
        pause_left=$((pause_left - pause_chunk))
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

# usage_figure: print `<percent> <resets_at> <source>` for the freshest
# figure of the 7-day window that is not past its reset, or nothing. The
# header says where it reads.
usage_figure() {
    uf_t=$(now)
    uf_at=-1
    uf_out=
    for uf_file in "$usage_dir"/*.json; do
        [ -f "$uf_file" ] || continue
        uf_row=$(jq -r '.last_activity as $a | .seven_day | select(type == "object")
            | select((.used_percentage | type) == "number" and .resets_at != null and ($a | type) == "number")
            | "\($a | floor) \(.used_percentage) \(.resets_at | tonumber | floor)"' "$uf_file" 2>/dev/null) || continue
        set -- $uf_row
        [ $# = 3 ] || continue
        [ "$3" -gt "$uf_t" ] 2>/dev/null || continue
        [ "$1" -gt "$uf_at" ] 2>/dev/null || continue
        uf_at=$1
        uf_out="$2 $3 $uf_file"
    done
    # The transcripts, newest first. A transcript that holds no 7-day figure
    # yet, such as a session that has just started, gives way to the next.
    uf_list=$(find "$root" -mindepth 4 -maxdepth 4 -type f -path "$root/*/sessions/*/raw.jsonl" -printf '%T@ %p\n' 2>/dev/null | sort -rn | head -n 10)
    while read -r uf_mtime uf_file; do
        [ -n "$uf_file" ] || continue
        uf_mtime=${uf_mtime%.*}
        uf_row=$(grep -F rate_limit_event "$uf_file" 2>/dev/null | jq -R -r 'fromjson? | objects
            | select(.type == "rate_limit_event") | .rate_limit_info | objects
            | if (.unifiedWindows.seven_day | type) == "object" then .unifiedWindows.seven_day
              elif .rateLimitType == "seven_day" then . else empty end
            | select((.utilization | type) == "number" and .resetsAt != null)
            | "\(.utilization * 100) \(.resetsAt | tonumber | floor)"' 2>/dev/null | tail -n 1)
        set -- $uf_row
        [ $# = 2 ] || continue
        [ "$2" -gt "$uf_t" ] 2>/dev/null || continue
        [ "$uf_mtime" -gt "$uf_at" ] && uf_out="$1 $2 $uf_file"
        break
    done <<EOF
$uf_list
EOF
    [ -n "$uf_out" ] && printf '%s\n' "$uf_out"
}

# usage_check: 0 when the gate lets the driver start a session, 1 when it
# trips, with the line in `usage_line` and `<root>/usage-gated` written.
usage_check() {
    if [ -z "$usage_on" ]; then
        [ -n "$usage_off_said" ] || { say "usage gate off"; usage_off_said=1; }
        return 0
    fi
    uc_figure=$(usage_figure)
    if [ -z "$uc_figure" ]; then
        uc_t=$(now)
        if [ -z "$usage_unknown_said" ] || [ $((uc_t - usage_unknown_said)) -ge 600 ]; then
            say "usage unknown: no 7-day figure in $usage_dir or under $root/*/sessions; the gate lets the batch go on"
            usage_unknown_said=$uc_t
        fi
        return 0
    fi
    uc_rest=${uc_figure#* }
    uc_p=$(awk -v p="${uc_figure%% *}" 'BEGIN { s = sprintf("%.1f", p); sub(/\.0$/, "", s); print s }')
    uc_resets=${uc_rest%% *}
    uc_source=${uc_rest#* }
    uc_threshold=$usage_stop
    [ $((uc_resets - $(now))) -lt $((usage_hours * 3600)) ] && uc_threshold=$usage_late
    # The figure as read decides, and the line shows it to one decimal.
    awk -v p="${uc_figure%% *}" -v t="$uc_threshold" 'BEGIN { exit !(p + 0 >= t + 0) }' || return 0
    usage_line="usage-gated: 7-day at $uc_p% (from $uc_source), threshold $uc_threshold%, resets $(date -u -d "@$uc_resets" +%Y-%m-%dT%H:%M:%SZ)"
    {
        echo "$(stamp) $usage_line"
        echo "percent: $uc_p"
        echo "resets-at: $uc_resets"
        echo "threshold: $uc_threshold"
        echo "source: $uc_source"
    } > "$root/usage-gated"
    return 1
}
usage_off_said=
usage_unknown_said=

# usage_gate: the check before an invocation. It exits 13 when the gate
# trips, and otherwise clears a `usage-gated` an earlier run left.
usage_gate() {
    if usage_check; then
        rm -f "$root/usage-gated"
        return 0
    fi
    say "$usage_line"
    exit 13
}

# hook <name> <cmd>: run `--on-canary` or `--on-done` once, and log its
# status. An empty command runs nothing.
hook() {
    [ -n "$2" ] || return 0
    say "on-$1: $2"
    sh -c "$2"
    say "on-$1 exited $?"
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

# pin_pushed <sha>: the commit is on origin/main or on a branch under
# origin/tune/.
pin_pushed() {
    git merge-base --is-ancestor "$1" origin/main 2>/dev/null && return 0
    for ref in $(git for-each-ref --format='%(refname)' refs/remotes/origin/tune/ 2>/dev/null); do
        git merge-base --is-ancestor "$1" "$ref" 2>/dev/null && return 0
    done
    return 1
}

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
    if ! pin_pushed "$pin"; then
        echo "campaign-drive: the pin $pin is not on origin/main or on a branch under origin/tune/." >&2
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
    usage_gate
}

write_capped() {
    set -- $(classify "$root/a")
    ra=$1 ta=$2
    set -- $(classify "$root/b")
    {
        echo "spent-a: $(spent "$root/a")"
        echo "spent-b: $(spent "$root/b")"
        echo "spent-all: $(spent_all)"
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
    halts=0
    retried=0
    if [ "$steps" -gt 0 ] && [ "$phase" != canary ]; then
        ramp_to 1
    fi
    while :; do
        checks
        # What every other batch directory under the root spent: the other
        # batch, and every earlier attempt moved aside beside them (#1775).
        others=$(($(spent_all) - $(spent "$root/$dir")))
        given=$((cap - others - p * margin))
        if [ "$given" -le 0 ]; then
            write_capped
            say "$phase capped: $cap less $others spent by the other batch directories less $p x $margin leaves $given"
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
            ${reps:+--repetitions "$reps"} >&2 &
        campaign_pid=$!
        # The usage gate while the invocation runs (#1876). A trip writes the
        # halt of the slice, so campaign.sh starts no job after it, and the
        # jobs in flight finish. The halt is written again at each step while
        # campaign.sh runs, because an invocation that had not yet cleared
        # its slice would remove the first one. A stop file here keeps its
        # rule: it does not end an invocation.
        gated=
        while kill -0 "$campaign_pid" 2>/dev/null; do
            if [ -z "$gated" ] && ! usage_check; then
                gated=1
                say "$usage_line; halting $dir, the sessions in flight finish"
            fi
            if [ -n "$gated" ] && [ -d "$root/$dir/slice" ] && [ ! -f "$root/$dir/slice/halt" ]; then
                : > "$root/$dir/slice/halt"
            fi
            $poll_cmd 60
        done
        wait "$campaign_pid"
        code=$?
        set -- $(classify "$root/$dir")
        rec=$1 total=$2 failed=$4 halt=$5
        shift 6
        names=$*
        new=$((rec - before))
        mine=$(spent "$root/$dir")
        all=$(spent_all)
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
            say "overspend: spent $all of $cap cents (a $(spent "$root/a"), b $(spent "$root/b"), $((all - $(spent "$root/a") - $(spent "$root/b"))) in earlier attempts)"
            exit 7
        fi
        # A gated invocation ends the driver, with no halt counted, no
        # backoff and no change to the ramp. A batch that finished, a
        # recorder that failed and a stop keep their own rows.
        if [ -n "$gated" ]; then
            case $action in
                halt | bound | refused-only)
                    say "$phase usage-gated: exit 13; relaunch once the 7-day figure is below the threshold"
                    exit 13
                    ;;
            esac
        fi
        [ "$action" = refused-only ] || retried=0
        # The ramp, for the next slice of this batch alone. A halt lowers it
        # one step, a slice that ended on the bound and was clean raises it
        # one step, and anything else holds it. A slice that ended on the
        # bound refused no job, because campaign.sh exits 7 and not 9 when
        # it refused one.
        if [ "$steps" -gt 0 ] && [ "$phase" != canary ]; then
            was=$p
            if [ "$action" = halt ] && [ "$step" -gt 1 ]; then
                ramp_to $((step - 1))
                say "ramp $phase parallel $was -> $p after a halt"
            elif [ "$action" = bound ] && [ "$step" -lt "$steps" ] && clean "$root/$dir"; then
                ramp_to $((step + 1))
                say "ramp $phase parallel $was -> $p after a clean slice"
            fi
        fi
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
                say "$phase: a recorder failed for $names; read their record.err, nothing is run again"
                exit 7
                ;;
            stop) exit "$code" ;;
        esac
    done
}

# ------------------------------------------------------------ the saturation stop

# saturate_b: under `--saturate-at <n>`, sets `spec_run` to `<root>/b.spec`,
# the spec of batch B less every probe of a cell that saturated in batch A,
# and writes the ledger `<root>/saturated`. It grades batch A once, into
# `<root>/cells.a`, and derives the spec once, so a restart gives batch B
# the same spec and grades nothing again. It writes `b.done` when no line
# is left. A count or a plan that fails exits 14 before batch B spends.
saturate_b() {
    if [ ! -f "$root/cells.a" ]; then
        if ! sh "$cells" counts "$root/a" > "$root/cells.a.part" 2> "$root/cells.err" < /dev/null; then
            rm -f "$root/cells.a.part"
            say "the saturation count of batch A failed, so batch B does not start: $(tail -1 "$root/cells.err")"
            exit 14
        fi
        mv "$root/cells.a.part" "$root/cells.a"
        say "graded batch A once for the saturation stop at $saturate: $(wc -l < "$root/cells.a" | tr -d ' ') probe counts in cells.a"
    fi
    spec_run=$root/b.spec
    if [ ! -f "$root/b.spec" ]; then
        # The probes each line of spec-b selects, by line number, as
        # `<line> <arm> <set> <probe> <action>`.
        : > "$root/b.probes"
        index=0
        while IFS= read -r line || [ -n "$line" ]; do
            index=$((index + 1))
            case $line in '' | '#'*) continue ;; esac
            set -f
            # shellcheck disable=SC2086
            set -- $line
            set +f
            if ! sh "$cells" probes "$@" > "$root/b.line" 2>> "$root/cells.err" < /dev/null; then
                say "the plan of line $index of $spec_b failed, so batch B does not start: $(tail -1 "$root/cells.err")"
                exit 14
            fi
            awk -v i="$index" 'NF { print i, $0 }' "$root/b.line" >> "$root/b.probes"
        done < "$spec_b"
        # A cell is one action in one arm and set. It stops when batch A
        # graded exactly n of its sessions and all of them, or none,
        # satisfied. A line keeps every probe of a cell that runs on, gets
        # each probe of a stopped cell as an exclude, and is dropped when
        # every probe it selects stopped. A line that selects no probe, such
        # as a line of another tier, is copied as it is.
        awk -v n="$saturate" -v cellsf="$root/cells.a" -v probesf="$root/b.probes" -v ledger="$root/saturated.part" '
            function decide(    c, k, sk, gk) {
                decided = 1
                printf "" > ledger
                for (c = 1; c <= count; c++) {
                    k = order[c]
                    sk = s[k] + 0
                    gk = g[k] + 0
                    if (arm[k] == "scenario-0")
                        printf "runs %s/%s: %d of %d, the anchor, which no stop applies to\n", act[k], arm[k], sk, gk > ledger
                    else if (gk == n && (sk == 0 || sk == n)) {
                        stop[k] = 1
                        printf "stopped %s/%s at %d: %d of %d\n", act[k], arm[k], n, sk, gk > ledger
                    } else
                        printf "runs %s/%s: %d of %d\n", act[k], arm[k], sk, gk > ledger
                }
                close(ledger)
            }
            FILENAME == cellsf { k = $1 SUBSEP $2 SUBSEP $3; s[k] += $5; g[k] += $6; next }
            FILENAME == probesf {
                k = $2 SUBSEP $3 SUBSEP $5
                sel[$1] = sel[$1] " " $4
                cell[$1 SUBSEP $4] = k
                if (!(k in seen)) { seen[k] = 1; order[++count] = k; arm[k] = $2; act[k] = $5 }
                next
            }
            {
                if (!decided) decide()
                if (!(FNR in sel)) { print; next }
                m = split(sel[FNR], p, " ")
                added = ""
                left = 0
                for (j = 1; j <= m; j++) {
                    if (cell[FNR SUBSEP p[j]] in stop) added = added " " p[j]
                    else left++
                }
                if (left > 0) print $0 added
            }
            END { if (!decided) decide() }
        ' "$root/cells.a" "$root/b.probes" "$spec_b" > "$root/b.spec.part" || exit 14
        mv "$root/saturated.part" "$root/saturated"
        mv "$root/b.spec.part" "$root/b.spec"
        rm -f "$root/b.probes" "$root/b.line"
        while IFS= read -r entry; do
            say "saturation: $entry"
        done < "$root/saturated"
    fi
    if ! grep -qv '^[[:space:]]*\(#.*\)\{0,1\}$' "$root/b.spec"; then
        : > "$root/b.done"
        say "batch B has no line left: every cell of spec-b stopped at $saturate (see saturated), so no batch B runs"
    fi
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
    awk -v d="$root/a" '
        function first(p,   line) { line = ""; if ((getline line < p) <= 0) line = ""; close(p); return line }
        NF && first(d "/sessions/" $1 "/status") == "0" {
            cost = first(d "/sessions/" $1 "/cost") + 0
            if (n == 0 || cost < lo) lo = cost
            if (n == 0 || cost > hi) hi = cost
            n++
        }
        END { print lo, hi }
    ' "$root/a/jobs" > "$root/canary-range"
    : > "$root/canary-recorded"
    say "canary recorded"
    hook canary "$on_canary"
fi
canary_wait
if [ ! -f "$root/a.done" ]; then
    run_batch a a "$spec_a" "$parallel" 500
    : > "$root/a.done"
fi
if [ -n "$spec_b" ] && [ ! -f "$root/b.done" ]; then
    spec_run=$spec_b
    [ -z "$saturate" ] || saturate_b
    if [ ! -f "$root/b.done" ]; then
        run_batch b b "$spec_run" "$parallel" 500 "$reps_b"
        : > "$root/b.done"
    fi
fi
if [ -f "$root/a.done" ] && { [ -z "$spec_b" ] || [ -f "$root/b.done" ]; }; then
    : > "$root/done"
    say "done"
    hook done "$on_done"
fi
exit 0
