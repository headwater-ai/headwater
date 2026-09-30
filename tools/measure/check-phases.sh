#!/bin/sh
# Say where the CPU time of one warm `headwater check --strict` goes.
#
#     sh tools/measure/check-phases.sh [runs]
#
# Spec 6 promises a warm commit hook in 200 ms, and until #1450 nothing said
# which part of a warm run was spent where past Phase A. This script is that
# command. It prints one line per stage of the run, the sum of the stages, and
# two figures the stages are held against, and spec 6 quotes them.
#
# The stages come from a measurement build of the engine, which writes one
# `phase <name> <microseconds>` line to standard error for each stage of
# `check`. `engine/crates/check/src/phase.rs` says how it counts. The feature
# that writes them, `phase-times`, is off by default, so no released binary and
# no binary the commit gate runs carries it. Build it into its own profile,
# which the gate never reads:
#
#     sh tools/hw-cargo build --profile phase --features phase-times -p headwater-cli --manifest-path engine/Cargo.toml --locked
#
# `HW_MEASURE_ENGINE` names another binary built with the feature. The script
# also times `headwater check --strict` and `headwater explain --json` of one
# document with `time -p`, the way `mcp-check.sh` does, as user plus system CPU
# time. The explain run is Phase A: walk, parse and graph, and no check.
#
# Each stage figure is the median of `runs` warm runs, 5 by default. Each
# `time -p` figure is the median of `runs` samples, each the mean of a batch of
# runs, 3 by default and `HW_MEASURE_BATCH` otherwise.
#
# The script refuses to print, and exits 1, when the stages do not account for
# the process: when their sum is not within 10% of the warm run that `time -p`
# measured. A stage the build does not mark, or a thread the clock of the
# stages does not see, would read as time that nothing spent. The same holds
# for the part past Phase A: the stages after `load` must sum to within 10% of
# the warm run less the explain run, or the script refuses. It writes only
# under a fresh temporary directory, and it deletes that directory when it ends.
set -eu

repo=$(cd "$(dirname "$0")/../.." && pwd)
runs=${1:-5}
batch=${HW_MEASURE_BATCH:-3}
engine=${HW_MEASURE_ENGINE:-$repo/engine/target/phase/headwater}
if [ ! -x "$engine" ]; then
    printf 'check-phases: no engine at %s. Build it with:\n  sh tools/hw-cargo build --profile phase --features phase-times -p headwater-cli --manifest-path engine/Cargo.toml --locked\n' "$engine" >&2
    exit 2
fi

scratch=$(mktemp -d "${TMPDIR:-/tmp}/hw-check-phases.XXXXXX")

# One run to fill the cache, and to see that the build carries the feature.
# A build without it writes no `phase` line, and the script refuses rather than
# print a sum of nothing.
"$engine" check --strict --root "$repo" > "$scratch/warm.out" 2> "$scratch/warm.err" || true
if ! grep -q '^phase load ' "$scratch/warm.err"; then
    printf 'check-phases: %s wrote no `phase` line. Build it with `--features phase-times`, as the header of this script says. Standard error:\n' "$engine" >&2
    tail -5 "$scratch/warm.err" >&2
    rm -rf "$scratch"
    exit 1
fi
documents=$(awk '$2 == "typed" && NF == 2 { print $1; exit }' "$scratch/warm.out")

# The stages of `runs` warm runs, one file each.
i=0
while [ "$i" -lt "$runs" ]; do
    "$engine" check --strict --root "$repo" > /dev/null 2> "$scratch/phase.$i" || true
    i=$((i + 1))
done
cache=$(grep -v '^phase ' "$scratch/phase.0" | grep 'served from cache' || true)

