#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
#
# The batch driver of a paired probe run: every session of a campaign, from one
# pinned commit, in one shuffled order, under a ceiling it enforces while it
# spends.
#
#     sh tools/probe/campaign.sh --out <dir> --model <model> --spec <file> \
#         [--repetitions <n>] [--parallel <n>] [--max-turns <n>] [--seed <n>] \
#         [--cap-cents <n>] [--max-sessions <n>]
#     sh tools/probe/campaign.sh --out <dir> --assemble
#     sh tools/probe/campaign.sh --dry-run --spec <file> [--repetitions <n>] \
#         [--max-sessions <n>]
#
# `--repetitions <n>` sets the repetitions of every line. A count at or below
# the one a line's tier declares is passed to the plan, a lowering. A count
# above it is admitted on a line only up to the count the dry run prices for
# that line, and the line is then planned, and each of its sessions
# recorded, at the declared count, because the plan refuses a raise (#1659).
# The owner's approval of a priced plan permits that raise and nothing above
# it. A pooled line prices at the powered count and any other line at the
# declared count, so a line not under `pooled:` admits no raise. The price is
# read from `campaign-dry-run.sh --priced`, which holds the one copy of the
# power calculation. A count above a line's price refuses the batch with 5
# before any tree is built and before the harness is asked for its version.
# A resumed invocation that gives no `--repetitions` keeps the count the
# batch recorded.
#
# `--dry-run` spends nothing and calls no model (#1472). It plans every line
# of the spec, prints the sessions and the cost of each line, each arm and
# each tier against its ceiling, and the total. It prints the power
# calculation that `power:` in `.headwater/probe.yml` declares, and prices a
# line of a pooled category at the repetitions that calculation needs. It
# builds the present tree from `git archive` of `HEAD` in a directory outside
# this checkout, prints the delta of every arm with `ablate.sh --diff`, starts
# `headwater mcp` in the `mcp` arm's tree and checks that it lists tools, and
# runs the leak check (`seal.sh --leak`) over the present tree and the `mcp`
# arm's tree. It refuses a line that pools a probe under `leaks_kept:` with one that
# is not. It exits 0 when the only refusal of any plan is the ceiling, which
# it prints as a line, 5 for any other refusal of a plan, and 8 when an arm
# differs from the present tree by more than its delta, a leak string leaks, a leak-kept
# probe shares a line, or the MCP server lists no tool. It deletes the trees
# before it exits. With `--max-sessions <n>` it also prints the stagger as
# `slices: <total> sessions in <slices> slices of at most <n>`.
#
# `--cap-cents` stops the batch below a figure a person agreed to for this batch
# alone, such as a pilot, where the tier's ceiling is set for the full run. The
# cap is cumulative over the batch directory, not per slice: a resumed run
# that does not give `--cap-cents` keeps the cap the batch recorded, and one
# that gives it replaces it.
#
# `--session-cents <B>` is the most one session may spend (#1659). The driver
# passes it to the harness as `--max-budget-usd <B/100>`, and the harness
# stops the session at the turn that crosses it. The ceiling and the cap then
# reserve B for each session in flight, not the declared cost, so a session
# starts only when `spent + (in flight + 1) * B` is at most the cap. The check
# and the `started` marker are written under one lock, so the invariant holds
# across `--parallel` workers: the batch spends at most the cap, and more only
# by what the harness spends past B in the turn that crosses it, once for
# each session in flight at the end. So the bound is the cap plus `--parallel`
# times the largest such overshoot: at `--parallel 1` it is one overshoot, and
# a verify of #1659 spent 360 under a cap of 280 at `--parallel 4` with a stub
# that crossed a B of 70 by 20 cents. Like the
# cap, it is recorded in `<out>/session-cents`, so a resumed invocation that
# does not give it keeps it. Without it, a session in flight reserves the
# declared cost and nothing bounds what one session spends, so the cap is a
# reservation and not a stop: #1474 had a cap of 400 and spent 437.
#
# `--max-sessions <n>` starts at most `n` new sessions in this invocation, so
# one batch can run in slices (#1472). A job already recorded with `status` 0
# does not count toward it. The workers claim the bound atomically, with one
# token file per session they start, created with the shell's `set -C`, so
# the bound holds across them.
#
# `tools/probe/probe-record.sh` records one session. A campaign is hundreds of
# them, over three tiers and arms, and the design on #980 named what the
# sessions must share for their rates to compare. This script is that list, in
# the order a run meets it.
#
# ## The spec
#
# One run per line: a tier, an arm, a category, and any probes named out of the
# category. A line that starts with `#` is a comment.
#
#     campaign      present sufficiency
#     campaign      absent  sufficiency
#     documentation absent  sufficiency
#     campaign      present discovery HW-PROBE-the-authoring-skill-reaches-…
#
# Each line is planned with `headwater probe plan` before anything is built,
# and a line the plan refuses stops the batch before it spends anything.
#
# ## One pinned commit
#
# The checkout must be clean, and every workspace is built from `git archive` of
# its `HEAD`. So every arm meets one corpus, the `tree` digest the plan fixed is
# the tree every session read, and no workspace carries the `.git` pointer that
# #897 found a session using. The plan runs in this checkout, so nothing may be
# committed here until the batch is done: the script refuses to start a session
# once `HEAD` moves.
#
# Nothing under `engine/target` is in an archive, so the driver copies this
# checkout's built `headwater` into every tree at the path the skills and the
# intent hook look for it. The pilot of #980 ran without one: the skills sent a
# present-arm session to `headwater route`, it found no binary, and the intent
# hook was live in none of 66 sessions. So the present arm measured the
# governance prose without the tool it points at. The owner ruled on #980 on
# 2026-09-28 that every workspace of both arms carries the binary. In an absent
# arm it refuses, because `.headwater/` is gone, which is what that arm is
# declared to remove. The driver runs the build before the batch, so the binary
# it copies is the build of the pinned commit. It reads the binary where cargo
# wrote it, under `CARGO_TARGET_DIR` when that is set, and refuses with 3 when
# the build left none there.
#
# ## The trees
#
# The present tree is the archive with the instrument removed
# (`ablate.sh --present`) and sealed against every probe of every line
# (`seal.sh`), so each arm of every run lost the same answer keys. Each other
# arm's tree is a copy of the present tree that `ablate.sh <tier> <tree>
# <arm>` changed: the absent arm loses its tier's ablation, and a component
# arm loses or gains its `components` delta. The
# present tree also serves as the oracle tree that `probe-transform.sh` grafts
# a `patched` artifact into, so both arms are graded by one oracle. Each
# session runs in a fresh copy of its arm's tree, deleted once it is recorded.
#
# ## The order, and the ceiling
#
# Every (line, probe, repetition) is one job. The jobs are shuffled with the
# seed, and `--parallel` workers take them in that order, so a drift inside the
# batch lands on both arms alike. A worker sums the realized cost of every
# recorded session of the job's tier, adds `--session-cents` (or, without it,
# the declared session cost) for each session in flight, and skips the job if
# one more session would cross the tier's ceiling. The engine refuses at plan
# time and this refuses at run time. A worker holds one lock, a file the
# shell creates with `set -C`, from the first read of that sum to the write
# of its `started` marker, so no two workers count the same room. A job that
# runs again after a failure keeps what its earlier attempt spent in
# `spent-before`, and both checks count it.
#
# ## The turn cap, and what the recorder sees
#
# Each session runs under the turn cap its tier declares as `max_turns` in
# `.headwater/probe.yml`, which the plan prints (#1384). `--max-turns` overrides
# it for every tier of the batch, and a line whose tier declares none, with no
# override, refuses with exit 2 before anything is built. A session the cap
# stops is recorded with status 0 and is never drawn again. Assembly counts
# those sessions per line.
#
# The recorder is given the arm's tree as `--baseline`, so a file the session
# wrote through `Bash` is in `produced` as well as one it wrote with `Edit`.
#
# ## Resume
#
# A job whose session directory holds `status` 0 is done and is not run again.
# Run the same command again to finish a batch that stopped, or to record again
# a session whose recorder failed. A capped session did not fail, and nor did
# one the session budget stopped. A job run again keeps what its failed
# attempt spent, in `spent-before`, so the cap of a resumed batch counts it.
#
# **A halt on a harness failure.** When the recorder of a session exits 10,
# the harness exited nonzero for a reason other than the turn cap or the
# session budget. A usage limit of the harness surfaces this way, and a
# session started after it would fail the same way. So the job writes `<out>/slice/halt`, and no job of this
# invocation starts after it. The script does not match the text of a limit
# message, because no log of one is recorded to match against. The next
# invocation clears the halt and runs the failed job again.
#
# **A staggered batch** runs in several invocations against one pinned commit,
# for example to stay inside the harness's 5-hour usage window. Detach a
# worktree of its own at the pin (`git worktree add --detach <dir> <pin>`) and
# build the engine there. Run the batch from that worktree with `--out`
# outside it and `--max-sessions <n>`. When the invocation stops with jobs
# left, it exits 9 and prints `campaign: <recorded> of <total> sessions
# recorded, <remaining> remain; run the same command again to continue at
# <pin>.` Run the same command again, with the same `--out`, until it exits 0.
# The batch refuses a checkout whose `HEAD` is not the pin. It records
# `claude --version` in `<out>/claude-version` on its first invocation, and
# refuses with 4, before any spend, an invocation whose harness reports
# another version: assembly refuses a line whose sessions disagree on their
# identity, so a batch that changed its harness half way is a batch that does
# not assemble. Run the batch to the end on the harness it started with.
#
# ## Assembly
#
# `--assemble` checks that every recorded session of a line carries one
# identity, and writes one identity block and one events block per line under
# `<out>/assembled/`, with a count of sessions, the summed cost, how many
# sessions the intent hook reached, how many the turn cap stopped, how many
# the session budget stopped, the paths
# outside the workspace its sessions named, how many sessions it could not
# count, and the calls its sessions made to a web tool (#1472). A session that
# did not record, or whose record states no count, is uncounted and adds
# nothing to either sum, so it never reads as 0. After the lines, one line on
# standard error states the same three figures over the whole batch, and the
# count of its sessions. A count above 0 does not fail assembly. A person
# wraps them in a transcript with `headwater new`, because a transcript is an
# authored document of this corpus and this script writes nothing inside it.
#
# It needs `git`, `jq`, `tar`, `awk`, `bwrap` and the `claude` harness, and it
# spends real money. Each session runs confined to its workspace, under a
# configuration directory of its own at `<out>/config/<session>`, as
# `probe-record.sh` describes (#1467).
#
# ## Exit status
#
#   0   the batch finished, or the assembly was written
#   2   a usage error, or a line whose tier declares no turn cap and no
#       `--max-turns` was given
#   3   a tool is missing, or no engine is built
#   4   the checkout is dirty, `HEAD` moved during the batch, `HEAD` is not
#       the commit the output directory holds a batch of, or the `claude`
#       harness is another version than the batch recorded
#   5   the plan refuses a line of the spec, or `--repetitions` is above the
#       count the dry run prices for a line
#   6   the output directory is inside this checkout, or a batch's output
#       directory is under `$HOME` (#1467)
#   7   a session of the batch did not record, or a line's sessions disagree on
#       their identity at assembly
#   9   the invocation stopped with jobs left, on `--max-sessions` or on a
#       halt, and no session it started failed for another reason. Run the
#       same command again to continue

