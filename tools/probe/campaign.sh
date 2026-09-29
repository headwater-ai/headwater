#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
#
# The batch driver of a paired probe run: every session of a campaign, from one
# pinned commit, in one shuffled order, under a ceiling it enforces while it
# spends.
#
#     sh tools/probe/campaign.sh --out <dir> --model <model> --spec <file> \
#         [--repetitions <n>] [--parallel <n>] [--max-turns <n>] [--seed <n>] \
#         [--cap-cents <n>]
#     sh tools/probe/campaign.sh --out <dir> --assemble
#
# `--cap-cents` stops the batch below a figure a person agreed to for this batch
# alone, such as a pilot, where the tier's ceiling is set for the full run.
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
# it copies is the build of the pinned commit.
#
# ## The trees
#
# The present tree is the archive with the instrument removed
# (`ablate.sh --present`) and sealed against every probe of every line
# (`seal.sh`), so each arm of every run lost the same answer keys. Each absent
# tree is a copy of the present tree with its tier's ablation removed. The
# present tree also serves as the oracle tree that `probe-transform.sh` grafts
# a `patched` artifact into, so both arms are graded by one oracle. Each
# session runs in a fresh copy of its arm's tree, deleted once it is recorded.
#
# ## The order, and the ceiling
#
# Every (line, probe, repetition) is one job. The jobs are shuffled with the
# seed, and `--parallel` workers take them in that order, so a drift inside the
# batch lands on both arms alike. A worker sums the realized cost of every
# recorded session of the job's tier, adds the declared session cost for each
# session in flight, and skips the job if one more session would cross the
# tier's ceiling. The engine refuses at plan time and this refuses at run time.
# The check is not atomic across workers, so the ceiling can be passed by at
# most one declared session cost per worker.
#
# ## Resume
#
# A job whose session directory holds `status` 0 is done and is not run again.
# Run the same command again to finish a batch that stopped, or to record again
# a session whose recorder failed.
#
# ## Assembly
#
# `--assemble` checks that every recorded session of a line carries one
# identity, and writes one identity block and one events block per line under
# `<out>/assembled/`, with a count of sessions, the summed cost, and how many
# sessions the intent hook reached. A person wraps them in a transcript with
# `headwater new`, because a transcript is an authored document of this corpus
# and this script writes nothing inside it.
#
# It needs `git`, `jq`, `tar`, `awk` and the `claude` harness, and it spends
# real money.
#
# ## Exit status
#
#   0   the batch finished, or the assembly was written
#   2   a usage error
#   3   a tool is missing, or no engine is built
#   4   the checkout is dirty, or `HEAD` moved during the batch
#   5   the plan refuses a line of the spec
#   6   the output directory is inside this checkout
#   7   a session of the batch did not record, or a line's sessions disagree on
#       their identity at assembly

set -u

out=
model=
spec=
repetitions=
parallel=1
max_turns=
seed=0
cap=
assemble=0
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
        --assemble) assemble=1; shift ;;
        --job) job=${2:-}; shift 2 ;;
        *) echo "campaign: unknown argument \`$1\`" >&2; exit 2 ;;
    esac
done