# The median of each stage over the runs, in milliseconds, in the order the
# engine wrote them.
awk '$1 == "phase" { if (!($2 in n)) order[++k] = $2; v[$2, ++n[$2]] = $3 }
    END {
        for (j = 1; j <= k; j++) {
            s = order[j]
            m = n[s]
            for (a = 1; a <= m; a++) w[a] = v[s, a]
            for (a = 2; a <= m; a++) { x = w[a]; b = a - 1; while (b > 0 && w[b] > x) { w[b + 1] = w[b]; b-- } w[b + 1] = x }
            med = (m % 2) ? w[(m + 1) / 2] : (w[m / 2] + w[m / 2 + 1]) / 2
            printf "%s %.1f\n", s, med / 1000
        }
    }' "$scratch"/phase.* > "$scratch/stages"

# The CPU seconds of one run of the command, as user plus system time.
# `time -p` counts in hundredths of a second, so one sample times a batch of
# runs in one shell and divides.
cpu() {
    /usr/bin/time -p sh -c 'k=$1; shift; i=0; while [ "$i" -lt "$k" ]; do "$@" < /dev/null > /dev/null 2>&1 || true; i=$((i + 1)); done' batch "$batch" "$@" 2> "$scratch/time" || true
    awk -v k="$batch" '$1 == "user" { u = $2 } $1 == "sys" { s = $2 } END { printf "%.4f\n", (u + s) / k }' "$scratch/time"
}

# The median of `runs` samples, in milliseconds.
median() {
    : > "$scratch/samples"
    i=0
    while [ "$i" -lt "$runs" ]; do
        cpu "$@" >> "$scratch/samples"
        i=$((i + 1))
    done
    sort -n "$scratch/samples" | awk '{ v[NR] = $1 } END {
        m = (NR % 2) ? v[(NR + 1) / 2] : (v[NR / 2] + v[NR / 2 + 1]) / 2
        printf "%.0f\n", m * 1000
    }'
}

warm_ms=$(median "$engine" check --strict --root "$repo")
phase_ms=$(median "$engine" explain --json docs/spec/06-engine-architecture.md --root "$repo")

sum=$(awk '{ t += $2 } END { printf "%.0f\n", t }' "$scratch/stages")
past=$(awk 'seen { t += $2 } $1 == "load" { seen = 1 } END { printf "%.0f\n", t }' "$scratch/stages")
rest=$((warm_ms - phase_ms))

# Within 10% of the figure it is held against, or a refusal that says which.
within() {
    awk -v a="$1" -v b="$2" 'BEGIN { d = a - b; if (d < 0) d = -d; exit !(b > 0 && d <= b / 10) }'
}
if ! within "$sum" "$warm_ms"; then
    printf 'check-phases: the stages sum to %s ms and the warm run took %s ms. They are more than 10%% apart, so the stages do not account for the run.\n' "$sum" "$warm_ms" >&2
    cat "$scratch/stages" >&2
    rm -rf "$scratch"
    exit 1
fi
if ! within "$past" "$rest"; then
    printf 'check-phases: the stages after load sum to %s ms and the warm run less Phase A is %s ms. They are more than 10%% apart, so the stages do not account for the part past Phase A.\n' "$past" "$rest" >&2
    cat "$scratch/stages" >&2
    rm -rf "$scratch"
    exit 1
fi

printf 'command           sh tools/measure/check-phases.sh %s\n' "$runs"
printf 'engine            %s\n' "$engine"
printf 'corpus            %s (%s typed documents)\n' "$repo" "$documents"
printf 'host              %s, %s cores, load %s\n' "$(uname -srm)" "$(getconf _NPROCESSORS_ONLN)" "$(cut -d' ' -f1-3 /proc/loadavg)"
printf 'cache             %s\n' "$cache"
printf 'runs              stages: median of %s warm runs; time -p: median of %s samples, each the mean of %s runs; user+system CPU\n' "$runs" "$runs" "$batch"
awk '{ printf "stage %-12s %6.1f ms\n", $1, $2 }' "$scratch/stages"
printf 'stages            %s ms (sum)\n' "$sum"
printf 'check --strict    %s ms (warm cache, time -p)\n' "$warm_ms"
printf 'explain --json    %s ms (Phase A, time -p)\n' "$phase_ms"
printf 'past Phase A      %s ms (stages after load); %s ms (warm less Phase A)\n' "$past" "$rest"

rm -rf "$scratch"