set -u

out=
model=
spec=
repetitions=
parallel=1
max_turns=
seed=0
cap=
session_cents=
max_sessions=
assemble=0
dry=0
job=

while [ $# -gt 0 ]; do
    case $1 in
        --out) out=${2:-}; shift 2 ;;
        --model) model=${2:-}; shift 2 ;;
        --spec) spec=${2:-}; shift 2 ;;
        --repetitions) repetitions=${2:-}; shift 2 ;;
        --parallel) parallel=${2:-}; shift 2 ;;
        --max-turns) max_turns=${2:-}; shift 2 ;;
        --seed) seed=${2:-}; shift 2 ;;
        --cap-cents) cap=${2:-}; shift 2 ;;
        --session-cents) session_cents=${2:-}; shift 2 ;;
        --max-sessions) max_sessions=${2:-}; shift 2 ;;
        --assemble) assemble=1; shift ;;
        --dry-run) dry=1; shift ;;
        --job) job=${2:-}; shift 2 ;;
        *) echo "campaign: unknown argument \`$1\`" >&2; exit 2 ;;
    esac
done
case $max_sessions in
    '') ;;
    *[!0-9]*|0) echo "campaign: --max-sessions takes a positive whole number, not \`$max_sessions\`." >&2; exit 2 ;;