case $0 in
    */*) invoked_from=${0%/*} ;;
    *) invoked_from=. ;;
esac
root=$(cd "$invoked_from/../.." && pwd -P)
engine=$root/engine/target/dev-release/headwater
[ -x "$engine" ] || engine=$root/engine/target/release/headwater

[ -n "$out" ] || { echo "campaign: --out is required" >&2; exit 2; }
mkdir -p "$out" || exit 2
out=$(cd "$out" && pwd -P)
case "$out" in
    "$root"|"$root"/*)
        echo "campaign: the output directory is inside this checkout. A workspace there moves the tree the plan fixed." >&2
        exit 6
        ;;
esac

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
    if [ "$(git -C "$root" rev-parse HEAD)" != "$(cat "$out/head")" ]; then
        echo "campaign: HEAD moved during the batch, so $name was not started." >&2
        return 0
    fi
    # The ceiling, at run time.
    ceiling=$(cat "$out/ceiling.$tier")
    unit=$(cat "$out/unit.$tier")
    spent=0
    inflight=0
    for other in "$out"/sessions/*; do
        [ -d "$other" ] || continue
        [ "$(cat "$other/tier" 2>/dev/null)" = "$tier" ] || continue
        if [ -f "$other/cost" ]; then
            spent=$((spent + $(cat "$other/cost")))
        elif [ -f "$other/started" ] && [ ! -f "$other/status" ]; then
            inflight=$((inflight + 1))
        fi
    done
    if [ $((spent + (inflight + 1) * unit)) -gt "$ceiling" ]; then
        echo "campaign: $name skipped: $spent cents spent and $inflight in flight against a $tier ceiling of $ceiling." >&2
        return 0
    fi
    # The cap, across every tier of the batch, where the caller set one.
    if [ -s "$out/cap" ]; then
        all=0
        running=0
        for other in "$out"/sessions/*; do
            [ -d "$other" ] || continue
            if [ -f "$other/cost" ]; then
                all=$((all + $(cat "$other/cost")))
            elif [ -f "$other/started" ] && [ ! -f "$other/status" ]; then
                running=$((running + 1))
            fi
        done
        if [ $((all + (running + 1) * unit)) -gt "$(cat "$out/cap")" ]; then
            echo "campaign: $name skipped: $all cents spent and $running in flight against the batch cap of $(cat "$out/cap")." >&2
            return 0
        fi
    fi
    mkdir -p "$dir"
    printf '%s\n' "$tier" > "$dir/tier"
    : > "$dir/started"
    rm -f "$dir/status" "$dir/cost"
    ws=$out/ws/$name
    rm -rf "$ws"
    cp -a "$out/trees/$tier-$arm" "$ws" || { echo 2 > "$dir/status"; return 0; }
    # shellcheck disable=SC2086
    sh "$root/tools/probe/probe-record.sh" --probe "$probe" --session "$name" \
        --task-file "$out/tasks/$probe.md" --workspace "$ws" --model "$model" \
        --tier "$tier" --arm "$arm" --category "$category" $excludes \
        ${repetitions:+--repetitions "$repetitions"} \
        --max-turns "${max_turns:-$(cat "$out/max-turns.$tier")}" \
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
    rm -rf "$ws"
    printf 'campaign: %s exited %s, %s cents\n' "$name" "$status" "$(cat "$dir/cost" 2>/dev/null || echo '?')" >&2
}

if [ -n "$job" ]; then
    model=$(cat "$out/model")
    repetitions=$(cat "$out/repetitions")
    max_turns=$(cat "$out/max-turns")
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
    while IFS= read -r line; do
        # shellcheck disable=SC2086
        set -- $line
        index=$1 tier=$2 arm=$3 category=$4
        stem=$tier-$arm-$category
        identity=
        cost=0
        count=0
        live=0
        capped=0
        first_at=
        : > "$out/assembled/$stem.events.yaml"
        for dir in "$out"/sessions/L$index-*; do
            [ -d "$dir" ] || continue
            if [ "$(cat "$dir/status" 2>/dev/null)" != 0 ]; then
                echo "campaign: $(basename "$dir") did not record (status $(cat "$dir/status" 2>/dev/null || echo none))." >&2
                failed=1
                continue
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
            sed -n '/^- probe: /,$p' "$dir/record.md" >> "$out/assembled/$stem.events.yaml"
        done
        {
            printf '%s\n' "$identity" | grep -v -e '^tier: ' -e '^arm: '
            printf 'tier: %s\narm: %s\nat: %s\ncost_cents: %s\n' "$tier" "$arm" "$first_at" "$cost"
        } > "$out/assembled/$stem.identity.yaml"
        printf '%s sessions, %s cents, the intent hook live in %s, %s stopped at the turn cap\n' \
            "$count" "$cost" "$live" "$capped" \
            > "$out/assembled/$stem.summary"
        printf 'campaign: %s: %s' "$stem" "$(cat "$out/assembled/$stem.summary")" >&2
        printf '\n' >&2
    done < "$out/lines"
    [ "$failed" = 0 ] || exit 7
    exit 0
fi

# ---------------------------------------------------------------------------
# The batch.
# ---------------------------------------------------------------------------
[ -n "$model" ] && [ -n "$spec" ] || {
    echo "usage: campaign.sh --out <dir> --model <model> --spec <file> [--repetitions <n>] [--parallel <n>] [--max-turns <n>] [--seed <n>]" >&2
    exit 2
}
[ -f "$spec" ] || { echo "campaign: no spec at $spec" >&2; exit 2; }
for tool in git jq tar awk cargo claude; do
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
engine=$root/engine/target/dev-release/headwater
if [ -f "$out/head" ] && [ "$(cat "$out/head")" != "$head" ]; then
    echo "campaign: $out holds a batch of $(cat "$out/head"), and HEAD is $head. Use another directory." >&2
    exit 4
fi
printf '%s\n' "$head" > "$out/head"
printf '%s\n' "$model" > "$out/model"
printf '%s\n' "$repetitions" > "$out/repetitions"
printf '%s\n' "$max_turns" > "$out/max-turns"
printf '%s' "$cap" > "$out/cap"
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
    # shellcheck disable=SC2086
    "$engine" probe plan --root "$root" --tier "$tier" --category "$category" $excludes \
        ${repetitions:+--repetitions "$repetitions"} > "$out/plans/L$index" 2>&1
    if grep -q '^## This run does not start' "$out/plans/L$index"; then
        echo "campaign: line $index ($tier $arm $category) is refused by the plan:" >&2
        sed -n '/^## This run does not start/,$p' "$out/plans/L$index" | sed '1,2d' >&2
        exit 5
    fi
    ceiling=$(sed -n 's/.*against a ceiling of \$\([0-9.]*\)\..*/\1/p' "$out/plans/L$index" | head -1)
    unit=$(sed -n 's/.*at a declared \$\([0-9.]*\) each.*/\1/p' "$out/plans/L$index" | head -1)
    awk -v d="$ceiling" 'BEGIN { printf "%d\n", d * 100 + 0.5 }' > "$out/ceiling.$tier"
    awk -v d="$unit" 'BEGIN { printf "%d\n", d * 100 + 0.5 }' > "$out/unit.$tier"
    # The turn cap the tier declares (#1384). `--max-turns` overrides it for
    # every tier of the batch. A tier that declares none, run with no
    # override, refuses before anything is built, because a batch with no cap
    # lets one session spend without a bound.
    turns=$(sed -n 's/.*stops at a turn cap of \([0-9][0-9]*\).*/\1/p' "$out/plans/L$index" | head -1)
    if [ -z "$turns" ] && [ -z "$max_turns" ]; then
        echo "campaign: line $index ($tier $arm $category) runs a tier that declares no \`max_turns\` in .headwater/probe.yml, and no --max-turns was given." >&2
        exit 2
    fi
    printf '%s\n' "$turns" > "$out/max-turns.$tier"
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
    cp -a "$out/trees/oracle" "$out/trees/$tier-$arm" || exit 3
    if [ "$arm" = absent ]; then
        sh "$root/tools/probe/ablate.sh" "$tier" "$out/trees/$tier-$arm" >/dev/null || exit 3
    fi
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

total=$(wc -l < "$out/jobs")
echo "campaign: $total sessions over $(wc -l < "$out/lines") lines, $parallel at a time, at $head." >&2
# Each worker runs this script again with one job, so every job reads the
# ceiling from the sessions already recorded.
tr '\n' '\0' < "$out/jobs" | xargs -0 -P "$parallel" -I '{}' sh "$0" --out "$out" --job '{}'

failed=0
while IFS= read -r row; do
    name=${row%% *}
    [ "$(cat "$out/sessions/$name/status" 2>/dev/null)" = 0 ] || failed=$((failed + 1))
done < "$out/jobs"
spent=0
for cost in "$out"/sessions/*/cost; do
    [ -f "$cost" ] && spent=$((spent + $(cat "$cost")))
done
echo "campaign: $((total - failed)) of $total sessions recorded, $spent cents spent." >&2
[ "$failed" = 0 ] || exit 7
exit 0