esac
case $repetitions in
    '') ;;
    *[!0-9]*|0*) echo "campaign: --repetitions takes a positive whole number, not \`$repetitions\`." >&2; exit 2 ;;
esac
case $session_cents in
    '') ;;
    *[!0-9]*|0*) echo "campaign: --session-cents takes a positive whole number of cents, not \`$session_cents\`." >&2; exit 2 ;;
esac

case $0 in
    */*) invoked_from=${0%/*} ;;
    *) invoked_from=. ;;
esac
root=$(cd "$invoked_from/../.." && pwd -P)
engine=$root/engine/target/dev-release/headwater
[ -x "$engine" ] || engine=$root/engine/target/release/headwater

if [ "$dry" = 1 ]; then
    exec sh "$root/tools/probe/campaign-dry-run.sh" "$spec" "$repetitions" "$max_sessions"
fi

[ -n "$out" ] || { echo "campaign: --out is required" >&2; exit 2; }
mkdir -p "$out" || exit 2
out=$(cd "$out" && pwd -P)
case "$out" in
    "$root"|"$root"/*)
        echo "campaign: the output directory is inside this checkout. A workspace there moves the tree the plan fixed." >&2
        exit 6
        ;;
esac
# A session runs confined to its workspace (#1467), and a batch directory
# under `$HOME` would sit beside the host's own configuration and copies of
# this repository. Assembly of a batch already recorded there still runs.
home_real=$(cd "${HOME:-/nonexistent}" 2>/dev/null && pwd -P) || home_real=
if [ "$assemble" = 0 ] && [ -n "$home_real" ]; then
    case "$out" in
        "$home_real"|"$home_real"/*)
            echo "campaign: the output directory is under \$HOME ($home_real). Put the batch outside it, such as under /mnt or /var/tmp (#1467)." >&2
            exit 6
            ;;
    esac
fi

# ---------------------------------------------------------------------------
# One job, run by a worker. The line is the job's row of `<out>/jobs`.
# ---------------------------------------------------------------------------
run_job() {
    # shellcheck disable=SC2086
    set -- $1
    name=$1 index=$2 tier=$3 arm=$4 category=$5 probe=$6
    shift 6
    excludes=
    for excluded in "$@"; do
        excludes="$excludes --exclude $excluded"
    done
    dir=$out/sessions/$name
    if [ -f "$dir/status" ] && [ "$(cat "$dir/status")" = 0 ]; then
        return 0
    fi
    # A halted invocation starts nothing more, and leaves the job to the next.
    if [ -f "$out/slice/halt" ]; then
        return 0
    fi
    if [ "$(git -C "$root" rev-parse HEAD)" != "$(cat "$out/head")" ]; then
        echo "campaign: HEAD moved during the batch, so $name was not started." >&2
        : > "$out/slice/refused/$name"
        return 0
    fi
    # The ceiling and the cap, at run time, under one lock (#1659). The lock is
    # a file the shell creates with `set -C`, as the tokens below are, and a
    # worker holds it from the first read of what the batch spent to the
    # write of its `started` marker. Without it, N workers can each read the
    # same room and all start. The lock holds the process id of its worker. A
    # worker that waits on a lock whose holder is gone, killed inside it or
    # returned without releasing it, halts the invocation rather than wait
    # forever or break the lock: nothing has started under it, so nothing is
    # spent, and the next invocation clears `<out>/slice` with the lock.
    # `HEADWATER_CAMPAIGN_HOLDER_DELAY` is a fixture seam between the read
    # of the holder and the test of it.
    until ( set -C; printf '%s\n' "$$" > "$out/slice/lock" ) 2>/dev/null; do
        holder=$(cat "$out/slice/lock" 2>/dev/null)
        if [ -n "${HEADWATER_CAMPAIGN_HOLDER_DELAY:-}" ]; then
            sleep "$HEADWATER_CAMPAIGN_HOLDER_DELAY"
        fi
        # A holder can release the lock and exit between the read of its
        # pid and the test of it. The holder is then gone and so is its
        # lock, so the waiter halts only when the lock still names the
        # process that is gone.
        if [ -n "$holder" ] && ! kill -0 "$holder" 2>/dev/null &&
            [ "$(cat "$out/slice/lock" 2>/dev/null)" = "$holder" ]; then
            : > "$out/slice/halt"
            echo "campaign: $name: the worker $holder holds the batch lock and is gone, so this invocation starts no other job (halt). Run the same command again." >&2
            return 0
        fi
        if [ -f "$out/slice/halt" ]; then
            return 0
        fi
        sleep 0.1
    done
    # What one session in flight may still spend: `--session-cents` where the
    # batch set one, because the harness stops the session there, and the
    # declared cost of the tier otherwise.
    ceiling=$(cat "$out/ceiling.$tier")
    unit=$(cat "$out/unit.$tier")
    reserve=$unit
    [ -s "$out/session-cents" ] && reserve=$(cat "$out/session-cents")
    # A session's spend is its `cost`, and `spent-before` holds what earlier
    # attempts of the same job spent before it was run again. A session that
    # started and has no status is in flight.
    # This job's own directory is never in flight: a session of a killed
    # invocation left it started, and this one is about to replace it. The
    # sums are one `find` and one `awk`, because the lock is held while they
    # run: a loop of `cat` over 4,284 sessions took 9.8 s per job (verify of
    # #1659).
    # shellcheck disable=SC2046
    set -- $(find "$out/sessions" -mindepth 2 -maxdepth 2 -type f \( -name tier -o -name cost \
        -o -name spent-before -o -name started -o -name status \) 2>/dev/null \
        | awk -v own="$dir" -v tier="$tier" '
            {
                n = split($0, part, "/"); f = part[n]; d = substr($0, 1, length($0) - length(f) - 1)
                dirs[d] = 1; has[d, f] = 1
                if (f != "started" && f != "status") { v = ""; if ((getline v < $0) > 0) value[d, f] = v; close($0) }
            }
            END {
                for (d in dirs) {
                    this = value[d, "cost"] + value[d, "spent-before"]
                    flying = (d != own && !((d, "cost") in has) && ((d, "started") in has) && !((d, "status") in has)) ? 1 : 0
                    all += this; running += flying
                    if (value[d, "tier"] == tier) { spent += this; inflight += flying }
                }
                printf "%d %d %d %d\n", spent, inflight, all, running
            }')
    spent=$1 inflight=$2 all=$3 running=$4
    if [ $((spent + (inflight + 1) * reserve)) -gt "$ceiling" ]; then
        rm -f "$out/slice/lock"
        echo "campaign: $name skipped: $spent cents spent and $inflight in flight at $reserve each against a $tier ceiling of $ceiling." >&2
        : > "$out/slice/refused/$name"
        return 0
    fi
    # The cap, across every tier of the batch, where the caller set one.
    if [ -s "$out/cap" ] && [ $((all + (running + 1) * reserve)) -gt "$(cat "$out/cap")" ]; then
        rm -f "$out/slice/lock"
        echo "campaign: $name skipped: $all cents spent and $running in flight at $reserve each against the batch cap of $(cat "$out/cap")." >&2
        : > "$out/slice/refused/$name"
        return 0
    fi
    # The bound of this invocation. A token is a file the shell creates with
    # `set -C`, which opens it with O_EXCL, so two workers never take one
    # token, and no more sessions start than there are tokens. Not `mkdir`:
    # the uutils `mkdir` that some hosts ship checks and then creates, and
    # three workers took two of its tokens in 56 of 200 trials (#1472).
    if [ -s "$out/slice/max" ]; then
        bound=$(cat "$out/slice/max")
        token=1
        while [ "$token" -le "$bound" ]; do
            ( set -C; : > "$out/slice/token.$token" ) 2>/dev/null && break
            token=$((token + 1))
        done
        if [ "$token" -gt "$bound" ]; then
            rm -f "$out/slice/lock"
            return 0
        fi
    fi
    # A seam for the fixtures alone: it holds a worker between its check and
    # its marker, so a check made outside the lock lets every worker through.
    if [ -n "${HEADWATER_CAMPAIGN_CHECK_DELAY:-}" ]; then
        sleep "$HEADWATER_CAMPAIGN_CHECK_DELAY"
    fi
    : > "$out/slice/started/$name"
    mkdir -p "$dir"
    printf '%s\n' "$tier" > "$dir/tier"
    # A job run again keeps what its earlier attempt spent, so the next
    # check still counts it.
    if [ -f "$dir/cost" ]; then
        before=$(cat "$dir/cost")
        [ -f "$dir/spent-before" ] && before=$((before + $(cat "$dir/spent-before")))
        printf '%s\n' "$before" > "$dir/spent-before"
    fi
    rm -f "$dir/status" "$dir/cost"
    : > "$dir/started"
    # The second seam for the fixtures: it holds the lock after the marker,
    # so a fixture can see that the marker is written before the lock goes.
    if [ -n "${HEADWATER_CAMPAIGN_HOLD_DELAY:-}" ]; then
        sleep "$HEADWATER_CAMPAIGN_HOLD_DELAY"
    fi
    rm -f "$out/slice/lock"
    ws=$out/ws/$name
    rm -rf "$ws"
    # The session's own configuration directory, outside every tree (#1467).
    # The driver requires it empty, and copies the host's credentials into it
    # for the session alone.
    rm -rf "$out/config/$name"
    mkdir -p "$out/config/$name"
    cp -a "$out/trees/$tier-$arm" "$ws" || { echo 2 > "$dir/status"; return 0; }
    # The count the line's plan was given: the batch's count when it is at or
    # below the declared one, and none when it raises it (#1659), so the
    # recorder plans the session as the batch planned its line.
    plan_repetitions=$repetitions
    [ -f "$out/plans/L$index.repetitions" ] && plan_repetitions=$(cat "$out/plans/L$index.repetitions")
    # shellcheck disable=SC2086
    HEADWATER_PROBE_CONFIG_DIR=$out/config/$name \
        sh "$root/tools/probe/probe-record.sh" --probe "$probe" --session "$name" \
        --task-file "$out/tasks/$probe.md" --workspace "$ws" --model "$model" \
        --tier "$tier" --arm "$arm" --category "$category" $excludes \
        ${plan_repetitions:+--repetitions "$plan_repetitions"} \
        ${max_turns:+--max-turns "$max_turns"} \
        ${budget:+--max-budget-usd "$budget"} \
        --baseline "$out/trees/$tier-$arm" \
        --oracle-tree "$out/trees/oracle" --raw "$dir/raw.jsonl" \
        > "$dir/record.md" 2> "$dir/record.err"
    status=$?
    sed -n 's/^cost_cents: //p' "$dir/record.md" | head -1 > "$dir/cost.tmp"
    if [ ! -s "$dir/cost.tmp" ] && [ -f "$dir/raw.jsonl" ]; then
        # A recorder that failed after the harness still spent. The cost is
        # read from the log it wrote, so the ceiling counts it.
        sh "$root/tools/probe/probe-record.sh" --provider-only "$dir/raw.jsonl" 2>/dev/null \
            | sed -n 's/^cost_cents: //p' > "$dir/cost.tmp"
    fi
    [ -s "$dir/cost.tmp" ] && mv "$dir/cost.tmp" "$dir/cost" || rm -f "$dir/cost.tmp"
    printf '%s\n' "$status" > "$dir/status"
    if [ "$status" = 10 ]; then
        : > "$out/slice/halt"
        echo "campaign: $name: the harness failed for a reason other than the turn cap or the budget, so this invocation starts no other job (halt). A usage limit stops a session this way." >&2
    fi
    rm -rf "$ws"
    printf 'campaign: %s exited %s, %s cents\n' "$name" "$status" "$(cat "$dir/cost" 2>/dev/null || echo '?')" >&2
}

if [ -n "$job" ]; then
    model=$(cat "$out/model")
    repetitions=$(cat "$out/repetitions")
    max_turns=$(cat "$out/max-turns")
    # The harness takes the budget in dollars, so 60 cents is `0.60`.
    budget=
    if [ -s "$out/session-cents" ]; then
        budget=$(awk -v c="$(cat "$out/session-cents")" 'BEGIN { printf "%.2f", c / 100 }')
    fi
    mkdir -p "$out/slice/started" "$out/slice/refused"
    run_job "$job"
    exit 0
fi

# ---------------------------------------------------------------------------
# Assembly: one identity block and one events block per line of the spec.
# ---------------------------------------------------------------------------
if [ "$assemble" = 1 ]; then
    [ -f "$out/lines" ] || { echo "campaign: no batch at $out" >&2; exit 2; }
    mkdir -p "$out/assembled"
    failed=0
    batch_sessions=0
    batch_outside=0
    batch_uncounted=0
    batch_web=0
    while IFS= read -r line; do
        # shellcheck disable=SC2086
        set -- $line
        index=$1 tier=$2 arm=$3 category=$4
        # The line number leads the stem, because a spec may run one category
        # on two lines of one arm, as the leak-kept probes of #1472 do.
        stem=L$index-$tier-$arm-$category
        identity=
        cost=0
        count=0
        live=0
        capped=0
        budgeted=0
        outside=0
        uncounted=0
        web=0
        first_at=
        : > "$out/assembled/$stem.events.yaml"
        for dir in "$out"/sessions/L$index-*; do
            [ -d "$dir" ] || continue
            batch_sessions=$((batch_sessions + 1))
            if [ "$(cat "$dir/status" 2>/dev/null)" != 0 ]; then
                echo "campaign: $(basename "$dir") did not record (status $(cat "$dir/status" 2>/dev/null || echo none))." >&2
                failed=1
                # It may have spent, and nobody counted what it named.
                uncounted=$((uncounted + 1))
                continue
            fi
            # The two counts each record states (#1472). A record that states
            # either one as uncounted, or states none, is an uncounted session
            # and adds nothing to either sum, so it never reads as 0.
            named=$(sed -n 's/^The session named \([0-9][0-9]*\) paths\{0,1\} outside its workspace .*/\1/p' "$dir/record.md" | head -1)
            webbed=$(sed -n 's/^The session made \([0-9][0-9]*\) calls\{0,1\} to a web tool\.$/\1/p' "$dir/record.md" | head -1)
            if [ -n "$named" ] && [ -n "$webbed" ]; then
                outside=$((outside + named))
                web=$((web + webbed))
            else
                uncounted=$((uncounted + 1))
            fi
            this=$(sed -n '/^```yaml$/,/^```$/p' "$dir/record.md" | sed '1d;$d' \
                | grep -v -e '^at: ' -e '^cost_cents: ')
            if [ -z "$identity" ]; then
                identity=$this
            elif [ "$this" != "$identity" ]; then
                echo "campaign: $(basename "$dir") records another identity than the rest of $stem." >&2
                failed=1
            fi
            at=$(sed -n 's/^at: //p' "$dir/record.md" | head -1)
            if [ -z "$first_at" ] || [ "$at" \< "$first_at" ]; then
                first_at=$at
            fi
            cost=$((cost + $(cat "$dir/cost")))
            count=$((count + 1))
            grep -q '^The intent hook was live' "$dir/record.md" && live=$((live + 1))
            grep -q '^The session stopped at the turn cap' "$dir/record.md" && capped=$((capped + 1))
            grep -q '^The session stopped at its budget' "$dir/record.md" && budgeted=$((budgeted + 1))
            sed -n '/^- probe: /,$p' "$dir/record.md" >> "$out/assembled/$stem.events.yaml"
        done
        {
            printf '%s\n' "$identity" | grep -v -e '^tier: ' -e '^arm: '
            printf 'tier: %s\narm: %s\nat: %s\ncost_cents: %s\n' "$tier" "$arm" "$first_at" "$cost"
        } > "$out/assembled/$stem.identity.yaml"
        printf '%s sessions, %s cents, the intent hook live in %s, %s stopped at the turn cap, %s stopped at the session budget, %s paths outside the workspace named, %s sessions uncounted, %s web-tool calls\n' \
            "$count" "$cost" "$live" "$capped" "$budgeted" "$outside" "$uncounted" "$web" \
            > "$out/assembled/$stem.summary"
        printf 'campaign: %s: %s' "$stem" "$(cat "$out/assembled/$stem.summary")" >&2
        printf '\n' >&2
        batch_outside=$((batch_outside + outside))
        batch_uncounted=$((batch_uncounted + uncounted))
        batch_web=$((batch_web + web))
    done < "$out/lines"
    # The figure the campaign's report states for confinement (#1472). It is a
    # report and not a gate: a confined session's tries are unreadable.
    printf 'campaign: the batch named %s paths outside the workspace over %s sessions, %s sessions uncounted, %s web-tool calls\n' \
        "$batch_outside" "$batch_sessions" "$batch_uncounted" "$batch_web" >&2
    [ "$failed" = 0 ] || exit 7
    exit 0
fi

# ---------------------------------------------------------------------------
# The batch.
# ---------------------------------------------------------------------------
[ -n "$model" ] && [ -n "$spec" ] || {
    echo "usage: campaign.sh --out <dir> --model <model> --spec <file> [--repetitions <n>] [--parallel <n>] [--max-turns <n>] [--seed <n>] [--cap-cents <n>] [--session-cents <n>] [--max-sessions <n>]" >&2
    exit 2
}
[ -f "$spec" ] || { echo "campaign: no spec at $spec" >&2; exit 2; }
for tool in git jq tar awk cargo claude bwrap; do
    command -v "$tool" >/dev/null 2>&1 || { echo "campaign: \`$tool\` is not on the path." >&2; exit 3; }
done
[ -x "$engine" ] || { echo "campaign: no engine is built at $root/engine/target." >&2; exit 3; }
if [ -n "$(git -C "$root" status --porcelain --untracked-files=no)" ]; then
    echo "campaign: the checkout has uncommitted changes, so the tree the plan fixes is not the tree an archive holds." >&2
    exit 4
fi
head=$(git -C "$root" rev-parse HEAD)
# Cargo decides whether the binary is the build of this tree, and the build is
# a no-op when it is. A comparison of timestamps would refuse a binary built
# before the commit that pinned its source.
cargo build --profile dev-release -p headwater-cli --locked \
    --manifest-path "$root/engine/Cargo.toml" >/dev/null 2>"$out/build.err" || {
    echo "campaign: the engine did not build, so no workspace can carry the pinned engine:" >&2
    tail -5 "$out/build.err" >&2
    exit 3
}
# The binary is where cargo wrote it: under `CARGO_TARGET_DIR` when the caller
# set one, which a relative value names from this directory, and under the
# engine's own `target` otherwise. Reading `engine/target` while cargo wrote
# elsewhere would copy a binary of another commit into every tree.
target=${CARGO_TARGET_DIR:-$root/engine/target}
case $target in
    /*) ;;
    *) target=$PWD/$target ;;
esac
engine=$target/dev-release/headwater
[ -x "$engine" ] || {
    echo "campaign: the build wrote no engine at $engine, so no workspace can carry the pinned engine." >&2
    exit 3
}
if [ -f "$out/head" ] && [ "$(cat "$out/head")" != "$head" ]; then
    echo "campaign: $out holds a batch of $(cat "$out/head"), and HEAD is $head. Use another directory." >&2
    exit 4
fi
# The count is the batch's, so a resumed invocation that gives none keeps the
# one recorded, as it keeps the cap: its job list was made at that count.
if [ -z "$repetitions" ] && [ -s "$out/repetitions" ]; then
    repetitions=$(cat "$out/repetitions")
fi
# A raise is admitted on a line only up to the count the dry run prices for
# it (#1659). This is checked before anything of the batch is written, so a
# refused count leaves a resumed batch as it was.
if [ -n "$repetitions" ]; then
    HW_CAMPAIGN_ENGINE=$engine sh "$root/tools/probe/campaign-dry-run.sh" --priced "$spec" \
        > "$out/priced" 2> "$out/priced.err" || {
        priced_status=$?
        echo "campaign: the dry run did not price the spec:" >&2
        cat "$out/priced.err" >&2
        exit "$priced_status"
    }
    raise_refused=0
    while read -r index declared price how; do
        [ "$repetitions" -gt "$declared" ] && [ "$repetitions" -gt "$price" ] || continue
        what=$(awk 'NF && $1 !~ /^#/' "$spec" | awk -v i="$index" 'NR == i { print $1, $2, $3 }')
        echo "campaign: line $index ($what): --repetitions $repetitions is above the $price the dry run prices for it ($how)." >&2
        raise_refused=1
    done < "$out/priced"
    [ "$raise_refused" = 0 ] || exit 5
fi
printf '%s\n' "$head" > "$out/head"
printf '%s\n' "$model" > "$out/model"
printf '%s\n' "$repetitions" > "$out/repetitions"
printf '%s\n' "$max_turns" > "$out/max-turns"
# The cap is the batch's, so a resumed invocation that gives none keeps the
# one recorded (#1472).
if [ -n "$cap" ]; then
    printf '%s' "$cap" > "$out/cap"
elif [ ! -f "$out/cap" ]; then
    : > "$out/cap"
fi
# The session budget is the batch's too, for the same reason (#1659).
if [ -n "$session_cents" ]; then
    printf '%s' "$session_cents" > "$out/session-cents"
elif [ ! -f "$out/session-cents" ]; then
    : > "$out/session-cents"
fi
# The state of this invocation alone: the bound, the tokens taken against it,
# the jobs it started or refused, and the halt.
rm -rf "$out/slice"
mkdir -p "$out/slice/started" "$out/slice/refused"
printf '%s' "$max_sessions" > "$out/slice/max"
mkdir -p "$out/plans" "$out/tasks" "$out/trees" "$out/sessions" "$out/ws"

# Plan every line. A refusal stops the batch before anything is built.
awk 'NF && $1 !~ /^#/' "$spec" | awk '{ print NR, $0 }' > "$out/lines"
probes=
while IFS= read -r line; do
    # shellcheck disable=SC2086
    set -- $line
    index=$1 tier=$2 arm=$3 category=$4
    shift 4
    excludes=
    for excluded in "$@"; do
        excludes="$excludes --exclude $excluded"
    done
    # A count above the line's declared one was held to its price above, and
    # the line is planned at the declared count, which the plan does not
    # refuse. A count at or below it is a lowering and goes to the plan.
    plan_repetitions=$repetitions
    if [ -n "$repetitions" ]; then
        declared=$(awk -v i="$index" '$1 == i { print $2 }' "$out/priced")
        [ -n "$declared" ] && [ "$repetitions" -gt "$declared" ] && plan_repetitions=
    fi
    printf '%s\n' "$plan_repetitions" > "$out/plans/L$index.repetitions"
    # shellcheck disable=SC2086
    "$engine" probe plan --root "$root" --tier "$tier" --category "$category" $excludes \
        ${plan_repetitions:+--repetitions "$plan_repetitions"} > "$out/plans/L$index" 2>&1
    if grep -q '^## This run does not start' "$out/plans/L$index"; then
        echo "campaign: line $index ($tier $arm $category) is refused by the plan:" >&2
        sed -n '/^## This run does not start/,$p' "$out/plans/L$index" | sed '1,2d' >&2
        exit 5
    fi
    ceiling=$(sed -n 's/.*against a ceiling of \$\([0-9.]*\)\..*/\1/p' "$out/plans/L$index" | head -1)
    unit=$(sed -n 's/.*at a declared \$\([0-9.]*\) each.*/\1/p' "$out/plans/L$index" | head -1)
    awk -v d="$ceiling" 'BEGIN { printf "%d\n", d * 100 + 0.5 }' > "$out/ceiling.$tier"
    awk -v d="$unit" 'BEGIN { printf "%d\n", d * 100 + 0.5 }' > "$out/unit.$tier"
    # The turn cap the tier declares (#1384). `probe-record.sh` reads it from
    # the same plan for each session, and `--max-turns` overrides it for every
    # tier of the batch. A tier that declares none, run with no override,
    # refuses before anything is built, because a batch with no cap lets one
    # session spend without a bound.
    turns=$(sed -n 's/.*stops at a turn cap of \([0-9][0-9]*\).*/\1/p' "$out/plans/L$index" | head -1)
    if [ -z "$turns" ] && [ -z "$max_turns" ]; then
        echo "campaign: line $index ($tier $arm $category) runs a tier that declares no \`max_turns\` in .headwater/probe.yml, and no --max-turns was given." >&2
        exit 2
    fi
    for probe in $(sed -n 's/^- \(HW-PROBE-[^ ]*\) (.*/\1/p' "$out/plans/L$index"); do
        case " $probes " in *" $probe "*) ;; *) probes="$probes $probe" ;; esac
        printf '%s %s\n' "$index" "$probe" >> "$out/pairs.tmp"
    done
done < "$out/lines"
sort -u "$out/pairs.tmp" > "$out/pairs" && rm -f "$out/pairs.tmp"

# The task of each probe: the body of `## Task` and nothing else (spec 15).
for probe in $probes; do
    file=$(grep -rlx -- "id: $probe" "$root/docs/probes")
    awk '/^## Task$/ { on = 1; next } on && /^## / { exit } on { print }' "$file" \
        | awk 'NF { seen = 1 } seen' | awk '{ lines[NR] = $0 } END { n = NR; while (n > 0 && lines[n] == "") n--; for (i = 1; i <= n; i++) print lines[i] }' \
        > "$out/tasks/$probe.md"
    [ -s "$out/tasks/$probe.md" ] || { echo "campaign: $probe has no task body." >&2; exit 5; }
done

# The trees, built once.
if [ ! -d "$out/trees/oracle" ]; then
    rm -rf "$out/trees"
    mkdir -p "$out/trees/base"
    git -C "$root" archive "$head" | tar -x -C "$out/trees/base" || exit 3
    mv "$out/trees/base" "$out/trees/oracle"
    mkdir -p "$out/trees/oracle/engine/target/dev-release"
    cp "$engine" "$out/trees/oracle/engine/target/dev-release/headwater" || exit 3
    sha256sum "$engine" | awk '{ print $1 }' > "$out/engine.sha256"
    sh "$root/tools/probe/ablate.sh" --present "$out/trees/oracle" >/dev/null || exit 3
    # shellcheck disable=SC2086
    sh "$root/tools/probe/seal.sh" "$out/trees/oracle" $probes >/dev/null || exit 3
fi
while IFS= read -r line; do
    # shellcheck disable=SC2086
    set -- $line
    tier=$2 arm=$3
    [ -d "$out/trees/$tier-$arm" ] && continue
    # A component arm is built by `ablate.sh` too (#1659). Before, only the
    # absent arm was, so the `no-hook`, `no-skills`, `no-claude-md` and `mcp`
    # trees were copies of the present tree, and the slice-1 live check found
    # no `.mcp.json` in the `mcp` arm's workspace. The tree is built under
    # another name and moved, so a build that stopped half way is not reused.
    rm -rf "$out/trees/.building"
    cp -a "$out/trees/oracle" "$out/trees/.building" || exit 3
    case $arm in
        present) ;;
        absent) sh "$root/tools/probe/ablate.sh" "$tier" "$out/trees/.building" >/dev/null || exit 3 ;;
        *) sh "$root/tools/probe/ablate.sh" "$tier" "$out/trees/.building" "$arm" >/dev/null || exit 3 ;;
    esac
    mv "$out/trees/.building" "$out/trees/$tier-$arm" || exit 3
done < "$out/lines"

# The jobs, shuffled with the seed. A job's name is its line, its probe's
# position and its repetition, so a resumed batch names each job the same way.
if [ ! -f "$out/jobs" ]; then
    count=${repetitions:-$(sed -n 's/.* x \([0-9]*\) repetitions = .*/\1/p' "$out/plans/L1" | head -1)}
    while IFS= read -r line; do
        # shellcheck disable=SC2086
        set -- $line
        index=$1 tier=$2 arm=$3 category=$4
        shift 4
        rest="$*"
        position=0
        grep "^$index " "$out/pairs" | while read -r _ probe; do
            position=$((position + 1))
            repetition=1
            while [ "$repetition" -le "$count" ]; do
                printf 'L%s-%s-%s-p%s-r%s %s %s %s %s %s %s\n' "$index" "$tier" "$arm" "$position" \
                    "$repetition" "$index" "$tier" "$arm" "$category" "$probe" "$rest"
                repetition=$((repetition + 1))
            done
        done
    done < "$out/lines" \
        | awk -v seed="$seed" 'BEGIN { srand(seed) } { printf "%.12f\t%s\n", rand(), $0 }' \
        | sort -k1,1 | cut -f2- > "$out/jobs"
fi

# The harness is part of every session's identity, and assembly refuses a line
# whose sessions disagree on it. A batch staggered over days can meet another
# `claude`, so the first invocation records its version and a later one on
# another version refuses before it spends. It is asked after the plans, so a
# spec the plan refuses never runs the harness at all.
harness=$(claude --version 2>/dev/null | head -1)
if [ -f "$out/claude-version" ]; then
    if [ "$(cat "$out/claude-version")" != "$harness" ]; then
        echo "campaign: $out was started with claude $(cat "$out/claude-version"), and this harness is ${harness:-of no version}. Its sessions would not assemble with the batch's. Run the rest of the batch on the harness it started with, or start a new batch in another directory." >&2
        exit 4
    fi
else
    printf '%s\n' "$harness" > "$out/claude-version"
fi

total=$(wc -l < "$out/jobs")
echo "campaign: $total sessions over $(wc -l < "$out/lines") lines, $parallel at a time, at $head${max_sessions:+, at most $max_sessions new in this invocation}." >&2
# Each worker runs this script again with one job, so every job reads the
# ceiling from the sessions already recorded.
tr '\n' '\0' < "$out/jobs" | xargs -0 -P "$parallel" -I '{}' sh "$0" --out "$out" --job '{}'

# A job is recorded, failed, or left for the next invocation. A job this
# invocation never started, on the bound or after a halt, is left and did
# not fail, and so is the job whose harness failure raised the halt. A job
# the ceiling, the cap or a moved `HEAD` refused did not record.
recorded=0
failed=0
remain=0
while IFS= read -r row; do
    name=${row%% *}
    status=$(cat "$out/sessions/$name/status" 2>/dev/null)
    if [ "$status" = 0 ]; then
        recorded=$((recorded + 1))
    elif [ -f "$out/slice/refused/$name" ]; then
        failed=$((failed + 1))
    elif [ ! -f "$out/slice/started/$name" ]; then
        remain=$((remain + 1))
    elif [ "$status" = 10 ] && [ -f "$out/slice/halt" ]; then
        remain=$((remain + 1))
    else
        failed=$((failed + 1))
    fi
done < "$out/jobs"
# What the batch spent, with what an earlier attempt of a job run again spent.
spent=0
for cost in "$out"/sessions/*/cost "$out"/sessions/*/spent-before; do
    [ -f "$cost" ] && spent=$((spent + $(cat "$cost")))
done
if [ "$remain" -gt 0 ]; then
    [ -f "$out/slice/halt" ] && echo "campaign: this invocation stopped on a halt." >&2
    echo "campaign: $recorded of $total sessions recorded, $remain remain; run the same command again to continue at $head." >&2
    echo "campaign: $spent cents spent over the batch." >&2
else
    echo "campaign: $recorded of $total sessions recorded, $spent cents spent." >&2
fi
[ "$failed" = 0 ] || exit 7
[ "$remain" = 0 ] || exit 9
exit 0
